mod backend;
mod command;
pub mod error;
mod resources;

use cpal::{
	traits::{DeviceTrait, HostTrait, StreamTrait},
	Stream,
};
use ringbuf::{Producer, RingBuffer};

use self::{
	backend::Backend,
	command::Command,
	error::SetupError,
	resources::{
		create_resources, create_unused_resource_channels, ResourceControllers,
		UnusedResourceConsumers,
	},
};

pub struct AudioManagerSettings {
	pub sound_capacity: usize,
	pub command_capacity: usize,
}

pub struct AudioManager {
	command_producer: Producer<Command>,
	resource_controllers: ResourceControllers,
	unused_resource_consumers: UnusedResourceConsumers,
	_stream: Stream,
}

impl AudioManager {
	pub fn new(settings: AudioManagerSettings) -> Result<Self, SetupError> {
		let host = cpal::default_host();
		let device = host
			.default_output_device()
			.ok_or(SetupError::NoDefaultOutputDevice)?;
		let config = device.default_output_config()?.config();
		let sample_rate = config.sample_rate;
		let channels = config.channels;
		let (unused_resource_producers, unused_resource_consumers) =
			create_unused_resource_channels(&settings);
		let (resources, resource_controllers) =
			create_resources(&settings, unused_resource_producers);
		let (command_producer, command_consumer) =
			RingBuffer::new(settings.command_capacity).split();
		let mut backend = Backend::new(sample_rate.0, resources, command_consumer);
		let stream = device.build_output_stream(
			&config,
			move |data: &mut [f32], _| {
				for frame in data.chunks_exact_mut(channels as usize) {
					let out = backend.process();
					if channels == 1 {
						frame[0] = (out.left + out.right) / 2.0;
					} else {
						frame[0] = out.left;
						frame[1] = out.right;
					}
				}
			},
			move |_| {},
		)?;
		stream.play()?;
		Ok(Self {
			command_producer,
			resource_controllers,
			unused_resource_consumers,
			_stream: stream,
		})
	}
}
