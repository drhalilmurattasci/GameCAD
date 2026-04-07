//! Frame timing utilities.
//!
//! The [`Clock`] struct tracks delta time, total elapsed time, and frame count,
//! advancing each frame via its [`tick`](Clock::tick) method.
//!
//! # Examples
//!
//! ```
//! use core::time::Clock;
//!
//! let mut clock = Clock::new();
//! clock.tick(0.0);
//! assert_eq!(clock.delta_seconds(), 0.0); // first tick always zero delta
//!
//! clock.tick(1.0 / 60.0);
//! assert!((clock.delta_seconds() - 1.0 / 60.0).abs() < 1e-6);
//! assert_eq!(clock.frame_count(), 2);
//! ```

use std::fmt;

use serde::{Deserialize, Serialize};

/// Time elapsed since the previous frame, in seconds.
///
/// # Examples
///
/// ```
/// use core::time::DeltaTime;
///
/// let dt = DeltaTime(0.016);
/// assert_eq!(format!("{dt}"), "0.016s");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct DeltaTime(pub f32);

impl fmt::Display for DeltaTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.3}s", self.0)
    }
}

/// Total time elapsed since the clock was created, in seconds.
///
/// # Examples
///
/// ```
/// use core::time::TotalTime;
///
/// let tt = TotalTime(10.5);
/// assert_eq!(format!("{tt}"), "10.500s");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct TotalTime(pub f64);

impl fmt::Display for TotalTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.3}s", self.0)
    }
}

/// Number of frames elapsed since the clock was created.
///
/// # Examples
///
/// ```
/// use core::time::FrameCount;
///
/// let fc = FrameCount(120);
/// assert_eq!(format!("{fc}"), "frame 120");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FrameCount(pub u64);

impl fmt::Display for FrameCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "frame {}", self.0)
    }
}

/// A simple frame clock that tracks [`DeltaTime`], [`TotalTime`], and [`FrameCount`].
///
/// # Examples
///
/// ```
/// use core::time::Clock;
///
/// let mut clock = Clock::new();
/// clock.tick(0.0);
/// clock.tick(0.016);
/// assert_eq!(clock.frame_count(), 2);
/// assert!((clock.delta_seconds() - 0.016).abs() < 1e-6);
/// ```
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

impl fmt::Display for Clock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Clock({}, dt: {}, total: {})",
            self.frame, self.delta, self.total,
        )
    }
}

impl Clock {
    /// Creates a new clock with all counters at zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::time::Clock;
    ///
    /// let clock = Clock::new();
    /// assert_eq!(clock.frame_count(), 0);
    /// assert_eq!(clock.delta_seconds(), 0.0);
    /// ```
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
    ///
    /// # Examples
    ///
    /// ```
    /// use core::time::Clock;
    ///
    /// let mut clock = Clock::new();
    /// clock.tick(0.0);
    /// assert_eq!(clock.delta_seconds(), 0.0);
    /// clock.tick(0.5);
    /// assert!((clock.delta_seconds() - 0.5).abs() < 1e-6);
    /// ```
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

    #[test]
    fn display_impls() {
        assert_eq!(format!("{}", DeltaTime(0.016)), "0.016s");
        assert_eq!(format!("{}", TotalTime(10.5)), "10.500s");
        assert_eq!(format!("{}", FrameCount(42)), "frame 42");
    }

    #[test]
    fn clock_display() {
        let mut clock = Clock::new();
        clock.tick(0.0);
        let s = format!("{clock}");
        assert!(s.contains("Clock"));
        assert!(s.contains("frame 1"));
    }

    #[test]
    fn many_ticks() {
        let mut clock = Clock::new();
        for i in 0..1000 {
            clock.tick(i as f64 / 60.0);
        }
        assert_eq!(clock.frame_count(), 1000);
    }
}
