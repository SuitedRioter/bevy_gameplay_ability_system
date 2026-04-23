//! Gameplay effect definitions.
//!
//! This module defines the structure of gameplay effects and their properties.

use super::components::ModifierOperation;
use bevy::prelude::*;
use bevy_gameplay_tag::{
    GameplayTagContainer, GameplayTagRequirements, GameplayTagsManager, gameplay_tag::GameplayTag,
};
use string_cache::DefaultAtom as Atom;
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

/// Attribute calculation type.
///
/// Defines which value to use when capturing an attribute for magnitude calculation.
/// Matches UE GAS's `EAttributeBasedFloatCalculationType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AttributeCalculationType {
    /// Use the final evaluated magnitude (current_value).
    AttributeMagnitude,
    /// Use the base value only.
    AttributeBaseValue,
    /// Use the bonus magnitude: (current_value - base_value).
    AttributeBonusMagnitude,
}

/// Attribute capture source.
///
/// Defines whether to capture the attribute from the source (instigator) or target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AttributeCaptureSource {
    /// Capture from the source entity (instigator).
    Source,
    /// Capture from the target entity.
    Target,
}

/// Magnitude calculation type.
///
/// Defines how the magnitude of a modifier is calculated.
/// Follows UE GAS's magnitude calculation system.
#[derive(Debug, Clone, PartialEq)]
pub enum MagnitudeCalculation {
    /// A fixed scalar value (optionally scaled by level).
    ///
    /// Formula: `base_value * level_multiplier^(level - 1)`
    ScalableFloat {
        base_value: f32,
        /// Multiplier applied per level (1.0 = no scaling).
        level_multiplier: f32,
    },

    /// Calculate from an attribute on the source or target entity.
    ///
    /// Formula: `(coefficient * (pre_multiply_additive + [attribute_value])) + post_multiply_additive`
    ///
    /// This allows you to scale damage based on the caster's stats, for example.
    AttributeBased {
        /// Name of the attribute to read.
        attribute_name: Atom,
        /// Which entity to capture from (Source or Target).
        capture_source: AttributeCaptureSource,
        /// Which value to use from the attribute.
        calculation_type: AttributeCalculationType,
        /// Coefficient to multiply the attribute value by.
        coefficient: f32,
        /// Value added before multiplication.
        pre_multiply_additive: f32,
        /// Value added after multiplication.
        post_multiply_additive: f32,
    },

    /// Custom calculation using a registered calculator.
    ///
    /// The calculator is looked up by name from a registry.
    /// This allows complex calculations that capture multiple attributes.
    CustomClass {
        /// Name of the custom calculator to use.
        calculator_name: Atom,
    },

    /// Magnitude set at runtime by the caller.
    ///
    /// The caller must provide a value for this tag when applying the effect.
    /// If not provided, defaults to 0.0.
    SetByCaller {
        /// Tag identifying this magnitude value.
        data_tag: GameplayTag,
    },
}

impl MagnitudeCalculation {
    /// Creates a simple scalar magnitude.
    pub fn scalar(value: f32) -> Self {
        Self::ScalableFloat {
            base_value: value,
            level_multiplier: 1.0,
        }
    }

    /// Creates a level-scaled magnitude.
    ///
    /// # Example
    /// ```ignore
    /// // Damage that scales: 10 at level 1, 20 at level 2, 40 at level 3
    /// MagnitudeCalculation::scaled(10.0, 2.0)
    /// ```
    pub fn scaled(base_value: f32, level_multiplier: f32) -> Self {
        Self::ScalableFloat {
            base_value,
            level_multiplier,
        }
    }

    /// Creates an attribute-based magnitude from the source entity.
    ///
    /// Uses the current value (AttributeMagnitude) by default.
    pub fn from_source_attribute(attribute_name: impl Into<Atom>, coefficient: f32) -> Self {
        Self::AttributeBased {
            attribute_name: attribute_name.into(),
            capture_source: AttributeCaptureSource::Source,
            calculation_type: AttributeCalculationType::AttributeMagnitude,
            coefficient,
            pre_multiply_additive: 0.0,
            post_multiply_additive: 0.0,
        }
    }

    /// Creates an attribute-based magnitude from the target entity.
    ///
    /// Uses the current value (AttributeMagnitude) by default.
    pub fn from_target_attribute(attribute_name: impl Into<Atom>, coefficient: f32) -> Self {
        Self::AttributeBased {
            attribute_name: attribute_name.into(),
            capture_source: AttributeCaptureSource::Target,
            calculation_type: AttributeCalculationType::AttributeMagnitude,
            coefficient,
            pre_multiply_additive: 0.0,
            post_multiply_additive: 0.0,
        }
    }

    /// Builder method to set the calculation type for AttributeBased.
    pub fn with_calculation_type(mut self, calc_type: AttributeCalculationType) -> Self {
        if let Self::AttributeBased {
            calculation_type, ..
        } = &mut self
        {
            *calculation_type = calc_type;
        }
        self
    }

    /// Builder method to set pre-multiply additive for AttributeBased.
    pub fn with_pre_multiply_add(mut self, value: f32) -> Self {
        if let Self::AttributeBased {
            pre_multiply_additive,
            ..
        } = &mut self
        {
            *pre_multiply_additive = value;
        }
        self
    }

