//! Math utilities for gameplay calculations.
//!
//! This module provides mathematical utilities commonly used in gameplay systems,
//! such as interpolation, clamping, and curve evaluation.

/// Clamps a value between a minimum and maximum.
///
/// If min or max is None, that bound is not enforced.
pub fn clamp_optional(value: f32, min: Option<f32>, max: Option<f32>) -> f32 {
    let mut result = value;

    if let Some(min_val) = min {
        result = result.max(min_val);
    }

    if let Some(max_val) = max {
        result = result.min(max_val);
    }

    result
}

/// Linear interpolation between two values.
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Inverse linear interpolation - returns the t value for a given interpolated value.
pub fn inverse_lerp(a: f32, b: f32, value: f32) -> f32 {
    if (b - a).abs() < f32::EPSILON {
        0.0
    } else {
        (value - a) / (b - a)
    }
}

/// Normalizes a value from one range to another.
pub fn remap(value: f32, from_min: f32, from_max: f32, to_min: f32, to_max: f32) -> f32 {
    let t = inverse_lerp(from_min, from_max, value);
    lerp(to_min, to_max, t)
}

/// Smoothstep interpolation (smooth ease-in/ease-out).
pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Calculates a percentage value (0-100) from a current and max value.
pub fn percentage(current: f32, max: f32) -> f32 {
    if max <= 0.0 {
        0.0
    } else {
        (current / max * 100.0).clamp(0.0, 100.0)
    }
}

/// Calculates a normalized value (0-1) from a current and max value.
pub fn normalize(current: f32, max: f32) -> f32 {
    if max <= 0.0 {
        0.0
    } else {
        (current / max).clamp(0.0, 1.0)
    }
}

/// Applies a scalar multiplier with proper handling of additive vs multiplicative.
///
/// For additive: result = base + (base * multiplier)
/// For multiplicative: result = base * multiplier
pub fn apply_multiplier(base: f32, multiplier: f32, is_additive: bool) -> f32 {
    if is_additive {
        base + (base * multiplier)
    } else {
        base * multiplier
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clamp_optional() {
        assert_eq!(clamp_optional(5.0, Some(0.0), Some(10.0)), 5.0);
        assert_eq!(clamp_optional(-5.0, Some(0.0), Some(10.0)), 0.0);
        assert_eq!(clamp_optional(15.0, Some(0.0), Some(10.0)), 10.0);
        assert_eq!(clamp_optional(5.0, None, Some(10.0)), 5.0);
        assert_eq!(clamp_optional(5.0, Some(0.0), None), 5.0);
        assert_eq!(clamp_optional(5.0, None, None), 5.0);
    }

    #[test]
    fn test_lerp() {
        assert_eq!(lerp(0.0, 10.0, 0.0), 0.0);
        assert_eq!(lerp(0.0, 10.0, 0.5), 5.0);
        assert_eq!(lerp(0.0, 10.0, 1.0), 10.0);
    }

    #[test]
    fn test_inverse_lerp() {
        assert_eq!(inverse_lerp(0.0, 10.0, 0.0), 0.0);
        assert_eq!(inverse_lerp(0.0, 10.0, 5.0), 0.5);
        assert_eq!(inverse_lerp(0.0, 10.0, 10.0), 1.0);
    }

    #[test]
    fn test_remap() {
        assert_eq!(remap(5.0, 0.0, 10.0, 0.0, 100.0), 50.0);
        assert_eq!(remap(0.0, 0.0, 10.0, 0.0, 100.0), 0.0);
        assert_eq!(remap(10.0, 0.0, 10.0, 0.0, 100.0), 100.0);
    }

    #[test]
    fn test_percentage() {
        assert_eq!(percentage(50.0, 100.0), 50.0);
        assert_eq!(percentage(100.0, 100.0), 100.0);
        assert_eq!(percentage(0.0, 100.0), 0.0);
        assert_eq!(percentage(150.0, 100.0), 100.0); // Clamped
    }

    #[test]
    fn test_normalize() {
        assert_eq!(normalize(50.0, 100.0), 0.5);
        assert_eq!(normalize(100.0, 100.0), 1.0);
        assert_eq!(normalize(0.0, 100.0), 0.0);
    }

    #[test]
    fn test_apply_multiplier() {
        // Additive: 100 + (100 * 0.5) = 150
        assert_eq!(apply_multiplier(100.0, 0.5, true), 150.0);

        // Multiplicative: 100 * 1.5 = 150
        assert_eq!(apply_multiplier(100.0, 1.5, false), 150.0);
    }
}
