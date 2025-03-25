use std::{fs::File, marker::PhantomData, path::Path};

use crate::video::{
	enable_video_stream, get_dev_settings, set_dev_settings, FrameBufferPool, VideoFormat,
	VideoPixelFormat, BLACKLIGHT_COMPENSATION, BRIGHTNESS, CONTRAST, EXPOSURE, EXPOSURE_AUTO, GAIN,
	GAMMA, HUE, MJPEG_FMT, SATURATION, WHITE_BALANCE, WHITE_BALANCE_AUTO,
};

pub struct Camera {
	dev: File,
	frame_buffers: FrameBufferPool,
}

pub trait CameraSetting {
	const ID: u32;
}

impl Camera {
	pub fn new<P: AsRef<Path>>(path: P) -> Self {
		let dev = File::options()
			.read(true)
			.write(true)
			.open(path)
			.expect("Failed to open device");
		let num_buffers = 4;

		let fmt = VideoFormat::new()
			.set_video_capture_type()
			.set_pix_format(VideoPixelFormat {
				width: 1920,
				height: 1080,
				format: MJPEG_FMT,
			});
		fmt.apply(&dev);

		let frame_buffers = FrameBufferPool::create(&dev, num_buffers);

		enable_video_stream(&dev);

		Self { dev, frame_buffers }
	}

	pub fn capture_frame(&mut self) -> &[u8] {
		self.frame_buffers.capture(&self.dev)
	}

	pub fn set<T: CameraSetting>(&mut self, value: i32) {
		set_dev_settings(&self.dev, T::ID, value);
	}

	pub fn get<T: CameraSetting>(&self) -> i32 {
		get_dev_settings(&self.dev, T::ID)
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
