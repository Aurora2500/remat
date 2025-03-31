use std::{fs::File, io::Write};

use camera::{Brightness, ExposureAuto, Gain, Gamma};
use color_eyre::eyre::Result;
use video::{test, Video};

mod camera;
mod robot;
mod video;

#[tokio::main]
async fn main() -> Result<()> {
	color_eyre::install()?;
	let _ctx = Video::new();
	test();

	// let (mut cam, mut buffers) = camera::Camera::new("/dev/video0").await?;
	// cam.set::<ExposureAuto>(3);
	// // cam.set::<Exposure>(166);
	// cam.set::<Brightness>(128);
	// cam.set::<Gamma>(133);
	// cam.set::<Gain>(0);

	// for fb in buffers.iter_mut() {
	// 	cam.capture_frame(fb).await?;
	// }

	// let frame_data = cam.capture_frame(&mut buffers[0]).await?;

	// let mut image = File::create("woah.jpeg")?;
	// image.write(frame_data)?;

	// let addr = "169.254.129.110:0";
	// let callback_addr = Some(Ipv4Addr::new(169, 254, 129, 50));

	// let mut r = robot::Robot::start_with_addr(addr, callback_addr)
	// 	.await?;
	// println!("Connected!");
	// r.servo_j([-1.5, -1.5, -1.5, 0., 1.5, 0.], 0.8, 0.1, 0.1, 0.1, 300.0)
	// 	.await?;

	Ok(())
}
