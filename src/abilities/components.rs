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

/// Tracks activation history for an ability.
///
/// Records statistics about ability activations for debugging, analytics,
/// and gameplay logic (e.g., combo systems, cooldown reduction based on usage).
///
/// Matches UE GAS's activation tracking in `FGameplayAbilityActivationInfo`.
#[derive(Component, Debug, Clone)]
pub struct AbilityActivationHistory {
    /// Total number of times this ability has been activated.
    pub activation_count: u32,
    /// Game time (in seconds) when this ability was last activated.
    pub last_activation_time: f64,
    /// Result of the last activation attempt.
    pub last_activation_result: ActivationResult,
    /// Game time (in seconds) when this ability was last successfully activated.
    pub last_successful_activation_time: Option<f64>,
    /// Number of successful activations.
    pub successful_activation_count: u32,
    /// Number of failed activations.
    pub failed_activation_count: u32,
}

impl Default for AbilityActivationHistory {
    fn default() -> Self {
        Self {
            activation_count: 0,
            last_activation_time: 0.0,
            last_activation_result: ActivationResult::Success,
            last_successful_activation_time: None,
            successful_activation_count: 0,
            failed_activation_count: 0,
        }
    }
}

impl AbilityActivationHistory {
    /// Creates a new empty activation history.
    pub fn new() -> Self {
        Self::default()
    }

    /// Records a new activation attempt.
    pub fn record_activation(&mut self, time: f64, result: ActivationResult) {
        self.activation_count += 1;
        self.last_activation_time = time;
        self.last_activation_result = result.clone();

        match result {
            ActivationResult::Success => {
                self.successful_activation_count += 1;
                self.last_successful_activation_time = Some(time);
            }
            _ => {
                self.failed_activation_count += 1;
            }
        }
    }

    /// Returns the time since the last activation.
    pub fn time_since_last_activation(&self, current_time: f64) -> f64 {
        current_time - self.last_activation_time
    }

    /// Returns the time since the last successful activation.
    pub fn time_since_last_successful_activation(&self, current_time: f64) -> Option<f64> {
        self.last_successful_activation_time
            .map(|time| current_time - time)
    }

    /// Returns the success rate (0.0 to 1.0).
    pub fn success_rate(&self) -> f32 {
        if self.activation_count == 0 {
            0.0
        } else {
            self.successful_activation_count as f32 / self.activation_count as f32
        }
    }
}

/// Result of an ability activation attempt.
///
/// Matches UE GAS's activation result types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActivationResult {
    /// Ability activated successfully.
    Success,
    /// Activation failed due to missing requirements.
    FailedRequirements,
    /// Activation failed due to insufficient resources (mana, stamina, etc.).
    FailedCost,
    /// Activation failed because the ability is on cooldown.
    FailedCooldown,
    /// Activation failed due to blocking tags.
    FailedBlocked,
    /// Activation failed for other reasons.
    Failed,
}

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
    /// The entity that owns this ability (the character/actor).
    pub owner: Entity,
    /// The entity that initiated the activation (may be different from owner).
    pub instigator: Option<Entity>,
    /// Target data for this ability activation.
    pub target_data: Option<super::target_data::GameplayAbilityTargetData>,
}

impl std::fmt::Debug for AbilitySpecInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AbilitySpecInstance")
            .field("definition_id", &self.definition_id)
            .field("level", &self.level)
            .field("behavior", &self.behavior.as_ref().map(|_| "<behavior>"))
            .field("owner", &self.owner)
            .field("instigator", &self.instigator)
            .field("target_data", &self.target_data)
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
#[path = "activation_history_tests.rs"]
mod activation_history_tests;

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
