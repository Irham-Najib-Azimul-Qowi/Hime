//! Analytic Apple Motion damped harmonic springs for 60/120Hz native UI motion.
//!
//! Combines `liquid-rust`'s closed-form `Spring` (`duration` / `bounce` parameterization)
//! and `Animated` (with `launch_to` for `EMERGE`/`RECEDE` surfaces) with backwards-compatible
//! [`SpringValue`] powered by the same analytic solver.

use std::f32::consts::TAU;

/// Position threshold below which [`SpringValue`] is considered settled.
const SETTLE_POS_EPSILON: f32 = 0.02;
/// Velocity threshold below which [`SpringValue`] is considered settled.
const SETTLE_VEL_EPSILON: f32 = 0.05;
/// Upper clamp for `SpringValue::tick` delta-time to avoid huge jumps after thread stalls.
const MAX_FRAME_DT: f32 = 0.05;
/// Rest threshold for [`Animated`].
const REST_EPSILON: f32 = 1e-3;
/// Maximum ULP steps near large float targets before landing at rest.
const REST_STEPS: f32 = 16.0;

/// Closed-form solution of a damped harmonic oscillator with natural frequency `w0`
/// and damping ratio `zeta`, advancing displacement `x0` and velocity `v0` by `t` seconds.
#[inline]
fn solve_harmonic(w0: f32, zeta: f32, x0: f32, v0: f32, t: f32) -> (f32, f32) {
    if zeta < 1.0 {
        let a = zeta * w0;
        let wd = w0 * (1.0 - zeta * zeta).sqrt().max(1e-5);
        let (s, c) = (wd * t).sin_cos();
        let e = (-a * t).exp();
        let b = (v0 + a * x0) / wd;
        (e * (x0 * c + b * s), e * (v0 * c - (x0 * wd + a * b) * s))
    } else if (zeta - 1.0).abs() <= 1e-6 {
        let e = (-w0 * t).exp();
        let b = v0 + w0 * x0;
        (e * (x0 + b * t), e * (v0 - w0 * b * t))
    } else {
        let q = (zeta * zeta - 1.0).sqrt();
        let (r1, r2) = (-w0 * (zeta - q), -w0 * (zeta + q));
        let denom = r2 - r1;
        if denom.abs() < 1e-6 {
            let e = (-w0 * t).exp();
            let b = v0 + w0 * x0;
            return (e * (x0 + b * t), e * (v0 - w0 * b * t));
        }
        let c2 = (v0 - r1 * x0) / denom;
        let c1 = x0 - c2;
        let (e1, e2) = ((r1 * t).exp(), (r2 * t).exp());
        (c1 * e1 + c2 * e2, c1 * r1 * e1 + c2 * r2 * e2)
    }
}

/// Spring timing in Apple's terms: `duration` is the perceptual settle time in
/// seconds, `bounce` is 0 for no overshoot, positive for bounce, negative for a
/// flatter, over-damped curve.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spring {
    /// Perceptual duration in seconds.
    pub duration: f32,
    /// Bounce in `-1.0..1.0`. Clamped to `-0.95..=0.95` when solving so springs always settle.
    pub bounce: f32,
}

impl Spring {
    /// SwiftUI `.smooth`: no overshoot.
    pub const SMOOTH: Self = Self::new(0.5, 0.0);
    /// SwiftUI `.snappy`: small crisp response used for layout and morphs.
    pub const SNAPPY: Self = Self::new(0.5, 0.15);
    /// SwiftUI `.bouncy`: visible overshoot, used for playful release feedback.
    pub const BOUNCY: Self = Self::new(0.5, 0.3);
    /// SwiftUI `.interactiveSpring`: tight, responsive tracking of hover/drag.
    pub const INTERACTIVE: Self = Self::new(0.15, 0.14);
    /// Surface opening (menus, popovers, drawers, expanded capsules): leaves at full speed
    /// with `launch_to(..., 1.0)` and settles without overshoot.
    pub const EMERGE: Self = Self::new(0.45, 0.0);
    /// Surface closing back into its trigger or edge: quicker exponential settle, never bounces.
    pub const RECEDE: Self = Self::new(0.3, 0.0);
    /// Slider or switch knob settling back after release: ~0.18s, zero wobble.
    pub const KNOB_SETTLE: Self = Self::new(0.18, 0.0);

    /// Creates a spring from a perceptual `duration` (seconds) and `bounce`.
    pub const fn new(duration: f32, bounce: f32) -> Self {
        Self { duration, bounce }
    }

    /// Critically damped variant of this spring for Reduce Motion accessibility.
    pub fn without_bounce(self) -> Self {
        Self::new(self.duration * 0.8, self.bounce.min(0.0))
    }

    /// Natural frequency `ω0 = 2π / duration` in radians per second.
    pub fn angular_frequency(self) -> f32 {
        TAU / self.duration.max(1e-3)
    }

