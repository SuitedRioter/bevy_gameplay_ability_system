//! Ability system components.
//!
//! Entity hierarchy:
//!   Owner Entity (player)
//!     └── AbilitySpec Entity (granted ability slot)
//!           ├── Components: AbilitySpec, AbilityActiveState, AbilityOwner
//!           └── AbilitySpecInstance Entity (active instance, child)
//!                 └── Components: AbilitySpecInstance, InstanceControlState

use bevy::prelude::*;
use std::sync::Arc;
use string_cache::DefaultAtom as Atom;

use super::traits::AbilityBehavior;

/// Ability specification component — lives on the granted-ability entity.
///
/// Represents a granted ability on a character. Contains only the reference
/// to the definition and per-grant configuration (level, input binding).
#[derive(Component, Clone)]
pub struct AbilitySpec {
    /// The ID of the ability definition in the AbilityRegistry.
    pub definition_id: Atom,
    /// The level at which this ability was granted.
    pub level: i32,
    /// Optional input ID for binding to input actions.
    pub input_id: Option<i32>,
}

impl std::fmt::Debug for AbilitySpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AbilitySpec")
            .field("definition_id", &self.definition_id)
            .field("level", &self.level)
            .field("input_id", &self.input_id)
            .finish()
    }
}

impl AbilitySpec {
    /// Creates a new ability spec.
    pub fn new(definition_id: impl Into<Atom>, level: i32) -> Self {
        Self {
            definition_id: definition_id.into(),
            level,
            input_id: None,
        }
    }

    /// Sets the input ID for this ability.
    pub fn with_input_id(mut self, input_id: i32) -> Self {
        self.input_id = Some(input_id);
        self
    }
}

/// Tracks activation state on the AbilitySpec entity.
///
/// Separated from AbilitySpec so Bevy change detection can track
/// activation state independently from the grant configuration.
#[derive(Component, Debug, Clone, Default)]
pub struct AbilityActiveState {
    /// Whether at least one instance is currently active.
    pub is_active: bool,
    /// Number of currently active instances.
    pub active_count: u8,
}

impl AbilityActiveState {
    pub fn increment(&mut self) {
        self.active_count += 1;
        self.is_active = true;
    }

    pub fn decrement(&mut self) {
        self.active_count = self.active_count.saturating_sub(1);
        if self.active_count == 0 {
            self.is_active = false;
        }
    }
}

/// Component that links an ability to its owner entity.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AbilityOwner(pub Entity);

// ---------------------------------------------------------------------------
// AbilitySpecInstance — spawned as a child of AbilitySpec on activation
// ---------------------------------------------------------------------------

/// An active ability instance. Spawned as a child entity of the AbilitySpec
/// entity when the ability is activated.
///
/// When the AbilitySpec entity is despawned (ability removed from character),
/// Bevy's hierarchy cleanup will also despawn all child instances. An observer
/// on removal of this component calls `behavior.end()` so cleanup logic runs.
#[derive(Component, Clone)]
pub struct AbilitySpecInstance {
    /// Copy of the definition ID (so we can look up the definition without
    /// querying the parent).
    pub definition_id: Atom,
    /// Level at the time of activation.
    pub level: i32,
    /// The behavior implementation for this instance.
    pub behavior: Option<Arc<dyn AbilityBehavior>>,
}

impl std::fmt::Debug for AbilitySpecInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AbilitySpecInstance")
            .field("definition_id", &self.definition_id)
            .field("level", &self.level)
            .field("behavior", &self.behavior.as_ref().map(|_| "<behavior>"))
            .finish()
    }
}

/// Runtime control flags for an active ability instance.
///
/// These mirror UE's FGameplayAbilityActivationInfo control flags.
#[derive(Component, Debug, Clone)]
pub struct InstanceControlState {
    /// Whether this instance is currently active (running its logic).
    pub is_active: bool,
    /// Whether this instance blocks other abilities from activating.
    pub is_blocking_other_abilities: bool,
    /// Whether this instance can be cancelled.
    pub is_cancelable: bool,
}

impl Default for InstanceControlState {
    fn default() -> Self {
        Self {
            is_active: true,
            is_blocking_other_abilities: false,
            is_cancelable: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ability_spec_creation() {
        let spec = AbilitySpec::new("test_ability", 1);
        assert_eq!(spec.definition_id, Atom::from("test_ability"));
        assert_eq!(spec.level, 1);
        assert_eq!(spec.input_id, None);
    }

    #[test]
    fn test_ability_spec_with_input() {
        let spec = AbilitySpec::new("test_ability", 1).with_input_id(42);
        assert_eq!(spec.input_id, Some(42));
    }

    #[test]
    fn test_ability_active_state() {
        let mut state = AbilityActiveState::default();
        assert!(!state.is_active);
        assert_eq!(state.active_count, 0);

        state.increment();
        assert!(state.is_active);
        assert_eq!(state.active_count, 1);

        state.increment();
        assert_eq!(state.active_count, 2);

        state.decrement();
        assert!(state.is_active);
        assert_eq!(state.active_count, 1);

        state.decrement();
        assert!(!state.is_active);
        assert_eq!(state.active_count, 0);
    }

}
