use std::{fs::File, io::Write, net::Ipv4Addr};

use camera::{Brightness, ExposureAuto, Gain, Gamma};
use tokio::runtime::Runtime;

mod camera;
mod robot;
mod video;

fn main() {
	// let mut cam = camera::Camera::new("/dev/video0");
	// cam.set::<ExposureAuto>(3);
	// // cam.set::<Exposure>(166);
	// cam.set::<Brightness>(128);
	// cam.set::<Gamma>(133);
	// cam.set::<Gain>(0);

	// for _ in 0..10 {
	// 	cam.capture_frame();
	// }

	// let frame_data = cam.capture_frame();

	// let mut image = File::create("woah.jpeg").unwrap();
	// image.write(frame_data).expect("Failed to write image");

	let addr = "169.254.129.110:0";
	let callback_addr = Some(Ipv4Addr::new(169, 254, 129, 50));

	let rt = Runtime::new().unwrap();
	rt.block_on(async move {
		let mut r = robot::Robot::start_with_addr(addr, callback_addr)
			.await
			.expect("Failed to connect to robot");
		println!("Connected!");
		r.servo_j([-1.5, -1.5, -1.5, 0., 1.5, 0.], 0.8, 0.1, 0.1, 0.1, 300.0)
			.await
			.expect("Failed ServoJ");
	})
}