    /// Damping ratio `ζ` derived from `bounce`.
    pub fn damping_ratio(self) -> f32 {
        let bounce = self.bounce.clamp(-0.95, 0.95);
        if bounce >= 0.0 {
            1.0 - bounce
        } else {
            1.0 / (1.0 + bounce)
        }
    }

    /// Converts this Apple `(duration, bounce)` spring into unit-mass `(stiffness, damping)`.
    pub fn to_stiffness_damping(self) -> (f32, f32) {
        let w0 = self.angular_frequency();
        let zeta = self.damping_ratio();
        (w0 * w0, 2.0 * zeta * w0)
    }

    /// Advances displacement `x0` and velocity `v0` by `t` seconds, returning `(x, v)`.
    pub fn solve(self, x0: f32, v0: f32, t: f32) -> (f32, f32) {
        solve_harmonic(self.angular_frequency(), self.damping_ratio(), x0, v0, t)
    }
}

/// A scalar driven by a [`Spring`] toward a target, with velocity-preserving retargeting
/// and `launch_to` support for instant `EMERGE`/`RECEDE` surface transitions.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Animated {
    pub value: f32,
    pub velocity: f32,
    pub target: f32,
    pub spring: Spring,
}

impl Animated {
    /// Creates a value at rest.
    pub const fn new(value: f32, spring: Spring) -> Self {
        Self {
            value,
            velocity: 0.0,
            target: value,
            spring,
        }
    }

    /// Retargets while preserving current value and velocity (interruptible motion).
    pub fn animate_to(&mut self, target: f32, spring: Spring) {
        self.target = target;
        self.spring = spring;
    }

    /// Retargets and *launches*: the value leaves toward `target` at `launch × ω0 × distance`.
    /// At `launch = 1.0` on a bounce-0 spring (`EMERGE`/`RECEDE`), displacement decays as a pure
    /// exponential `e^(-ω0·t)` with immediate first-frame response and zero overshoot.
    pub fn launch_to(&mut self, target: f32, spring: Spring, launch: f32) {
        let speed = (target - self.value) * spring.angular_frequency() * launch.max(0.0);
        let faster = self.velocity * speed > 0.0 && self.velocity.abs() > speed.abs();
        if !faster {
            self.velocity = speed;
        }
        self.animate_to(target, spring);
    }

    /// Jumps to `value` immediately with zero velocity.
    pub fn snap(&mut self, value: f32) {
        *self = Self::new(value, self.spring);
    }

    /// Advances by `dt` seconds. Returns `true` while still moving.
    pub fn step(&mut self, dt: f32) -> bool {
        if self.is_at_rest() {
            self.value = self.target;
            self.velocity = 0.0;
            return false;
        }
        let (x, v) = self
            .spring
            .solve(self.value - self.target, self.velocity, dt);
        if !(x.is_finite() && v.is_finite()) {
            if !self.target.is_finite() {
                self.target = if self.value.is_finite() {
                    self.value
                } else {
                    0.0
                };
            }
            self.value = self.target;
            self.velocity = 0.0;
            return false;
        }
        let value = self.target + x;
        if dt > 0.0 && value == self.value && x.abs() <= self.stall_distance(dt) {
            self.value = self.target;
            self.velocity = 0.0;
            return false;
        }
        self.value = value;
        self.velocity = v;
        true
    }

    /// `true` when the value sits at its target with negligible velocity.
    pub fn is_at_rest(&self) -> bool {
        let offset = self.value - self.target;
        if offset.abs() < REST_EPSILON && self.velocity.abs() < REST_EPSILON {
            return true;
        }
        let limit = REST_STEPS * self.float_step();
        offset.abs() <= limit
            && offset.hypot(self.velocity / self.spring.angular_frequency()) <= limit
    }

    fn float_step(&self) -> f32 {
        self.target.abs().next_up() - self.target.abs()
    }

    fn stall_distance(&self, dt: f32) -> f32 {
        let creep_per_offset =
            self.spring.angular_frequency() / (2.0 * self.spring.damping_ratio());
        self.float_step() / (creep_per_offset * dt.max(1e-3))
    }
}

/// A 1D damped harmonic oscillator solved analytically (framerate-independent across 60/120Hz).
/// Preserves full compatibility with `(stiffness, damping, mass)` while supporting Apple [`Spring`] presets.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpringValue {
    pub value: f32,
    pub target: f32,
    pub velocity: f32,
    pub stiffness: f32,
    pub damping: f32,
    pub mass: f32,
}

impl SpringValue {
    /// Creates a new spring resting at `initial` with the given physical parameters.
    pub fn new(initial: f32, stiffness: f32, damping: f32, mass: f32) -> Self {
        Self {
            value: initial,
            target: initial,
            velocity: 0.0,
            stiffness: stiffness.max(0.01),
            damping: damping.max(0.0),
            mass: mass.max(0.01),
        }
    }

