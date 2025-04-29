use std::{
	ffi::c_void,
	io, mem,
	num::NonZeroUsize,
	os::fd::{AsFd, AsRawFd, OwnedFd},
	ptr::NonNull,
	slice,
};

use nix::{
	errno::Errno,
	ioctl_read_bad, ioctl_write_ptr_bad,
	sys::mman::{mmap, munmap, MapFlags, ProtFlags},
};
use tokio::io::unix::AsyncFd;
use v4l2_sys::{
	v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE, v4l2_buffer, v4l2_control,
	v4l2_field_V4L2_FIELD_NONE, v4l2_format, v4l2_memory_V4L2_MEMORY_MMAP, v4l2_requestbuffers,
	V4L2_CID_BACKLIGHT_COMPENSATION, V4L2_CID_BRIGHTNESS, V4L2_CID_CONTRAST,
	V4L2_CID_EXPOSURE_ABSOLUTE, V4L2_CID_EXPOSURE_AUTO, V4L2_CID_GAIN, V4L2_CID_GAMMA,
	V4L2_CID_HUE, V4L2_CID_SATURATION, V4L2_CID_WHITE_BALANCE_TEMPERATURE, VIDIOC_DQBUF,
	VIDIOC_QBUF, VIDIOC_QUERYBUF, VIDIOC_REQBUFS, VIDIOC_STREAMON, VIDIOC_S_CTRL, VIDIOC_S_FMT,
};

pub const EXPOSURE: u32 = V4L2_CID_EXPOSURE_ABSOLUTE;
pub const EXPOSURE_AUTO: u32 = V4L2_CID_EXPOSURE_AUTO;
pub const GAIN: u32 = V4L2_CID_GAIN;
pub const GAMMA: u32 = V4L2_CID_GAMMA;
pub const BRIGHTNESS: u32 = V4L2_CID_BRIGHTNESS;
pub const CONTRAST: u32 = V4L2_CID_CONTRAST;
pub const HUE: u32 = V4L2_CID_HUE;
pub const SATURATION: u32 = V4L2_CID_SATURATION;
pub const WHITE_BALANCE: u32 = V4L2_CID_WHITE_BALANCE_TEMPERATURE;
pub const BLACKLIGHT_COMPENSATION: u32 = V4L2_CID_BACKLIGHT_COMPENSATION;
pub const WHITE_BALANCE_AUTO: u32 = V4L2_CID_WHITE_BALANCE_TEMPERATURE;

#[derive(Debug, Clone, Copy)]
pub struct VideoPixelFormat {
	pub width: u32,
	pub height: u32,
	// field: u32,
	pub format: u32,
}

pub struct VideoFormat(pub v4l2_format);

pub const MJPEG_FMT: u32 = u32::from_ne_bytes(*b"MJPG").swap_bytes();

impl VideoFormat {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn set_type(mut self, ty: u32) -> Self {
		self.0.type_ = ty;
		self
	}

	pub fn set_video_capture_type(self) -> Self {
		self.set_type(v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE)
	}

	pub fn set_pix_format(
		mut self,
		VideoPixelFormat {
			width,
			height,
			format,
		}: VideoPixelFormat,
	) -> Self {
		self.0.fmt.pix.width = width;
		self.0.fmt.pix.height = height;
		self.0.fmt.pix.pixelformat = format;
		self.0.fmt.pix.field = v4l2_field_V4L2_FIELD_NONE;
		self
	}

	pub fn apply(&self, dev: &impl AsRawFd) {
		unsafe {
			set_v4l2_format(dev.as_raw_fd(), &self.0).expect("Failed applying v4l2 format");
		}
	}
}

impl Default for VideoFormat {
	fn default() -> Self {
		Self(unsafe { mem::zeroed() })
	}
}

pub struct FrameBufferPool {
	pool: Box<[FrameBuffer]>,
	idx: u32,
}

struct FrameBuffer {
	data: *mut u8,
	length: usize,
}

impl FrameBufferPool {
	pub fn new(dev: &impl AsFd, count: u32) -> io::Result<Self> {
		let mut req: v4l2_requestbuffers = unsafe { mem::zeroed() };
		req.count = count;
		req.type_ = v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
		req.memory = v4l2_memory_V4L2_MEMORY_MMAP;
		unsafe {
			set_v4l2_reqbufs(dev.as_fd().as_raw_fd(), &req)
				.expect("Failed setting request buffers");
		}
		let pool = (0..count)
			.map(|idx| FrameBuffer::create(dev, idx))
			.collect::<io::Result<_>>()?;

		Ok(Self { pool, idx: 0 })
	}

	pub fn len(&self) -> usize {
		self.pool.len()
	}

	pub(super) async fn capture(&mut self, dev: &mut AsyncFd<OwnedFd>) -> io::Result<&[u8]> {
		let len = self.pool.len() as u32;
		let frame = &mut self.pool[self.idx as usize];
		let image = frame.capture(self.idx, dev).await?;
		self.idx = (self.idx + 1) % len;
		Ok(image)
	}

	pub(super) fn enqueue(&mut self, dev: &mut AsyncFd<OwnedFd>) -> io::Result<()> {
		for (idx, frame) in self.pool.iter_mut().enumerate() {
			frame.enqueue(idx as u32, dev)?;
		}
		Ok(())
	}

