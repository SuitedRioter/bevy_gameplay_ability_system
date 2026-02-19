//! Ability system components.
//!
//! This module defines the core components for the gameplay ability system.

use bevy::prelude::*;

/// Ability specification component.
///
/// Represents a granted ability instance. Each granted ability is a separate entity.
#[derive(Component, Debug, Clone)]
pub struct AbilitySpec {
    /// The ID of the ability definition.
    pub definition_id: String,
    /// The level at which this ability was granted.
    pub level: i32,
    /// Optional input ID for binding to input actions.
    pub input_id: Option<i32>,
    /// Whether this ability is currently active.
    pub is_active: bool,
}

impl AbilitySpec {
    /// Creates a new ability spec.
    pub fn new(definition_id: String, level: i32) -> Self {
        Self {
            definition_id,
            level,
            input_id: None,
            is_active: false,
        }
    }

    /// Sets the input ID for this ability.
    pub fn with_input_id(mut self, input_id: i32) -> Self {
        self.input_id = Some(input_id);
        self
    }
}

/// Component that links an ability to its owner entity.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AbilityOwner(pub Entity);

/// Active ability instance component.
///
/// Created when an ability is activated with InstancedPerExecution policy.
#[derive(Component, Debug, Clone)]
pub struct ActiveAbilityInstance {
    /// The ability spec entity that spawned this instance.
    pub spec_entity: Entity,
    /// The time when this ability was activated.
    pub activation_time: f32,
    /// Whether this ability has been committed (costs/cooldowns applied).
    pub is_committed: bool,
}

impl ActiveAbilityInstance {
    /// Creates a new active ability instance.
    pub fn new(spec_entity: Entity, activation_time: f32) -> Self {
        Self {
            spec_entity,
            activation_time,
            is_committed: false,
        }
    }
}

/// Ability state component.
///
/// Tracks the current state of an ability.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AbilityState {
    /// Ability is ready to be activated.
    #[default]
    Ready,
    /// Ability is currently active.
    Active,
    /// Ability is on cooldown.
    Cooldown,
    /// Ability is blocked by tags or other conditions.
    Blocked,
}

/// Component that marks an ability as being on cooldown.
#[derive(Component, Debug, Clone, Copy)]
pub struct AbilityCooldown {
    /// Remaining cooldown time in seconds.
    pub remaining: f32,
    /// Total cooldown duration in seconds.
    pub total: f32,
}

impl AbilityCooldown {
    /// Creates a new cooldown.
    pub fn new(duration: f32) -> Self {
        Self {
            remaining: duration,
            total: duration,
        }
    }

    /// Returns true if the cooldown has expired.
    pub fn is_expired(&self) -> bool {
        self.remaining <= 0.0
    }

    /// Updates the remaining time.
    pub fn tick(&mut self, delta: f32) {
        self.remaining -= delta;
    }

    /// Returns the cooldown progress (0.0 = just started, 1.0 = finished).
    pub fn progress(&self) -> f32 {
        if self.total <= 0.0 {
            1.0
        } else {
            1.0 - (self.remaining / self.total).max(0.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ability_spec_creation() {
        let spec = AbilitySpec::new("test_ability".to_string(), 1);
        assert_eq!(spec.definition_id, "test_ability");
        assert_eq!(spec.level, 1);
        assert_eq!(spec.input_id, None);
        assert!(!spec.is_active);
    }

    #[test]
    fn test_ability_spec_with_input() {
        let spec = AbilitySpec::new("test_ability".to_string(), 1).with_input_id(42);
        assert_eq!(spec.input_id, Some(42));
    }

    #[test]
    fn test_cooldown() {
        let mut cooldown = AbilityCooldown::new(5.0);
        assert_eq!(cooldown.remaining, 5.0);
        assert!(!cooldown.is_expired());
        assert_eq!(cooldown.progress(), 0.0);

        cooldown.tick(2.5);
        assert_eq!(cooldown.remaining, 2.5);
        assert!(!cooldown.is_expired());
        assert!((cooldown.progress() - 0.5).abs() < f32::EPSILON);

        cooldown.tick(3.0);
        assert_eq!(cooldown.remaining, -0.5);
        assert!(cooldown.is_expired());
        assert_eq!(cooldown.progress(), 1.0);
    }

    #[test]
    fn test_ability_state_default() {
        let state = AbilityState::default();
        assert_eq!(state, AbilityState::Ready);
    }
}
