//! Animation system with easing functions for smooth UI transitions.
//!
//! Provides:
//! - Easing functions (ease-in, ease-out, elastic, bounce)
//! - Tween interpolation for smooth value transitions
//! - Animation state management

#![allow(dead_code)]

use std::time::{Duration, Instant};

/// Easing functions for smooth animations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Easing {
    /// Linear interpolation (no easing)
    Linear,
    /// Quadratic ease-in (slow start)
    EaseIn,
    /// Quadratic ease-out (slow end)
    EaseOut,
    /// Quadratic ease-in-out (slow start and end)
    EaseInOut,
    /// Cubic ease-in (slower start)
    CubicIn,
    /// Cubic ease-out (slower end)
    CubicOut,
    /// Cubic ease-in-out
    CubicInOut,
    /// Elastic bounce at end
    Elastic,
    /// Bounce effect at end
    Bounce,
    /// Spring physics
    Spring,
    /// Overshoot and return
    Back,
}

impl Easing {
    /// Apply the easing function to a progress value (0.0 to 1.0)
    pub fn apply(&self, t: f64) -> f64 {
        let t = t.clamp(0.0, 1.0);

        match self {
            Self::Linear => t,
            Self::EaseIn => t * t,
            Self::EaseOut => t * (2.0 - t),
            Self::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    -1.0 + (4.0 - 2.0 * t) * t
                }
            }
            Self::CubicIn => t * t * t,
            Self::CubicOut => {
                let t = t - 1.0;
                t * t * t + 1.0
            }
            Self::CubicInOut => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    let t = t - 1.0;
                    1.0 + 4.0 * t * t * t
                }
            }
            Self::Elastic => {
                if t == 0.0 || t == 1.0 {
                    t
                } else {
                    let p = 0.3;
                    let s = p / 4.0;
                    2.0_f64.powf(-10.0 * t) * ((t - s) * (2.0 * std::f64::consts::PI) / p).sin()
                        + 1.0
                }
            }
            Self::Bounce => {
                let t = 1.0 - t;
                let result = if t < 1.0 / 2.75 {
                    7.5625 * t * t
                } else if t < 2.0 / 2.75 {
                    let t = t - 1.5 / 2.75;
                    7.5625 * t * t + 0.75
                } else if t < 2.5 / 2.75 {
                    let t = t - 2.25 / 2.75;
                    7.5625 * t * t + 0.9375
                } else {
                    let t = t - 2.625 / 2.75;
                    7.5625 * t * t + 0.984375
                };
                1.0 - result
            }
            Self::Spring => {
                let omega = 10.0;
                let zeta = 0.5;
                let t_adj = t.max(0.0001);
                1.0 - ((-zeta * omega * t_adj).exp()
                    * ((omega * (1.0 - zeta * zeta).sqrt() * t_adj).cos()
                        + zeta / (1.0 - zeta * zeta).sqrt()
                            * (omega * (1.0 - zeta * zeta).sqrt() * t_adj).sin()))
            }
            Self::Back => {
                let s = 1.70158;
                t * t * ((s + 1.0) * t - s)
            }
        }
    }
}

/// A tween for animating between two values.
#[derive(Debug, Clone)]
pub struct Tween<T: Tweenable> {
    /// Starting value
    from: T,
    /// Ending value
    to: T,
    /// Animation duration
    duration: Duration,
    /// Easing function
    easing: Easing,
    /// When the animation started
    started_at: Option<Instant>,
    /// Whether the animation is complete
    complete: bool,
}

/// Trait for values that can be tweened.
pub trait Tweenable: Clone {
    /// Interpolate between two values.
    fn lerp(&self, other: &Self, t: f64) -> Self;
}

impl Tweenable for f64 {
    fn lerp(&self, other: &Self, t: f64) -> Self {
        self + (other - self) * t
    }
}

impl Tweenable for f32 {
    fn lerp(&self, other: &Self, t: f64) -> Self {
        self + (other - self) * t as f32
    }
}

impl Tweenable for u8 {
    fn lerp(&self, other: &Self, t: f64) -> Self {
        let from = *self as f64;
        let to = *other as f64;
        (from + (to - from) * t).round() as u8
    }
}

impl Tweenable for u16 {
    fn lerp(&self, other: &Self, t: f64) -> Self {
        let from = *self as f64;
        let to = *other as f64;
        (from + (to - from) * t).round() as u16
    }
}

impl Tweenable for (u8, u8, u8) {
    fn lerp(&self, other: &Self, t: f64) -> Self {
        (
            self.0.lerp(&other.0, t),
            self.1.lerp(&other.1, t),
            self.2.lerp(&other.2, t),
        )
    }
}

impl<T: Tweenable> Tween<T> {
    /// Create a new tween.
    pub fn new(from: T, to: T, duration: Duration, easing: Easing) -> Self {
        Self {
            from,
            to,
            duration,
            easing,
            started_at: None,
            complete: false,
        }
    }

    /// Start the animation.
    pub fn start(&mut self) {
        self.started_at = Some(Instant::now());
        self.complete = false;
    }

    /// Reset the animation.
    pub fn reset(&mut self) {
        self.started_at = None;
        self.complete = false;
    }

    /// Check if the animation has started.
    pub fn is_started(&self) -> bool {
        self.started_at.is_some()
    }

    /// Check if the animation is complete.
    pub fn is_complete(&self) -> bool {
        self.complete
    }

