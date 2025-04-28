#![allow(dead_code)]

mod internal;

use std::{
	io,
	marker::PhantomData,
	os::fd::{FromRawFd, OwnedFd},
};

use tokio::io::unix::AsyncFd;

use nix::{
	fcntl::{open, OFlag},
	sys::stat::Mode,
	NixPath,
};

use internal::{
	enable_video_stream, get_dev_settings, set_dev_settings, FrameBufferPool, VideoFormat,
	VideoPixelFormat, BLACKLIGHT_COMPENSATION, BRIGHTNESS, CONTRAST, EXPOSURE, EXPOSURE_AUTO, GAIN,
	GAMMA, HUE, MJPEG_FMT, SATURATION, WHITE_BALANCE, WHITE_BALANCE_AUTO,
};

pub struct CameraStream<'cam, 'buf> {
	cam: &'cam mut Camera,
	buffer: &'buf mut FrameBufferPool,
}

pub struct Camera {
	dev: AsyncFd<OwnedFd>,
}

pub trait CameraSetting {
	const ID: u32;
}

impl Camera {
	pub async fn new<P: NixPath + ?Sized>(path: &P) -> io::Result<(Self, FrameBufferPool)> {
		let fd = open(path, OFlag::O_RDWR | OFlag::O_NONBLOCK, Mode::empty())?;
		// SAFETY: the fd was opened right above, returning if it failed, so this should be safe
		let dev = unsafe { OwnedFd::from_raw_fd(fd) };
		let dev = AsyncFd::new(dev)?;
		let num_buffers = 4;

		let fmt = VideoFormat::new()
			.set_video_capture_type()
			.set_pix_format(VideoPixelFormat {
				width: 1920,
				height: 1080,
				format: MJPEG_FMT,
			});
		fmt.apply(&dev);

		let buffers = FrameBufferPool::new(&dev, num_buffers)?;

		enable_video_stream(&dev);

		Ok((Self { dev }, buffers))
	}

	pub async fn capture_frame<'fb>(
		&mut self,
		buffer: &'fb mut FrameBufferPool,
	) -> io::Result<&'fb [u8]> {
		buffer.capture(&mut self.dev).await
	}

	pub fn stream<'cam, 'buf>(
		&'cam mut self,
		buffer: &'buf mut FrameBufferPool,
	) -> io::Result<CameraStream<'cam, 'buf>> {
		buffer.enqueue(&mut self.dev)?;
		Ok(CameraStream { cam: self, buffer })
	}

	pub fn set<T: CameraSetting>(&mut self, value: i32) {
		set_dev_settings(&self.dev, T::ID, value);
	}

	pub fn get<T: CameraSetting>(&self) -> i32 {
		get_dev_settings(&self.dev, T::ID)
	}
}

impl CameraStream<'_, '_> {
	pub async fn get_frame(&mut self) -> io::Result<&[u8]> {
		self.buffer.dequeue(&mut self.cam.dev).await
	}

	pub async fn stop(self) -> io::Result<()> {
		self.buffer.dequeue_all(&mut self.cam.dev).await
	}
}

struct View<'a, T> {
	cam: &'a Camera,
	_marker: PhantomData<T>,
}

impl<T> View<'_, T>
where
	T: CameraSetting,
{
	pub fn get(&self) -> i32 {
		self.cam.get::<T>()
	}
}

struct ViewMut<'a, T> {
	cam: &'a mut Camera,
	_marker: PhantomData<T>,
}

impl<T> ViewMut<'_, T>
where
	T: CameraSetting,
{
	pub fn get(&self) -> i32 {
		self.cam.get::<T>()
	}

	pub fn set(&mut self, value: i32) {
		self.cam.set::<T>(value)
	}
}

macro_rules! cam_setting {
	($($setting:ident => $id:expr),+ $(,)?) => {
		$(
			pub struct $setting;

			impl CameraSetting for $setting {
				const ID: u32 = $id;
			}
		)+
	};
}

cam_setting! {
	Exposure => EXPOSURE,
	ExposureAuto => EXPOSURE_AUTO,
	Gain => GAIN,
	Gamma => GAMMA,
	Brightness => BRIGHTNESS,
	Contrast => CONTRAST,
	Hue => HUE,
	Saturation => SATURATION,
	WhiteBalance => WHITE_BALANCE,
	BlacklightCompensation => BLACKLIGHT_COMPENSATION,
	WhiteBalanceAuto => WHITE_BALANCE_AUTO,
}