    /// Creates a new spring resting at `initial` configured from an Apple [`Spring`] preset.
    pub fn from_spring(initial: f32, spring: Spring) -> Self {
        let (stiffness, damping) = spring.to_stiffness_damping();
        Self::new(initial, stiffness, damping, 1.0)
    }

    /// Snappy island resize preset (stiffness 320, damping 28, mass 1) solved analytically.
    pub fn snappy(initial: f32) -> Self {
        Self::new(initial, 320.0, 28.0, 1.0)
    }

    /// Gentle fade/opacity preset (stiffness 180, damping 24, mass 1) solved analytically.
    pub fn gentle(initial: f32) -> Self {
        Self::new(initial, 180.0, 24.0, 1.0)
    }

    /// Apple `.smooth` preset alias.
    pub fn smooth(initial: f32) -> Self {
        Self::from_spring(initial, Spring::SMOOTH)
    }

    /// Apple `.bouncy` preset.
    pub fn bouncy(initial: f32) -> Self {
        Self::from_spring(initial, Spring::BOUNCY)
    }

    /// Apple `.interactiveSpring` preset (tight hover/press/drag tracking).
    pub fn interactive(initial: f32) -> Self {
        Self::from_spring(initial, Spring::INTERACTIVE)
    }

    /// Apple `EMERGE` preset for opening surfaces.
    pub fn emerge(initial: f32) -> Self {
        Self::from_spring(initial, Spring::EMERGE)
    }

    /// Apple `RECEDE` preset for closing surfaces.
    pub fn recede(initial: f32) -> Self {
        Self::from_spring(initial, Spring::RECEDE)
    }

    /// Apple `KNOB_SETTLE` preset for slider/toggle release settle.
    pub fn knob_settle(initial: f32) -> Self {
        Self::from_spring(initial, Spring::KNOB_SETTLE)
    }

    /// Updates physical parameters from `spring` without resetting value or velocity.
    pub fn set_spring(&mut self, spring: Spring) {
        let (stiffness, damping) = spring.to_stiffness_damping();
        self.stiffness = stiffness.max(0.01);
        self.damping = damping.max(0.0);
        self.mass = 1.0;
    }

    /// Sets a new animation target without resetting current velocity.
    pub fn set_target(&mut self, target: f32) {
        self.target = target;
    }

    /// Retargets with an Apple [`Spring`] preset while preserving velocity.
    pub fn animate_to(&mut self, target: f32, spring: Spring) {
        self.set_spring(spring);
        self.target = target;
    }

    /// Retargets and launches with `launch × ω0 × distance` initial velocity (use `1.0` with
    /// [`Spring::EMERGE`] / [`Spring::RECEDE`] for immediate, lag-free surface open/close).
    pub fn launch_to(&mut self, target: f32, spring: Spring, launch: f32) {
        let speed = (target - self.value) * spring.angular_frequency() * launch.max(0.0);
        let faster = self.velocity * speed > 0.0 && self.velocity.abs() > speed.abs();
        if !faster {
            self.velocity = speed;
        }
        self.animate_to(target, spring);
    }

    /// Immediately snaps value and target to `value` with zero velocity.
    pub fn snap_to(&mut self, value: f32) {
        self.value = value;
        self.target = value;
        self.velocity = 0.0;
    }

    /// Advances the spring simulation analytically by `dt` seconds and returns the updated value.
    pub fn tick(&mut self, dt: f32) -> f32 {
        if dt <= 0.0 || !dt.is_finite() {
            return self.value;
        }
        if self.is_settled() {
            self.value = self.target;
            self.velocity = 0.0;
            return self.value;
        }

        let clamped_dt = dt.min(MAX_FRAME_DT);
        let mass = self.mass.max(0.01);
        let stiffness = self.stiffness.max(0.01);
        let w0 = (stiffness / mass).sqrt();
        let zeta = self.damping.max(0.0) / (2.0 * (stiffness * mass).sqrt());

        let (x, v) = solve_harmonic(w0, zeta, self.value - self.target, self.velocity, clamped_dt);
        if !(x.is_finite() && v.is_finite()) {
            if !self.target.is_finite() {
                self.target = if self.value.is_finite() {
                    self.value
                } else {
                    0.0
                };
            }
            self.value = self.target;
            self.velocity = 0.0;
            return self.value;
        }

        self.value = self.target + x;
        self.velocity = v;

        if self.is_settled() {
            self.value = self.target;
            self.velocity = 0.0;
        }

        self.value
    }