    /// Get the current value.
    pub fn value(&mut self) -> T {
        let Some(started_at) = self.started_at else {
            return self.from.clone();
        };

        let elapsed = started_at.elapsed();
        if elapsed >= self.duration {
            self.complete = true;
            return self.to.clone();
        }

        let progress = elapsed.as_secs_f64() / self.duration.as_secs_f64();
        let eased = self.easing.apply(progress);
        self.from.lerp(&self.to, eased)
    }

    /// Update the target value (for chained animations).
    pub fn retarget(&mut self, new_to: T) {
        if self.started_at.is_some() {
            self.from = self.value();
        }
        self.to = new_to;
        self.started_at = Some(Instant::now());
        self.complete = false;
    }
}

/// State for managing multiple animations.
#[derive(Debug, Clone)]
pub struct AnimationState {
    /// Current frame number
    pub frame: u64,
    /// Target FPS
    pub target_fps: u32,
    /// Frame duration
    pub frame_duration: Duration,
    /// Last update time
    pub last_update: Instant,
    /// Whether animations are enabled
    pub enabled: bool,
}

impl Default for AnimationState {
    fn default() -> Self {
        Self::new(60)
    }
}

impl AnimationState {
    /// Create a new animation state with target FPS.
    pub fn new(target_fps: u32) -> Self {
        Self {
            frame: 0,
            target_fps,
            frame_duration: Duration::from_secs_f64(1.0 / target_fps as f64),
            last_update: Instant::now(),
            enabled: true,
        }
    }

    /// Disable animations.
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Default::default()
        }
    }

    /// Check if enough time has passed for a new frame.
    pub fn should_update(&self) -> bool {
        self.enabled && self.last_update.elapsed() >= self.frame_duration
    }

    /// Update the animation state for a new frame.
    pub fn tick(&mut self) {
        if self.should_update() {
            self.frame = self.frame.wrapping_add(1);
            self.last_update = Instant::now();
        }
    }

    /// Get a spinner character based on the current frame.
    pub fn spinner_char<'a>(&self, chars: &'a [&'a str]) -> &'a str {
        if chars.is_empty() {
            return "";
        }
        chars[(self.frame as usize) % chars.len()]
    }

    /// Get a pulsing intensity (0.0 to 1.0) for effects.
    pub fn pulse(&self, period_frames: u64) -> f64 {
        if period_frames == 0 {
            return 1.0;
        }
        let phase = (self.frame % period_frames) as f64 / period_frames as f64;
        (phase * 2.0 * std::f64::consts::PI).sin() * 0.5 + 0.5
    }

    /// Get a color that pulses between two RGB values.
    pub fn pulse_color(
        &self,
        from: (u8, u8, u8),
        to: (u8, u8, u8),
        period_frames: u64,
    ) -> (u8, u8, u8) {
        let t = self.pulse(period_frames);
        from.lerp(&to, t)
    }
}

/// Pre-defined animation durations.
pub mod durations {
    use std::time::Duration;

    /// Very fast animation (100ms)
    pub const INSTANT: Duration = Duration::from_millis(100);
    /// Fast animation (200ms)
    pub const FAST: Duration = Duration::from_millis(200);
    /// Normal animation (300ms)
    pub const NORMAL: Duration = Duration::from_millis(300);
    /// Slow animation (500ms)
    pub const SLOW: Duration = Duration::from_millis(500);
    /// Very slow animation (1s)
    pub const VERY_SLOW: Duration = Duration::from_secs(1);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_easing_linear() {
        let easing = Easing::Linear;
        assert!((easing.apply(0.0) - 0.0).abs() < 0.001);
        assert!((easing.apply(0.5) - 0.5).abs() < 0.001);
        assert!((easing.apply(1.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_easing_ease_in() {
        let easing = Easing::EaseIn;
        assert!((easing.apply(0.0) - 0.0).abs() < 0.001);
        assert!(easing.apply(0.5) < 0.5); // Should be slower at start
        assert!((easing.apply(1.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_easing_ease_out() {
        let easing = Easing::EaseOut;
        assert!((easing.apply(0.0) - 0.0).abs() < 0.001);
        assert!(easing.apply(0.5) > 0.5); // Should be faster at start
        assert!((easing.apply(1.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_easing_clamps_input() {
        let easing = Easing::Linear;
        assert!((easing.apply(-0.5) - 0.0).abs() < 0.001);
        assert!((easing.apply(1.5) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_tween_f64() {
        let mut tween = Tween::new(
            0.0_f64,
            100.0_f64,
            Duration::from_millis(100),
            Easing::Linear,
        );
        assert!((tween.value() - 0.0).abs() < 0.001);

        tween.start();
        // Immediately after start, should still be near 0
        assert!(tween.value() < 50.0);
    }

    #[test]
    fn test_tween_color() {
        let from = (0_u8, 0_u8, 0_u8);
        let to = (255_u8, 255_u8, 255_u8);
        let result = from.lerp(&to, 0.5);
        assert_eq!(result, (128, 128, 128));
    }

    #[test]
    fn test_animation_state_new() {
        let state = AnimationState::new(60);
        assert_eq!(state.target_fps, 60);
        assert!(state.enabled);
    }

    #[test]
    fn test_animation_state_disabled() {
        let state = AnimationState::disabled();
        assert!(!state.enabled);
        assert!(!state.should_update());
    }

    #[test]
    fn test_spinner_char() {
        let state = AnimationState::new(60);
        let chars = &["a", "b", "c"];
        let char = state.spinner_char(chars);
        assert!(chars.contains(&char));
    }

    #[test]
    fn test_pulse() {
        let state = AnimationState::new(60);
        let pulse = state.pulse(60);
        assert!((0.0..=1.0).contains(&pulse));
    }
}
