use std::{fs::File, io::Write};

use camera::{Brightness, ExposureAuto, Gain, Gamma};

mod camera;
mod robot;
mod video;

fn main() {
	let mut cam = camera::Camera::new("/dev/video0");
	cam.set::<ExposureAuto>(3);
	// cam.set::<Exposure>(166);
	cam.set::<Brightness>(128);
	cam.set::<Gamma>(133);
	cam.set::<Gain>(0);

	for _ in 0..10 {
		cam.capture_frame();
	}

	let frame_data = cam.capture_frame();

	let mut image = File::create("woah.jpeg").unwrap();
	image.write(frame_data).expect("Failed to write image");
}