    /// Builder method to set post-multiply additive for AttributeBased.
    pub fn with_post_multiply_add(mut self, value: f32) -> Self {
        if let Self::AttributeBased {
            post_multiply_additive,
            ..
        } = &mut self
        {
            *post_multiply_additive = value;
        }
        self
    }

    /// Creates a SetByCaller magnitude.
    pub fn set_by_caller(data_tag: GameplayTag) -> Self {
        Self::SetByCaller { data_tag }
    }

    /// Creates a custom calculation magnitude.
    pub fn custom(calculator_name: impl Into<Atom>) -> Self {
        Self::CustomClass {
            calculator_name: calculator_name.into(),
        }
    }

    /// Evaluates the magnitude given a level and optional source value.
    ///
    /// For AttributeBased calculations, pass the captured attribute value as `source_value`.
    /// For SetByCaller, pass the caller-provided value.
    pub fn evaluate(&self, level: i32, source_value: Option<f32>) -> f32 {
        match self {
            MagnitudeCalculation::ScalableFloat {
                base_value,
                level_multiplier,
            } => {
                if *level_multiplier == 1.0 {
                    *base_value
                } else {
                    base_value * level_multiplier.powi(level - 1)
                }
            }
            MagnitudeCalculation::AttributeBased {
                coefficient,
                pre_multiply_additive,
                post_multiply_additive,
                ..
            } => {
                let source = source_value.unwrap_or(0.0);
                (source + pre_multiply_additive) * coefficient + post_multiply_additive
            }
            MagnitudeCalculation::SetByCaller { .. } => {
                // Caller must provide the value
                source_value.unwrap_or(0.0)
            }
            MagnitudeCalculation::CustomClass { .. } => {
                // Custom calculators are looked up from a registry
                // For now, return 0.0 as placeholder
                warn!("Custom calculation not yet implemented");
                0.0
            }
        }
    }
}

/// Information about a modifier in an effect.
#[derive(Debug, Clone, PartialEq)]
pub struct ModifierInfo {
    /// The name of the attribute to modify.
    pub attribute_name: Atom,
    /// The operation to perform.
    pub operation: ModifierOperation,
    /// How to calculate the magnitude.
    pub magnitude: MagnitudeCalculation,
}

impl ModifierInfo {
    /// Creates a new modifier info.
    pub fn new(
        attribute_name: impl Into<Atom>,
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
    pub id: Atom,
    /// Duration policy.
    pub duration_policy: DurationPolicy,
    /// Duration in seconds (if HasDuration).
    pub duration_magnitude: f32,
    /// Period for periodic effects (0.0 = not periodic).
    pub period: f32,
    /// Modifiers applied by this effect.
    pub modifiers: Vec<ModifierInfo>,
    /// Tags granted while this effect is active.
    pub granted_tags: GameplayTagContainer,
    /// Tags that identify this effect (for immunity checks).
    /// If a target has any of these tags in their immunity_tags, the effect is rejected.
    pub asset_tags: GameplayTagContainer,
    /// Tags that grant immunity to effects.
    /// If this effect has any of these tags, targets with matching immunity_tags will reject it.
    pub immunity_tags: GameplayTagContainer,
    /// Tag requirements for applying this effect.
    pub application_tag_requirements: GameplayTagRequirements,
    /// Stacking policy.
    pub stacking_policy: StackingPolicy,
}

impl GameplayEffectDefinition {
    /// Creates a new gameplay effect definition.
    pub fn new(id: impl Into<Atom>) -> Self {
        Self {
            id: id.into(),
            duration_policy: DurationPolicy::Instant,
            duration_magnitude: 0.0,
            period: 0.0,
            modifiers: Vec::new(),
            granted_tags: GameplayTagContainer::default(),
            asset_tags: GameplayTagContainer::default(),
            immunity_tags: GameplayTagContainer::default(),
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
    pub fn grant_tag(mut self, tag: GameplayTag, tags_manager: &Res<GameplayTagsManager>) -> Self {
        self.granted_tags.add_tag(tag, tags_manager);
        self
    }

    /// Adds an asset tag (for immunity checks).
    pub fn with_asset_tag(
        mut self,
        tag: GameplayTag,
        tags_manager: &Res<GameplayTagsManager>,
    ) -> Self {
        self.asset_tags.add_tag(tag, tags_manager);
        self
    }

    /// Adds an immunity tag.
    ///
    /// Effects with these tags can be blocked by targets that have matching immunity.
    pub fn with_immunity_tag(
        mut self,
        tag: GameplayTag,
        tags_manager: &Res<GameplayTagsManager>,
    ) -> Self {
        self.immunity_tags.add_tag(tag, tags_manager);
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
    pub definitions: std::collections::HashMap<Atom, GameplayEffectDefinition>,
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
    pub fn get(&self, id: impl Into<Atom>) -> Option<&GameplayEffectDefinition> {
        self.definitions.get(&id.into())
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
        let mag = MagnitudeCalculation::from_source_attribute("Strength", 2.0);
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

        assert_eq!(effect.id, Atom::from("test_effect"));
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
