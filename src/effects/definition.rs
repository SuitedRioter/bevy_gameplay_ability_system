//! Gameplay effect definitions.
//!
//! This module defines the structure of gameplay effects and their properties.

use super::components::ModifierOperation;
use bevy::prelude::*;
use bevy_gameplay_tag::{GameplayTagRequirements, gameplay_tag::GameplayTag};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DurationPolicy {
    /// Effect applies instantly and is removed immediately.
    Instant,
    /// Effect has a limited duration.
    HasDuration,
    /// Effect lasts forever until explicitly removed.
    Infinite,
}

/// Stacking policy for gameplay effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StackingPolicy {
    /// Each application is independent.
    Independent,
    /// Refresh the duration on reapplication.
    RefreshDuration,
    /// Increment stack count up to a maximum.
    StackCount { max_stacks: i32 },
}

/// Magnitude calculation type.
#[derive(Debug, Clone, PartialEq)]
pub enum MagnitudeCalculation {
    /// A fixed scalar value.
    ScalableFloat { base_value: f32 },
    /// Calculate from an attribute on the source.
    AttributeBased {
        attribute_name: String,
        coefficient: f32,
        pre_multiply_additive: f32,
        post_multiply_additive: f32,
    },
    /// Custom calculation (placeholder for future extension).
    Custom,
}

impl MagnitudeCalculation {
    /// Creates a simple scalar magnitude.
    pub fn scalar(value: f32) -> Self {
        Self::ScalableFloat { base_value: value }
    }

    /// Creates an attribute-based magnitude.
    pub fn from_attribute(attribute_name: impl Into<String>, coefficient: f32) -> Self {
        Self::AttributeBased {
            attribute_name: attribute_name.into(),
            coefficient,
            pre_multiply_additive: 0.0,
            post_multiply_additive: 0.0,
        }
    }

    /// Evaluates the magnitude given a level and optional source entity.
    ///
    /// For attribute-based calculations, you'll need to query the source entity's attributes.
    pub fn evaluate(&self, _level: i32, _source_value: Option<f32>) -> f32 {
        match self {
            MagnitudeCalculation::ScalableFloat { base_value } => *base_value,
            MagnitudeCalculation::AttributeBased {
                coefficient,
                pre_multiply_additive,
                post_multiply_additive,
                ..
            } => {
                let source = _source_value.unwrap_or(0.0);
                (source + pre_multiply_additive) * coefficient + post_multiply_additive
            }
            MagnitudeCalculation::Custom => 0.0,
        }
    }
}

/// Information about a modifier in an effect.
#[derive(Debug, Clone, PartialEq)]
pub struct ModifierInfo {
    /// The name of the attribute to modify.
    pub attribute_name: String,
    /// The operation to perform.
    pub operation: ModifierOperation,
    /// How to calculate the magnitude.
    pub magnitude: MagnitudeCalculation,
}

impl ModifierInfo {
    /// Creates a new modifier info.
    pub fn new(
        attribute_name: impl Into<String>,
        operation: ModifierOperation,
        magnitude: MagnitudeCalculation,
    ) -> Self {
        Self {
            attribute_name: attribute_name.into(),
            operation,
            magnitude,
        }
    }
}

/// Definition of a gameplay effect.
///
/// This is the template for creating active effect instances.
/// Store these in a resource or asset system.
#[derive(Debug, Clone, PartialEq)]
pub struct GameplayEffectDefinition {
    /// Unique identifier for this effect.
    pub id: String,
    /// Duration policy.
    pub duration_policy: DurationPolicy,
    /// Duration in seconds (if HasDuration).
    pub duration_magnitude: f32,
    /// Period for periodic effects (0.0 = not periodic).
    pub period: f32,
    /// Modifiers applied by this effect.
    pub modifiers: Vec<ModifierInfo>,
    /// Tags granted while this effect is active.
    pub granted_tags: Vec<GameplayTag>,
    /// Tag requirements for applying this effect.
    pub application_tag_requirements: GameplayTagRequirements,
    /// Stacking policy.
    pub stacking_policy: StackingPolicy,
}

impl GameplayEffectDefinition {
    /// Creates a new gameplay effect definition.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            duration_policy: DurationPolicy::Instant,
            duration_magnitude: 0.0,
            period: 0.0,
            modifiers: Vec::new(),
            granted_tags: Vec::new(),
            application_tag_requirements: GameplayTagRequirements::default(),
            stacking_policy: StackingPolicy::Independent,
        }
    }

    /// Sets the duration policy.
    pub fn with_duration_policy(mut self, policy: DurationPolicy) -> Self {
        self.duration_policy = policy;
        self
    }

    /// Sets the duration magnitude.
    pub fn with_duration(mut self, duration: f32) -> Self {
        self.duration_magnitude = duration;
        self.duration_policy = DurationPolicy::HasDuration;
        self
    }

    /// Sets the period for periodic effects.
    pub fn with_period(mut self, period: f32) -> Self {
        self.period = period;
        self
    }

    /// Adds a modifier to this effect.
    pub fn add_modifier(mut self, modifier: ModifierInfo) -> Self {
        self.modifiers.push(modifier);
        self
    }

    /// Adds a granted tag.
    pub fn grant_tag(mut self, tag: GameplayTag) -> Self {
        self.granted_tags.push(tag);
        self
    }

    /// Sets the tag requirements.
    pub fn with_tag_requirements(mut self, requirements: GameplayTagRequirements) -> Self {
        self.application_tag_requirements = requirements;
        self
    }

    /// Sets the stacking policy.
    pub fn with_stacking_policy(mut self, policy: StackingPolicy) -> Self {
        self.stacking_policy = policy;
        self
    }
}

/// Resource that stores all gameplay effect definitions.
#[derive(Resource, Default)]
pub struct GameplayEffectRegistry {
    pub definitions: std::collections::HashMap<String, GameplayEffectDefinition>,
}

impl GameplayEffectRegistry {
    /// Creates a new empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers an effect definition.
    pub fn register(&mut self, definition: GameplayEffectDefinition) {
        self.definitions.insert(definition.id.clone(), definition);
    }

    /// Gets an effect definition by ID.
    pub fn get(&self, id: &str) -> Option<&GameplayEffectDefinition> {
        self.definitions.get(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magnitude_calculation_scalar() {
        let mag = MagnitudeCalculation::scalar(10.0);
        assert_eq!(mag.evaluate(1, None), 10.0);
    }

    #[test]
    fn test_magnitude_calculation_attribute_based() {
        let mag = MagnitudeCalculation::from_attribute("Strength", 2.0);
        assert_eq!(mag.evaluate(1, Some(5.0)), 10.0);
    }

    #[test]
    fn test_effect_definition_builder() {
        let effect = GameplayEffectDefinition::new("test_effect")
            .with_duration(5.0)
            .with_period(1.0)
            .add_modifier(ModifierInfo::new(
                "Health",
                ModifierOperation::AddCurrent,
                MagnitudeCalculation::scalar(10.0),
            ));

        assert_eq!(effect.id, "test_effect");
        assert_eq!(effect.duration_policy, DurationPolicy::HasDuration);
        assert_eq!(effect.duration_magnitude, 5.0);
        assert_eq!(effect.period, 1.0);
        assert_eq!(effect.modifiers.len(), 1);
    }

    #[test]
    fn test_registry() {
        let mut registry = GameplayEffectRegistry::new();
        let effect = GameplayEffectDefinition::new("test");
        registry.register(effect);

        assert!(registry.get("test").is_some());
        assert!(registry.get("nonexistent").is_none());
    }
}
