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
    AbilityActivatedEvent, AbilityEndedEvent, CancelAbilityEvent, CommitAbilityEvent,
    TryActivateAbilityEvent,
};

// Re-export cue events
pub use crate::cues::systems::TriggerGameplayCueEvent;

/// Trait for events that can be batched for performance.
///
/// This allows multiple events of the same type to be processed together
/// in a single frame, reducing overhead.
pub trait BatchableEvent: Send + Sync + 'static {
    /// Returns true if this event can be batched with others of the same type.
    fn can_batch(&self) -> bool {
        true
    }
}

// Implement BatchableEvent for effect events (they benefit most from batching)
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
            target: Entity::from_bits(0),
            effect_id: "test".to_string(),
            level: 1,
            instigator: None,
        };

        assert!(event.can_batch());
    }
}
