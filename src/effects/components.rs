//! Effect system components.
//!
//! This module defines the core components for the gameplay effect system.

use bevy::prelude::*;
use bevy_gameplay_tag::GameplayTagContainer;

/// Active gameplay effect instance component.
///
/// Each active effect is a separate entity with this component.
/// The effect modifies attributes on the target entity.
#[derive(Component, Debug, Clone)]
pub struct ActiveGameplayEffect {
    /// The ID of the effect definition.
    pub definition_id: String,
    /// The level at which this effect was applied.
    pub level: i32,
    /// The time when this effect was applied (in seconds).
    pub start_time: f32,
    /// The current stack count for this effect.
    pub stack_count: i32,
}

impl ActiveGameplayEffect {
    /// Creates a new active gameplay effect.
    pub fn new(definition_id: String, level: i32, start_time: f32) -> Self {
        Self {
            definition_id,
            level,
            start_time,
            stack_count: 1,
        }
    }
}

/// Component that links an effect to its target entity.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectTarget(pub Entity);

/// Component that identifies the instigator of an effect.
///
/// This is the entity that caused the effect to be applied.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectInstigator(pub Option<Entity>);

/// Component for effects with a duration.
///
/// This tracks the remaining time for duration-based effects.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct EffectDuration {
    /// Remaining time in seconds.
    pub remaining: f32,
    /// Total duration in seconds.
    pub total: f32,
}

impl EffectDuration {
    /// Creates a new effect duration.
    pub fn new(duration: f32) -> Self {
        Self {
            remaining: duration,
            total: duration,
        }
    }

    /// Returns true if the effect has expired.
    pub fn is_expired(&self) -> bool {
        self.remaining <= 0.0
    }

    /// Updates the remaining time.
    pub fn tick(&mut self, delta: f32) {
        self.remaining -= delta;
    }
}

/// Component for periodic effects.
///
/// Periodic effects execute their modifiers at regular intervals.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct PeriodicEffect {
    /// The period between executions in seconds.
    pub period: f32,
    /// Time until the next execution in seconds.
    pub time_until_next: f32,
}

impl PeriodicEffect {
    /// Creates a new periodic effect.
    pub fn new(period: f32) -> Self {
        Self {
            period,
            time_until_next: period,
        }
    }

    /// Returns true if the effect should execute this frame.
    pub fn should_execute(&self) -> bool {
        self.time_until_next <= 0.0
    }

    /// Updates the timer and returns true if execution should happen.
    pub fn tick(&mut self, delta: f32) -> bool {
        self.time_until_next -= delta;
        if self.should_execute() {
            self.time_until_next += self.period;
            true
        } else {
            false
        }
    }
}

/// Attribute modifier component.
///
/// This represents a single modifier applied to an attribute by an effect.
/// Each modifier is a separate entity linked to both the effect and the target attribute.
#[derive(Component, Debug, Clone, PartialEq)]
pub struct AttributeModifier {
    /// The entity that owns the attribute being modified.
    pub target_entity: Entity,
    /// The name of the attribute being modified.
    pub target_attribute: String,
    /// The operation to perform.
    pub operation: ModifierOperation,
    /// The magnitude of the modification.
    pub magnitude: f32,
}

/// The source of a modifier (which effect created it).
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModifierSource(pub Entity);

/// The type of modification operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModifierOperation {
    /// Add to the base value (permanent).
    AddBase,
    /// Add to the current value (temporary).
    AddCurrent,
    /// Multiply the current value.
    MultiplyAdditive,
    /// Multiply the current value (multiplicative stacking).
    MultiplyMultiplicative,
    /// Override the current value.
    Override,
}

impl ModifierOperation {
    /// Returns the priority order for applying modifiers.
    ///
    /// Lower values are applied first.
    pub fn priority(&self) -> i32 {
        match self {
            ModifierOperation::AddBase => 0,
            ModifierOperation::AddCurrent => 1,
            ModifierOperation::MultiplyAdditive => 2,
            ModifierOperation::MultiplyMultiplicative => 3,
            ModifierOperation::Override => 4,
        }
    }
}

/// Tags granted by an active effect.
///
/// These tags are added to the target entity while the effect is active.
#[derive(Component, Debug, Clone)]
pub struct EffectGrantedTags {
    pub tags: GameplayTagContainer,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effect_duration() {
        let mut duration = EffectDuration::new(5.0);
        assert_eq!(duration.remaining, 5.0);
        assert!(!duration.is_expired());

        duration.tick(3.0);
        assert_eq!(duration.remaining, 2.0);
        assert!(!duration.is_expired());

        duration.tick(3.0);
        assert_eq!(duration.remaining, -1.0);
        assert!(duration.is_expired());
    }

    #[test]
    fn test_periodic_effect() {
        let mut periodic = PeriodicEffect::new(1.0);
        assert_eq!(periodic.time_until_next, 1.0);
        assert!(!periodic.should_execute());

        assert!(!periodic.tick(0.5));
        assert_eq!(periodic.time_until_next, 0.5);

        assert!(periodic.tick(0.6));
        assert_eq!(periodic.time_until_next, 0.9);
    }

    #[test]
    fn test_modifier_operation_priority() {
        assert!(ModifierOperation::AddBase.priority() < ModifierOperation::AddCurrent.priority());
        assert!(
            ModifierOperation::AddCurrent.priority()
                < ModifierOperation::MultiplyAdditive.priority()
        );
        assert!(
            ModifierOperation::MultiplyAdditive.priority()
                < ModifierOperation::MultiplyMultiplicative.priority()
        );
        assert!(
            ModifierOperation::MultiplyMultiplicative.priority()
                < ModifierOperation::Override.priority()
        );
    }
}