    /// Returns `true` when both position error and velocity are within settle thresholds.
    pub fn is_settled(&self) -> bool {
        (self.value - self.target).abs() <= SETTLE_POS_EPSILON
            && self.velocity.abs() <= SETTLE_VEL_EPSILON
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settle_animated(spring: Spring, steps: usize, dt: f32) -> Animated {
        let mut a = Animated::new(0.0, spring);
        a.animate_to(1.0, spring);
        for _ in 0..steps {
            a.step(dt);
        }
        a
    }

    #[test]
    fn spring_converges_and_settles_at_target() {
        let mut spring = SpringValue::new(0.0, 300.0, 26.0, 1.0);
        assert!(spring.is_settled());

        spring.set_target(120.0);
        assert!(!spring.is_settled());

        for _ in 0..180 {
            spring.tick(1.0 / 60.0);
        }

        assert!(spring.is_settled(), "expected settled, got {:?}", spring);
        assert!((spring.value - 120.0).abs() < 1e-4);
        assert_eq!(spring.velocity, 0.0);
    }

    #[test]
    fn spring_snap_to_resets_velocity_and_settles() {
        let mut spring = SpringValue::snappy(0.0);
        spring.set_target(200.0);
        spring.tick(1.0 / 60.0);
        assert!(!spring.is_settled());

        spring.snap_to(50.0);
        assert!(spring.is_settled());
        assert_eq!(spring.value, 50.0);
        assert_eq!(spring.target, 50.0);
        assert_eq!(spring.velocity, 0.0);
    }

    #[test]
    fn spring_handles_zero_and_large_dt_safely() {
        let mut spring = SpringValue::new(10.0, 250.0, 22.0, 1.0);
        spring.set_target(100.0);

        let before = spring.tick(0.0);
        assert_eq!(before, 10.0);

        let after_stall = spring.tick(5.0);
        assert!(after_stall.is_finite());
        assert!(after_stall > 10.0 && after_stall < 150.0);
    }

    #[test]
    fn spring_is_frame_rate_independent() {
        let at_60 = settle_animated(Spring::BOUNCY, 18, 1.0 / 60.0).value;
        let at_120 = settle_animated(Spring::BOUNCY, 36, 1.0 / 120.0).value;
        assert!((at_60 - at_120).abs() < 1e-4, "{at_60} vs {at_120}");

        let mut sv60 = SpringValue::from_spring(0.0, Spring::BOUNCY);
        sv60.set_target(100.0);
        for _ in 0..18 {
            sv60.tick(1.0 / 60.0);
        }
        let mut sv120 = SpringValue::from_spring(0.0, Spring::BOUNCY);
        sv120.set_target(100.0);
        for _ in 0..36 {
            sv120.tick(1.0 / 120.0);
        }
        assert!((sv60.value - sv120.value).abs() < 1e-3, "{} vs {}", sv60.value, sv120.value);
    }

    #[test]
    fn smooth_emerge_and_recede_never_overshoot() {
        for spring in [Spring::SMOOTH, Spring::EMERGE, Spring::RECEDE, Spring::KNOB_SETTLE] {
            let mut a = Animated::new(0.0, spring);
            a.launch_to(1.0, spring, 1.0);
            let peak = (0..600)
                .map(|_| {
                    a.step(1.0 / 120.0);
                    a.value
                })
                .fold(0.0_f32, f32::max);
            assert!(peak <= 1.0 + 1e-4, "{spring:?} peaked at {peak}");
        }
    }

    #[test]
    fn emerge_and_recede_exponential_curve_and_retarget() {
        let mut a = Animated::new(0.0, Spring::EMERGE);
        a.launch_to(1.0, Spring::EMERGE, 1.0);
        let w0 = Spring::EMERGE.angular_frequency();
        for frame in 1..=30 {
            a.step(1.0 / 120.0);
            let expected = 1.0 - (-w0 * frame as f32 / 120.0).exp();
            assert!((a.value - expected).abs() < 1e-4, "frame {frame}: {} vs {expected}", a.value);
        }

        // Mid-flight reverse with RECEDE flips velocity immediately.
        let mut sv = SpringValue::emerge(0.0);
        sv.launch_to(100.0, Spring::EMERGE, 1.0);
        sv.tick(1.0 / 60.0);
        assert!(sv.value > 15.0 && sv.velocity > 0.0);
        sv.launch_to(0.0, Spring::RECEDE, 1.0);
        assert!(sv.velocity < 0.0);
    }

    #[test]
    fn bouncy_spring_overshoots_and_settles() {
        let mut a = Animated::new(0.0, Spring::BOUNCY);
        a.animate_to(1.0, Spring::BOUNCY);
        let peak = (0..120)
            .map(|_| {
                a.step(1.0 / 120.0);
                a.value
            })
            .fold(0.0_f32, f32::max);
        assert!(peak > 1.02, "peak {peak}");
        assert!(settle_animated(Spring::SNAPPY, 120, 1.0 / 120.0).is_at_rest());
    }
}
