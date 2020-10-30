use crate::{
	command::InstanceCommand,
	command::MetronomeCommand,
	command::{Command, SequenceCommand},
	duration::Duration,
	instance::{InstanceId, InstanceSettings},
	metronome::Metronome,
	sequence::SequenceOutputCommand,
	sequence::{Sequence, SequenceId, SequenceTask},
};
use indexmap::IndexMap;
use ringbuf::Producer;
use std::vec::Drain;

pub(crate) struct Sequences<CustomEvent> {
	sequences: IndexMap<SequenceId, Sequence<CustomEvent>>,
	sequences_to_remove: Vec<SequenceId>,
	sequence_output_command_queue: Vec<SequenceOutputCommand<InstanceId, CustomEvent>>,
	output_command_queue: Vec<Command<CustomEvent>>,
}

impl<CustomEvent: Copy> Sequences<CustomEvent> {
	pub fn new(sequence_capacity: usize, command_capacity: usize) -> Self {
		Self {
			sequences: IndexMap::with_capacity(sequence_capacity),
			sequences_to_remove: Vec::with_capacity(sequence_capacity),
			sequence_output_command_queue: Vec::with_capacity(command_capacity),
			output_command_queue: Vec::with_capacity(command_capacity),
		}
	}

	fn start_sequence(&mut self, id: SequenceId, mut sequence: Sequence<CustomEvent>) {
		sequence.start();
		self.sequences.insert(id, sequence);
	}

	pub fn run_command(&mut self, command: SequenceCommand<CustomEvent>, metronome: &Metronome) {
		match command {
			SequenceCommand::StartSequence(id, sequence) => {
				self.start_sequence(id, sequence);
			}
			SequenceCommand::LoopSound(id, sound_id, loop_settings, instance_settings) => {
				let tempo = sound_id
					.metadata()
					.tempo
					.unwrap_or(metronome.settings.tempo);
				let duration = sound_id
					.metadata()
					.semantic_duration
					.unwrap_or(Duration::Seconds(sound_id.duration()));
				let start = loop_settings
					.start
					.unwrap_or(Duration::Seconds(0.0))
					.in_seconds(tempo);
				let end = loop_settings.end.unwrap_or(duration).in_seconds(tempo);
				let mut sequence = Sequence::new();
				sequence.play_sound(sound_id, instance_settings);
				sequence.wait(Duration::Seconds(end - instance_settings.position));
				sequence.start_loop();
				sequence.play_sound(
					sound_id,
					InstanceSettings {
						position: start,
						..instance_settings
					},
				);
				sequence.wait(Duration::Seconds(end - start));
				self.start_sequence(id, sequence);
			}
			SequenceCommand::MuteSequence(id) => {
				if let Some(sequence) = self.sequences.get_mut(&id) {
					sequence.mute();
				}
			}
			SequenceCommand::UnmuteSequence(id) => {
				if let Some(sequence) = self.sequences.get_mut(&id) {
					sequence.unmute();
				}
			}
			SequenceCommand::PauseSequence(id) => {
				if let Some(sequence) = self.sequences.get_mut(&id) {
					sequence.pause();
				}
			}
			SequenceCommand::ResumeSequence(id) => {
				if let Some(sequence) = self.sequences.get_mut(&id) {
					sequence.resume();
				}
			}
			SequenceCommand::StopSequence(id) => {
				if let Some(sequence) = self.sequences.get_mut(&id) {
					sequence.stop();
				}
			}
		}
	}

	pub fn update(
		&mut self,
		dt: f64,
		metronome: &Metronome,
		sequences_to_unload_producer: &mut Producer<Sequence<CustomEvent>>,
	) -> Drain<Command<CustomEvent>> {
		// update sequences and collect their commands
		for (id, sequence) in &mut self.sequences {
			sequence.update(dt, metronome, &mut self.sequence_output_command_queue);
			if sequence.finished() {
				self.sequences_to_remove.push(*id);
			}
		}
		// remove finished sequences
		for id in self.sequences_to_remove.drain(..) {
			let sequence = self.sequences.remove(&id).unwrap();
			match sequences_to_unload_producer.push(sequence) {
				Ok(_) => {}
				Err(sequence) => {
					self.sequences.insert(id, sequence);
				}
			}
		}
		// convert sequence commands to commands that can be consumed
		// by the backend
		for command in self.sequence_output_command_queue.drain(..) {
			self.output_command_queue.push(match command {
				SequenceOutputCommand::PlaySound(sound_id, instance_id, settings) => {
					Command::Instance(InstanceCommand::PlaySound(sound_id, instance_id, settings))
				}
				SequenceOutputCommand::SetInstanceVolume(instance_id, volume, tween) => {
					Command::Instance(InstanceCommand::SetInstanceVolume(
						instance_id,
						volume,
						tween,
					))
				}
				SequenceOutputCommand::SetInstancePitch(instance_id, pitch, tween) => {
					Command::Instance(InstanceCommand::SetInstancePitch(instance_id, pitch, tween))
				}
				SequenceOutputCommand::PauseInstance(instance_id, fade_tween) => {
					Command::Instance(InstanceCommand::PauseInstance(instance_id, fade_tween))
				}
				SequenceOutputCommand::ResumeInstance(instance_id, fade_tween) => {
					Command::Instance(InstanceCommand::ResumeInstance(instance_id, fade_tween))
				}
				SequenceOutputCommand::StopInstance(instance_id, fade_tween) => {
					Command::Instance(InstanceCommand::StopInstance(instance_id, fade_tween))
				}
				SequenceOutputCommand::PauseInstancesOfSound(sound_id, fade_tween) => {
					Command::Instance(InstanceCommand::PauseInstancesOfSound(sound_id, fade_tween))
				}
				SequenceOutputCommand::ResumeInstancesOfSound(sound_id, fade_tween) => {
					Command::Instance(InstanceCommand::ResumeInstancesOfSound(
						sound_id, fade_tween,
					))
				}
				SequenceOutputCommand::StopInstancesOfSound(sound_id, fade_tween) => {
					Command::Instance(InstanceCommand::StopInstancesOfSound(sound_id, fade_tween))
				}
				SequenceOutputCommand::SetMetronomeTempo(tempo) => {
					Command::Metronome(MetronomeCommand::SetMetronomeTempo(tempo))
				}
				SequenceOutputCommand::StartMetronome => {
					Command::Metronome(MetronomeCommand::StartMetronome)
				}
				SequenceOutputCommand::PauseMetronome => {
					Command::Metronome(MetronomeCommand::PauseMetronome)
				}
				SequenceOutputCommand::StopMetronome => {
					Command::Metronome(MetronomeCommand::StopMetronome)
				}
				SequenceOutputCommand::EmitCustomEvent(event) => Command::EmitCustomEvent(event),
			});
		}
		self.output_command_queue.drain(..)
	}
}
