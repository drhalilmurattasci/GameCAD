//! Frame timing utilities.
//!
//! The [`Clock`] struct tracks delta time, total elapsed time, and frame count,
//! advancing each frame via its [`tick`](Clock::tick) method.

use serde::{Deserialize, Serialize};

/// Time elapsed since the previous frame, in seconds.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct DeltaTime(pub f32);

/// Total time elapsed since the clock was created, in seconds.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct TotalTime(pub f64);

/// Number of frames elapsed since the clock was created.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FrameCount(pub u64);

/// A simple frame clock that tracks [`DeltaTime`], [`TotalTime`], and [`FrameCount`].
#[derive(Debug, Clone)]
pub struct Clock {
    /// Delta time for the current frame.
    pub delta: DeltaTime,
    /// Total elapsed time.
    pub total: TotalTime,
    /// Current frame number (starts at 0).
    pub frame: FrameCount,
    /// The timestamp (in seconds) of the last tick.
    last_tick: f64,
}

impl Default for Clock {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock {
    /// Creates a new clock with all counters at zero.
    pub fn new() -> Self {
        Self {
            delta: DeltaTime(0.0),
            total: TotalTime(0.0),
            frame: FrameCount(0),
            last_tick: 0.0,
        }
    }

    /// Advances the clock.
    ///
    /// `now` is the current time in seconds (from any monotonic source). On the
    /// very first call the delta will be zero; subsequent calls compute the delta
    /// from the previous `now` value.
    pub fn tick(&mut self, now: f64) {
        if self.frame.0 == 0 {
            self.delta = DeltaTime(0.0);
        } else {
            self.delta = DeltaTime((now - self.last_tick) as f32);
        }
        self.last_tick = now;
        self.total = TotalTime(now);
        self.frame.0 += 1;
    }

    /// Returns the delta time in seconds.
    #[inline]
    pub fn delta_seconds(&self) -> f32 {
        self.delta.0
    }

    /// Returns the total elapsed time in seconds.
    #[inline]
    pub fn total_seconds(&self) -> f64 {
        self.total.0
    }

    /// Returns the current frame count.
    #[inline]
    pub fn frame_count(&self) -> u64 {
        self.frame.0
    }
}

// ─────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_tick_zero_delta() {
        let mut clock = Clock::new();
        clock.tick(1.0);
        assert_eq!(clock.delta_seconds(), 0.0);
        assert_eq!(clock.frame_count(), 1);
    }

    #[test]
    fn subsequent_ticks_compute_delta() {
        let mut clock = Clock::new();
        clock.tick(0.0);
        clock.tick(1.0 / 60.0);
        let dt = clock.delta_seconds();
        assert!((dt - 1.0 / 60.0).abs() < 1e-6);
        assert_eq!(clock.frame_count(), 2);
    }

    #[test]
    fn total_time_tracks() {
        let mut clock = Clock::new();
        clock.tick(0.0);
        clock.tick(0.5);
        clock.tick(1.0);
        assert!((clock.total_seconds() - 1.0).abs() < 1e-10);
        assert_eq!(clock.frame_count(), 3);
    }

    #[test]
    fn default_is_zero() {
        let clock = Clock::default();
        assert_eq!(clock.delta_seconds(), 0.0);
        assert_eq!(clock.total_seconds(), 0.0);
        assert_eq!(clock.frame_count(), 0);
    }
}
