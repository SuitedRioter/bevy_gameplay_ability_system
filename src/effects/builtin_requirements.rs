//! Built-in application requirements.
//!
//! This module provides common application requirement implementations.

use super::application_requirement::{ApplicationContext, ApplicationRequirement};
use string_cache::DefaultAtom as Atom;

/// Requires target's attribute to be above a percentage of another attribute.
///
/// Useful for health percentage checks where you have separate Health and MaxHealth attributes.
///
/// # Example
/// ```ignore
/// // Only apply if Health >= 50% of MaxHealth
/// let requirement = AttributePercentAbove::new("Health", "MaxHealth", 0.5);
/// registry.register("high_health_only", Box::new(requirement));
/// ```
pub struct AttributePercentAbove {
    current_attribute: Atom,
    max_attribute: Atom,
    min_percent: f32,
}

impl AttributePercentAbove {
    pub fn new(
        current_attribute: impl Into<Atom>,
        max_attribute: impl Into<Atom>,
        min_percent: f32,
    ) -> Self {
        Self {
            current_attribute: current_attribute.into(),
            max_attribute: max_attribute.into(),
            min_percent,
        }
    }
}

impl ApplicationRequirement for AttributePercentAbove {
    fn can_apply(&self, ctx: &ApplicationContext) -> bool {
        if let (Some(current), Some(max)) = (
            ctx.get_target_attribute(&self.current_attribute),
            ctx.get_target_attribute(&self.max_attribute),
        ) {
            if max <= 0.0 {
                return false;
            }
            (current / max) >= self.min_percent
        } else {
            false
        }
    }
}

/// Requires target's attribute to be below a percentage of another attribute.
///
/// # Example
/// ```ignore
/// // Only apply if Health < 50% of MaxHealth
/// let requirement = AttributePercentBelow::new("Health", "MaxHealth", 0.5);
/// registry.register("low_health_only", Box::new(requirement));
/// ```
pub struct AttributePercentBelow {
    current_attribute: Atom,
    max_attribute: Atom,
    max_percent: f32,
}

impl AttributePercentBelow {
    pub fn new(
        current_attribute: impl Into<Atom>,
        max_attribute: impl Into<Atom>,
        max_percent: f32,
    ) -> Self {
        Self {
            current_attribute: current_attribute.into(),
            max_attribute: max_attribute.into(),
            max_percent,
        }
    }
}

impl ApplicationRequirement for AttributePercentBelow {
    fn can_apply(&self, ctx: &ApplicationContext) -> bool {
        if let (Some(current), Some(max)) = (
            ctx.get_target_attribute(&self.current_attribute),
            ctx.get_target_attribute(&self.max_attribute),
        ) {
            if max <= 0.0 {
                return false;
            }
            (current / max) <= self.max_percent
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

/// Requires target to have all specified tags.
pub struct RequireAllTags {
    required_tags: Vec<bevy_gameplay_tag::gameplay_tag::GameplayTag>,
}

impl RequireAllTags {
    pub fn new(tags: Vec<bevy_gameplay_tag::gameplay_tag::GameplayTag>) -> Self {
        Self {
            required_tags: tags,
        }
    }
}

impl ApplicationRequirement for RequireAllTags {
    fn can_apply(&self, ctx: &ApplicationContext) -> bool {
        if let Some(target_tags) = ctx.target_tags {
            self.required_tags
                .iter()
                .all(|tag| target_tags.0.explicit_tags.has_tag_exact(tag))
        } else {
            false
        }
    }
}

/// Requires target to have any of the specified tags.
pub struct RequireAnyTag {
    required_tags: Vec<bevy_gameplay_tag::gameplay_tag::GameplayTag>,
}

impl RequireAnyTag {
    pub fn new(tags: Vec<bevy_gameplay_tag::gameplay_tag::GameplayTag>) -> Self {
        Self {
            required_tags: tags,
        }
    }
}

impl ApplicationRequirement for RequireAnyTag {
    fn can_apply(&self, ctx: &ApplicationContext) -> bool {
        if let Some(target_tags) = ctx.target_tags {
            self.required_tags
                .iter()
                .any(|tag| target_tags.0.explicit_tags.has_tag_exact(tag))
        } else {
            false
        }
    }
}

/// Requires target to NOT have any of the specified tags.
pub struct BlockIfHasTag {
    blocked_tags: Vec<bevy_gameplay_tag::gameplay_tag::GameplayTag>,
}

impl BlockIfHasTag {
    pub fn new(tags: Vec<bevy_gameplay_tag::gameplay_tag::GameplayTag>) -> Self {
        Self { blocked_tags: tags }
    }
}

impl ApplicationRequirement for BlockIfHasTag {
    fn can_apply(&self, ctx: &ApplicationContext) -> bool {
        if let Some(target_tags) = ctx.target_tags {
            !self
                .blocked_tags
                .iter()
                .any(|tag| target_tags.0.explicit_tags.has_tag_exact(tag))
        } else {
            true
        }
    }
}

/// Combines multiple requirements with AND logic.
pub struct AndRequirement {
    requirements: Vec<Box<dyn ApplicationRequirement>>,
}

impl AndRequirement {
    pub fn new(requirements: Vec<Box<dyn ApplicationRequirement>>) -> Self {
        Self { requirements }
    }
}

impl ApplicationRequirement for AndRequirement {
    fn can_apply(&self, ctx: &ApplicationContext) -> bool {
        self.requirements.iter().all(|req| req.can_apply(ctx))
    }
}

/// Combines multiple requirements with OR logic.
pub struct OrRequirement {
    requirements: Vec<Box<dyn ApplicationRequirement>>,
}

impl OrRequirement {
    pub fn new(requirements: Vec<Box<dyn ApplicationRequirement>>) -> Self {
        Self { requirements }
    }
}

impl ApplicationRequirement for OrRequirement {
    fn can_apply(&self, ctx: &ApplicationContext) -> bool {
        self.requirements.iter().any(|req| req.can_apply(ctx))
    }
}

/// Inverts another requirement (NOT logic).
pub struct NotRequirement {
    requirement: Box<dyn ApplicationRequirement>,
}

impl NotRequirement {
    pub fn new(requirement: Box<dyn ApplicationRequirement>) -> Self {
        Self { requirement }
    }
}

impl ApplicationRequirement for NotRequirement {
    fn can_apply(&self, ctx: &ApplicationContext) -> bool {
        !self.requirement.can_apply(ctx)
    }
}