	pub(super) async fn dequeue(&mut self, dev: &mut AsyncFd<OwnedFd>) -> io::Result<&[u8]> {
		let buf = loop {
			let mut guard = dev.readable().await?;
			let mut buf: v4l2_buffer = unsafe { mem::zeroed() };
			buf.memory = v4l2_memory_V4L2_MEMORY_MMAP;
			buf.type_ = v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
			let ret = unsafe { v4l2_dequeue(dev.as_raw_fd(), &mut buf) };
			match ret {
				Ok(_) => break buf,
				Err(Errno::EAGAIN) => {
					guard.clear_ready();
					continue;
				}
				Err(e) => return Err(io::Error::from_raw_os_error(e as i32)),
			}
		};
		let active_frame = &mut self.pool[buf.index as usize];
		active_frame.enqueue(buf.index, dev)?;

		unsafe {
			Ok(slice::from_raw_parts(
				active_frame.data,
				buf.bytesused as usize,
			))
		}
	}

	pub(super) async fn dequeue_all(&mut self, dev: &mut AsyncFd<OwnedFd>) -> io::Result<()> {
		for _ in 0..self.len() {
			self.dequeue(dev).await?;
		}
		Ok(())
	}
}

impl FrameBuffer {
	fn create(dev: &impl AsFd, index: u32) -> io::Result<Self> {
		let mut buf: v4l2_buffer = unsafe { mem::zeroed() };
		buf.type_ = v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
		buf.memory = v4l2_memory_V4L2_MEMORY_MMAP;
		buf.index = index;
		unsafe { create_v4l2_buf(dev.as_fd().as_raw_fd(), &buf)? };
		let length = buf.length as usize;
		let data = unsafe {
			mmap(
				None,
				NonZeroUsize::new_unchecked(length),
				ProtFlags::PROT_READ | ProtFlags::PROT_WRITE,
				MapFlags::MAP_SHARED,
				dev,
				buf.m.offset.into(),
			)
			.expect("Failed mmap for buffer")
			.as_ptr() as *mut u8
		};
		Ok(Self { length, data })
	}

	fn enqueue(&mut self, idx: u32, dev: &mut AsyncFd<OwnedFd>) -> io::Result<()> {
		let mut buf: v4l2_buffer = unsafe { mem::zeroed() };
		buf.index = idx;
		buf.memory = v4l2_memory_V4L2_MEMORY_MMAP;
		buf.type_ = v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
		unsafe {
			v4l2_queue(dev.as_raw_fd(), &buf)?;
		}
		Ok(())
	}

	async fn capture(&mut self, idx: u32, dev: &mut AsyncFd<OwnedFd>) -> io::Result<&[u8]> {
		let mut buf: v4l2_buffer = unsafe { mem::zeroed() };
		buf.index = idx;
		buf.memory = v4l2_memory_V4L2_MEMORY_MMAP;
		buf.type_ = v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
		unsafe {
			v4l2_queue(dev.as_raw_fd(), &buf)?;
		}
		let buf = loop {
			let mut guard = dev.readable().await?;
			let mut buf: v4l2_buffer = unsafe { mem::zeroed() };
			buf.memory = v4l2_memory_V4L2_MEMORY_MMAP;
			buf.type_ = v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
			let ret = unsafe { v4l2_dequeue(dev.as_raw_fd(), &mut buf) };
			match ret {
				Ok(_) => break buf,
				Err(Errno::EAGAIN) => {
					guard.clear_ready();
					continue;
				}
				Err(e) => return Err(io::Error::from_raw_os_error(e as i32)),
			}
		};
		unsafe { Ok(slice::from_raw_parts(self.data, buf.bytesused as usize)) }
	}
}

impl Drop for FrameBuffer {
	fn drop(&mut self) {
		let res = unsafe {
			munmap(
				NonNull::<c_void>::new_unchecked(self.data as *mut c_void),
				self.length,
			)
		};
		if let Err(e) = res {
			eprintln!("Munmap error: {e}");
		}
	}
}

pub fn enable_video_stream(dev: &impl AsRawFd) {
	unsafe {
		let ty = v4l2_buf_type_V4L2_BUF_TYPE_VIDEO_CAPTURE;
		enable_v4l2_stream(dev.as_raw_fd(), &ty).expect("Failed turning on stream");
	}
}

pub fn set_dev_settings(dev: &impl AsRawFd, setting: u32, value: i32) {
	let c: v4l2_control = v4l2_control { id: setting, value };
	unsafe {
		v4l2_set_ctrl(dev.as_raw_fd(), &c).expect("Failed updating settings");
	}
}

pub fn get_dev_settings(dev: &impl AsRawFd, setting: u32) -> i32 {
	let mut c: v4l2_control = v4l2_control {
		id: setting,
		value: 0,
	};
	unsafe {
		v4l2_get_ctrl(dev.as_raw_fd(), &mut c).expect("Failed getting settings");
	}
	c.value
}

ioctl_write_ptr_bad!(set_v4l2_format, VIDIOC_S_FMT, v4l2_format);
ioctl_write_ptr_bad!(set_v4l2_reqbufs, VIDIOC_REQBUFS, v4l2_requestbuffers);
ioctl_write_ptr_bad!(create_v4l2_buf, VIDIOC_QUERYBUF, v4l2_buffer);
ioctl_write_ptr_bad!(enable_v4l2_stream, VIDIOC_STREAMON, u32);
ioctl_write_ptr_bad!(v4l2_queue, VIDIOC_QBUF, v4l2_buffer);
ioctl_read_bad!(v4l2_dequeue, VIDIOC_DQBUF, v4l2_buffer);
ioctl_write_ptr_bad!(v4l2_set_ctrl, VIDIOC_S_CTRL, v4l2_control);
ioctl_read_bad!(v4l2_get_ctrl, VIDIOC_S_CTRL, v4l2_control);
