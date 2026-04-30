//! Built-in application requirements.
//!
//! This module provides common application requirement implementations.

use super::application_requirement::{ApplicationContext, ApplicationRequirement};
use string_cache::DefaultAtom as Atom;

/// Requires target's health to be above a threshold percentage.
///
/// # Example
/// ```ignore
/// // Only apply if target has at least 50% health
/// let requirement = HealthPercentThreshold::new("Health", 0.5);
/// registry.register("low_health_only", Box::new(requirement));
/// ```
pub struct HealthPercentThreshold {
    health_attribute: Atom,
    min_percent: f32,
}

impl HealthPercentThreshold {
    pub fn new(health_attribute: impl Into<Atom>, min_percent: f32) -> Self {
        Self {
            health_attribute: health_attribute.into(),
            min_percent,
        }
    }
}

impl ApplicationRequirement for HealthPercentThreshold {
    fn can_apply(&self, ctx: &ApplicationContext) -> bool {
        if let Some(health) = ctx.get_target_attribute(&self.health_attribute) {
            // Assume max health is stored in metadata or calculate from base value
            // For simplicity, we'll use a heuristic: if current < base, use base as max
            // Otherwise, current is already at max
            let max_health = if let Some(snapshot) = ctx
                .attributes
                .iter()
                .find(|s| s.owner == ctx.target && s.attribute_name == self.health_attribute)
            {
                snapshot.base_value.max(snapshot.current_value)
            } else {
                return false;
            };

            if max_health <= 0.0 {
                return false;
            }

            (health / max_health) >= self.min_percent
        } else {
            false
        }
    }
}

/// Requires target's health to be below a threshold percentage.
pub struct HealthPercentBelowThreshold {
    health_attribute: Atom,
    max_percent: f32,
}

impl HealthPercentBelowThreshold {
    pub fn new(health_attribute: impl Into<Atom>, max_percent: f32) -> Self {
        Self {
            health_attribute: health_attribute.into(),
            max_percent,
        }
    }
}

impl ApplicationRequirement for HealthPercentBelowThreshold {
    fn can_apply(&self, ctx: &ApplicationContext) -> bool {
        if let Some(health) = ctx.get_target_attribute(&self.health_attribute) {
            let max_health = if let Some(snapshot) = ctx
                .attributes
                .iter()
                .find(|s| s.owner == ctx.target && s.attribute_name == self.health_attribute)
            {
                snapshot.base_value.max(snapshot.current_value)
            } else {
                return false;
            };

            if max_health <= 0.0 {
                return false;
            }

            (health / max_health) <= self.max_percent
        } else {
            false
        }
    }
}

/// Requires an attribute to be above a threshold value.
pub struct AttributeAboveThreshold {
    attribute_name: Atom,
    threshold: f32,
}

impl AttributeAboveThreshold {
    pub fn new(attribute_name: impl Into<Atom>, threshold: f32) -> Self {
        Self {
            attribute_name: attribute_name.into(),
            threshold,
        }
    }
}

impl ApplicationRequirement for AttributeAboveThreshold {
    fn can_apply(&self, ctx: &ApplicationContext) -> bool {
        ctx.get_target_attribute(&self.attribute_name)
            .map(|value| value > self.threshold)
            .unwrap_or(false)
    }
}

/// Requires an attribute to be below a threshold value.
pub struct AttributeBelowThreshold {
    attribute_name: Atom,
    threshold: f32,
}

impl AttributeBelowThreshold {
    pub fn new(attribute_name: impl Into<Atom>, threshold: f32) -> Self {
        Self {
            attribute_name: attribute_name.into(),
            threshold,
        }
    }
}

impl ApplicationRequirement for AttributeBelowThreshold {
    fn can_apply(&self, ctx: &ApplicationContext) -> bool {
        ctx.get_target_attribute(&self.attribute_name)
            .map(|value| value < self.threshold)
            .unwrap_or(false)
    }
}

/// Requires source's attribute to be above target's attribute.
///
/// Useful for "only if attacker is stronger" type conditions.
pub struct SourceAttributeGreaterThanTarget {
    source_attribute: Atom,
    target_attribute: Atom,
}

impl SourceAttributeGreaterThanTarget {
    pub fn new(
        source_attribute: impl Into<Atom>,
        target_attribute: impl Into<Atom>,
    ) -> Self {
        Self {
            source_attribute: source_attribute.into(),
            target_attribute: target_attribute.into(),
        }
    }
}

impl ApplicationRequirement for SourceAttributeGreaterThanTarget {
    fn can_apply(&self, ctx: &ApplicationContext) -> bool {
        if let (Some(source_value), Some(target_value)) = (
            ctx.get_source_attribute(&self.source_attribute),
            ctx.get_target_attribute(&self.target_attribute),
        ) {
            source_value > target_value
        } else {
            false
        }
    }
}

/// Requires effect level to be within a range.
pub struct LevelRangeRequirement {
    min_level: i32,
    max_level: i32,
}

impl LevelRangeRequirement {
    pub fn new(min_level: i32, max_level: i32) -> Self {
        Self {
            min_level,
            max_level,
        }
    }
}

impl ApplicationRequirement for LevelRangeRequirement {
    fn can_apply(&self, ctx: &ApplicationContext) -> bool {
        ctx.level >= self.min_level && ctx.level <= self.max_level
    }
}
