//! Effect system components.
//!
//! This module defines the core components for the gameplay effect system.

use bevy::prelude::*;
use bevy_gameplay_tag::{GameplayTag, GameplayTagContainer};
use std::collections::HashMap;
use string_cache::DefaultAtom as Atom;

/// Active gameplay effect instance component.
///
/// Each active effect is a separate entity with this component.
/// The effect modifies attributes on the target entity.
#[derive(Component, Debug, Clone)]
pub struct ActiveGameplayEffect {
    /// The ID of the effect definition.
    pub definition_id: Atom,
    /// The level at which this effect was applied.
    pub level: i32,
    /// The time when this effect was applied (in seconds).
    pub start_time: f32,
    /// The current stack count for this effect.
    pub stack_count: i32,
}

impl ActiveGameplayEffect {
    /// Creates a new active gameplay effect.
    pub fn new(definition_id: impl Into<Atom>, level: i32, start_time: f32) -> Self {
        Self {
            definition_id: definition_id.into(),
            level,
            start_time,
            stack_count: 1,
        }
    }
}

/// Component storing SetByCaller magnitudes for an effect.
///
/// When applying an effect with SetByCaller magnitude calculations,
/// the caller must provide values for each data tag.
///
/// # Example
/// ```ignore
/// commands.spawn((
///     ActiveGameplayEffect::new("damage", 1, time),
///     SetByCallerMagnitudes::new()
///         .with_magnitude(damage_tag, 50.0)
///         .with_magnitude(crit_chance_tag, 0.25),
/// ));
/// ```
#[derive(Component, Debug, Clone, Default)]
pub struct SetByCallerMagnitudes {
    magnitudes: HashMap<GameplayTag, f32>,
}

impl SetByCallerMagnitudes {
    /// Creates a new empty magnitude map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a magnitude for a data tag.
    pub fn with_magnitude(mut self, tag: GameplayTag, magnitude: f32) -> Self {
        self.magnitudes.insert(tag, magnitude);
        self
    }

    /// Sets a magnitude for a data tag.
    pub fn set_magnitude(&mut self, tag: GameplayTag, magnitude: f32) {
        self.magnitudes.insert(tag, magnitude);
    }

    /// Gets a magnitude for a data tag.
    pub fn get_magnitude(&self, tag: &GameplayTag) -> Option<f32> {
        self.magnitudes.get(tag).copied()
    }
}

/// Component that links an effect to its target entity.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectTarget(pub Entity);

/// Component tracking abilities granted by this effect.
///
/// When an effect grants abilities, this component stores the ability spec entities
/// so they can be properly removed when the effect ends.
#[derive(Component, Debug, Clone, Default)]
pub struct EffectGrantedAbilities {
    /// List of ability spec entities granted by this effect.
    pub granted_ability_specs: Vec<Entity>,
}

/// Component that identifies the instigator of an effect.
///
/// This is the entity that caused the effect to be applied.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectInstigator(pub Option<Entity>);

/// Context information for a gameplay effect.
///
/// Stores information about where the effect came from and how it was applied.
/// This is useful for tracking damage sources, applying conditional logic, etc.
///
/// # Example
/// ```ignore
/// commands.spawn((
///     ActiveGameplayEffect::new("damage", 1, time),
///     GameplayEffectContext::new()
///         .with_source(caster_entity)
///         .with_instigator(weapon_entity)
///         .with_hit_location(Vec3::new(0.0, 1.0, 0.0)),
/// ));
/// ```
#[derive(Component, Debug, Clone)]
pub struct GameplayEffectContext {
    /// The entity that owns the ability/effect (e.g., the player).
    pub source: Option<Entity>,
    /// The entity that directly caused the effect (e.g., a projectile or weapon).
    pub instigator: Option<Entity>,
    /// The location where the effect was applied (e.g., hit location).
    pub hit_location: Option<Vec3>,
    /// The normal vector at the hit location.
    pub hit_normal: Option<Vec3>,
    /// Custom data that can be attached to the context.
    pub custom_data: HashMap<String, f32>,
}

impl Default for GameplayEffectContext {
    fn default() -> Self {
        Self::new()
    }
}

impl GameplayEffectContext {
    /// Creates a new empty context.
    pub fn new() -> Self {
        Self {
            source: None,
            instigator: None,
            hit_location: None,
            hit_normal: None,
            custom_data: HashMap::new(),
        }
    }

    /// Sets the source entity.
    pub fn with_source(mut self, source: Entity) -> Self {
        self.source = Some(source);
        self
    }

    /// Sets the instigator entity.
    pub fn with_instigator(mut self, instigator: Entity) -> Self {
        self.instigator = Some(instigator);
        self
    }

    /// Sets the hit location.
    pub fn with_hit_location(mut self, location: Vec3) -> Self {
        self.hit_location = Some(location);
        self
    }

    /// Sets the hit normal.
    pub fn with_hit_normal(mut self, normal: Vec3) -> Self {
        self.hit_normal = Some(normal);
        self
    }

    /// Adds custom data.
    pub fn with_custom_data(mut self, key: impl Into<String>, value: f32) -> Self {
        self.custom_data.insert(key.into(), value);
        self
    }

    /// Gets custom data by key.
    pub fn get_custom_data(&self, key: &str) -> Option<f32> {
        self.custom_data.get(key).copied()
    }
}

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

    /// Updates the timer and returns the number of times the effect should execute.
    pub fn tick(&mut self, delta: f32) -> u32 {
        self.time_until_next -= delta;
        let mut executions = 0;
        while self.time_until_next <= 0.0 {
            executions += 1;
            self.time_until_next += self.period;
        }
        executions
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
    pub target_attribute: Atom,
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
    /// Override has highest priority (checked first, short-circuits).
    /// Lower values are applied first for other operations.
    pub fn priority(&self) -> i32 {
        match self {
            ModifierOperation::Override => 0,
            ModifierOperation::AddBase => 1,
            ModifierOperation::AddCurrent => 2,
            ModifierOperation::MultiplyAdditive => 3,
            ModifierOperation::MultiplyMultiplicative => 4,
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

/// Component that tracks abilities granted by this effect.
///
/// When the effect is removed, these abilities should also be removed.
#[derive(Component, Debug, Clone, Default)]
pub struct GrantedAbilities {
    /// List of ability spec entities that were granted by this effect.
    pub ability_specs: Vec<Entity>,
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

        assert_eq!(periodic.tick(0.5), 0);
        assert_eq!(periodic.time_until_next, 0.5);

        assert_eq!(periodic.tick(0.6), 1);
        assert_eq!(periodic.time_until_next, 0.9);
    }

    #[test]
    fn test_periodic_effect_large_delta() {
        let mut periodic = PeriodicEffect::new(1.0);

        // Large delta should trigger multiple executions
        assert_eq!(periodic.tick(2.5), 2);
        assert_eq!(periodic.time_until_next, 0.5);
    }

    #[test]
    fn test_modifier_operation_priority() {
        assert!(ModifierOperation::Override.priority() < ModifierOperation::AddBase.priority());
        assert!(ModifierOperation::AddBase.priority() < ModifierOperation::AddCurrent.priority());
        assert!(
            ModifierOperation::AddCurrent.priority()
                < ModifierOperation::MultiplyAdditive.priority()
        );
        assert!(
            ModifierOperation::MultiplyAdditive.priority()
                < ModifierOperation::MultiplyMultiplicative.priority()
        );
    }
}
