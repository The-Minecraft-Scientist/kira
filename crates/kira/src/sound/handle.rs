use crate::tween::Tween;
use crate::{CommandError, PlaybackRate, Volume};
use super::static_sound::PlaybackState;

/// Trait defining the Handle API
pub trait Handle {
    /// Returns the current playback state of the sound.
    fn state(&self) -> PlaybackState;

    /// Returns the current playback position of the sound (in seconds).
    fn position(&self) -> f64;

    /// Sets the volume of the sound (as a factor of the original volume).
    fn set_volume(
        &mut self,
        volume: impl Into<Volume>,
        tween: Tween,
    ) -> Result<(), CommandError>;

    /// Sets the playback rate of the sound.
    ///
    /// Changing the playback rate will change both the speed
    /// and pitch of the sound.
    fn set_playback_rate(
        &mut self,
        playback_rate: impl Into<PlaybackRate>,
        tween: Tween,
    ) -> Result<(), CommandError>;

    /// Sets the panning of the sound, where `0.0` is hard left,
    /// `0.5` is center, and `1.0` is hard right.
    fn set_panning(&mut self, panning: f64, tween: Tween) -> Result<(), CommandError>;

    /// Fades out the sound to silence with the given tween and then
    /// pauses playback.
    fn pause(&mut self, tween: Tween) -> Result<(), CommandError>;

    /// Resumes playback and fades in the sound from silence
    /// with the given tween.
    fn resume(&mut self, tween: Tween) -> Result<(), CommandError>;

    /// Fades out the sound to silence with the given tween and then
    /// stops playback.
    ///
    /// Once the sound is stopped, it cannot be restarted.
    fn stop(&mut self, tween: Tween) -> Result<(), CommandError>;

    /// Sets the playback position to the specified time in seconds.
    fn seek_to(&mut self, position: f64) -> Result<(), CommandError>;

    /// Moves the playback position by the specified amount of time in seconds.
    fn seek_by(&mut self, amount: f64) -> Result<(), CommandError>;
}