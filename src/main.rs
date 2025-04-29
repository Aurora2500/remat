use std::io;

use camera::{Brightness, ExposureAuto, Gain, Gamma};
use color_eyre::eyre::Result;
use video::{Encoder, VideoContext};
use zune_jpeg::{
	zune_core::{colorspace::ColorSpace, options::DecoderOptions},
	JpegDecoder,
};

mod camera;
mod compute;
mod robot;
mod video;

#[tokio::main]
async fn main() -> Result<()> {
	color_eyre::install()?;
	let _ctx = VideoContext::new();
	// test();

	let (mut cam, mut buffers) = camera::Camera::new("/dev/video0").await?;
	cam.set::<ExposureAuto>(3);
	// cam.set::<Exposure>(166);
	cam.set::<Brightness>(128);
	cam.set::<Gamma>(133);
	cam.set::<Gain>(0);

	for _ in 0..buffers.len() {
		cam.capture_frame(&mut buffers).await?;
	}

	// let frame_data = cam.capture_frame(&mut buffers).await?;

	// let mut image = File::create("woah.jpeg")?;
	// image.write(frame_data)?;

	let mut raw = vec![0; 1920 * 1080 * 3];
	let mut out = vec![0; 1920 * 1080 * 3];
	let mut encoder = Encoder::new();
	let mut stream = cam.stream(&mut buffers)?;
	for i in 0..10 {
		println!("{i}");
		stream
			.with_frame(|frame| -> Result<()> {
				let mut decoder = JpegDecoder::new_with_options(
					frame,
					DecoderOptions::new_cmd().jpeg_set_out_colorspace(ColorSpace::YCbCr),
				);
				decoder.decode_into(&mut raw)?;
				Ok(())
			})
			.await??;
		let min = raw.iter().min().unwrap();
		let max = raw.iter().max().unwrap();
		println!("min {min}\nmax{max}");
		println!("[{}, {}, {}]", raw[0], raw[1], raw[2]);
		for i in 0..1080 {
			for j in 0..1920 {
				for k in 0..3 {
					out[k * 1920 * 1080 + i * 1920 + j] = raw[i * 1920 * 3 + j * 3 + k];
				}
			}
		}
		encoder.encode(
			i * 400,
			&out[..1920 * 1080],
			&out[1920 * 1080..1920 * 1080 * 2],
			&out[1920 * 1080 * 2..1920 * 1080 * 3],
		);
	}
	println!("finish");
	encoder.finish();
	stream.stop().await?;

	// let addr = "169.254.129.110:0";
	// let callback_addr = Some(Ipv4Addr::new(169, 254, 129, 50));

	// let mut r = robot::Robot::start_with_addr(addr, callback_addr)
	// 	.await?;
	// println!("Connected!");
	// r.servo_j([-1.5, -1.5, -1.5, 0., 1.5, 0.], 0.8, 0.1, 0.1, 0.1, 300.0)
	// 	.await?;

	Ok(())
}
