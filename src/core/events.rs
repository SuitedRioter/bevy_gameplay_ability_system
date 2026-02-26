//! Centralized event definitions.
//!
//! This module re-exports all events from the various GAS subsystems
//! for convenient access.

// Re-export attribute events
pub use crate::attributes::systems::AttributeChangedEvent;

// Re-export effect events
pub use crate::effects::systems::{
    ApplyGameplayEffectEvent, GameplayEffectAppliedEvent, GameplayEffectRemovedEvent,
};

// Re-export ability events
pub use crate::abilities::systems::{
    AbilityActivatedEvent, AbilityActivationFailedEvent, AbilityEndedEvent, CancelAbilityEvent,
    CommitAbilityEvent, CommitAbilityResultEvent, EndAbilityEvent, TryActivateAbilityEvent,
};

// Re-export ability enums
pub use crate::abilities::systems::ActivationFailureReason;

// Re-export cue events
pub use crate::cues::systems::TriggerGameplayCueEvent;

/// Trait for events that can be batched for performance.
pub trait BatchableEvent: Send + Sync + 'static {
    fn can_batch(&self) -> bool {
        true
    }
}

impl BatchableEvent for ApplyGameplayEffectEvent {}
impl BatchableEvent for GameplayEffectAppliedEvent {}
impl BatchableEvent for GameplayEffectRemovedEvent {}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_batchable_event_trait() {
        let event = ApplyGameplayEffectEvent {
            target: Entity::PLACEHOLDER,
            effect_id: "test".to_string(),
            level: 1,
            instigator: None,
        };

        assert!(event.can_batch());
    }
}
