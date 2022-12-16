use std::sync::Arc;

use ringbuf::HeapProducer;

use crate::{tween::Tween, CommandError, PlaybackRate, Volume};
use crate::sound::handle::Handle;

use super::{sound::Shared, Command, PlaybackState};

/// Controls a static sound.
pub struct StaticSoundHandle {
	pub(super) command_producer: HeapProducer<Command>,
	pub(super) shared: Arc<Shared>,
}

impl StaticSoundHandle {}

impl Handle for StaticSoundHandle {
	/// Returns the current playback state of the sound.
	fn state(&self) -> PlaybackState {
		self.shared.state()
	}

	/// Returns the current playback position of the sound (in seconds).
	fn position(&self) -> f64 {
		self.shared.position()
	}

	/// Sets the volume of the sound (as a factor of the original volume).
	fn set_volume(
		&mut self,
		volume: impl Into<Volume>,
		tween: Tween,
	) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetVolume(volume.into(), tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	/// Sets the playback rate of the sound.
	///
	/// Changing the playback rate will change both the speed
	/// and pitch of the sound.
	fn set_playback_rate(
		&mut self,
		playback_rate: impl Into<PlaybackRate>,
		tween: Tween,
	) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetPlaybackRate(playback_rate.into(), tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	/// Sets the panning of the sound, where `0.0` is hard left,
	/// `0.5` is center, and `1.0` is hard right.
	fn set_panning(&mut self, panning: f64, tween: Tween) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SetPanning(panning, tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	/// Fades out the sound to silence with the given tween and then
	/// pauses playback.
	fn pause(&mut self, tween: Tween) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::Pause(tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	/// Resumes playback and fades in the sound from silence
	/// with the given tween.
	fn resume(&mut self, tween: Tween) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::Resume(tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	/// Fades out the sound to silence with the given tween and then
	/// stops playback.
	///
	/// Once the sound is stopped, it cannot be restarted.
	fn stop(&mut self, tween: Tween) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::Stop(tween))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	/// Sets the playback position to the specified time in seconds.
	fn seek_to(&mut self, position: f64) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SeekTo(position))
			.map_err(|_| CommandError::CommandQueueFull)
	}

	/// Moves the playback position by the specified amount of time in seconds.
	fn seek_by(&mut self, amount: f64) -> Result<(), CommandError> {
		self.command_producer
			.push(Command::SeekBy(amount))
			.map_err(|_| CommandError::CommandQueueFull)
	}
}