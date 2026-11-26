//! Damped harmonic spring animation primitive for 60/120Hz native UI motion.

/// Position threshold below which a spring is considered settled.
const SETTLE_POS_EPSILON: f32 = 0.05;
/// Velocity threshold below which a spring is considered settled.
const SETTLE_VEL_EPSILON: f32 = 0.05;
/// Maximum single integration step in seconds to keep semi-implicit Euler stable.
const MAX_SUBSTEP_DT: f32 = 1.0 / 120.0;
/// Upper clamp for frame delta-time to avoid huge jumps after thread stalls.
const MAX_FRAME_DT: f32 = 0.05;

/// A 1D damped harmonic oscillator (`F = -k(x - target) - c*v`).
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

    /// Snappy island resize preset (stiffness 320, damping 28, mass 1).
    pub fn snappy(initial: f32) -> Self {
        Self::new(initial, 320.0, 28.0, 1.0)
    }

    /// Gentle fade/opacity preset (stiffness 180, damping 24, mass 1).
    pub fn gentle(initial: f32) -> Self {
        Self::new(initial, 180.0, 24.0, 1.0)
    }

    /// Sets a new animation target without resetting current velocity.
    pub fn set_target(&mut self, target: f32) {
        self.target = target;
    }

    /// Immediately snaps value and target to `value` with zero velocity.
    pub fn snap_to(&mut self, value: f32) {
        self.value = value;
        self.target = value;
        self.velocity = 0.0;
    }

    /// Advances the spring simulation by `dt` seconds and returns the updated value.
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
        let steps = (clamped_dt / MAX_SUBSTEP_DT).ceil().max(1.0) as u32;
        let sub_dt = clamped_dt / steps as f32;

        for _ in 0..steps {
            let displacement = self.value - self.target;
            let force = -self.stiffness * displacement - self.damping * self.velocity;
            let acceleration = force / self.mass;
            self.velocity += acceleration * sub_dt;
            self.value += self.velocity * sub_dt;
        }

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
}
