use color_eyre::eyre::{ContextCompat, Result};
use wgpu::{Backends, Device, DeviceDescriptor, Instance, PowerPreference, Queue};

pub struct GPUContext {
	device: Device,
	queue: Queue,
}

impl GPUContext {
	pub async fn new() -> Result<Self> {
		let instance = Instance::new(&wgpu::InstanceDescriptor {
			backends: Backends::PRIMARY,
			..Default::default()
		});

		let adapter = instance
			.request_adapter(&wgpu::RequestAdapterOptions {
				// We're doing numeric computations,
				// so we care about performance over making a battery lifespan friendly applications.
				power_preference: PowerPreference::HighPerformance,
				compatible_surface: None,
				force_fallback_adapter: false,
			})
			.await
			.wrap_err("Couldn't get adapter")?;

		let (device, queue) = adapter
			.request_device(&DeviceDescriptor::default(), None)
			.await?;

		Ok(Self { device, queue })
	}
}
