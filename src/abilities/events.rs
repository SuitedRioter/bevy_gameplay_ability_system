//! Gameplay event system for ability triggers.
//!
//! Provides a generic event system that can trigger abilities.

use bevy::prelude::*;
use bevy_gameplay_tag::gameplay_tag::GameplayTag;

use super::target_data::GameplayAbilityTargetData;

/// A gameplay event that can trigger abilities.
///
/// This is a generic event system that carries a tag and optional payload.
/// Abilities with matching trigger tags will be automatically activated.
#[derive(Event, Debug, Clone)]
pub struct GameplayEvent {
    /// The tag identifying this event type.
    pub event_tag: GameplayTag,
    /// The entity that triggered this event (optional).
    pub instigator: Option<Entity>,
    /// The target entity for this event (optional).
    pub target: Option<Entity>,
    /// Optional magnitude value (e.g., damage amount, heal amount).
    pub magnitude: Option<f32>,
    /// Optional targeting data.
    pub target_data: Option<GameplayAbilityTargetData>,
}

impl GameplayEvent {
    /// Creates a new gameplay event.
    pub fn new(event_tag: GameplayTag) -> Self {
        Self {
            event_tag,
            instigator: None,
            target: None,
            magnitude: None,
            target_data: None,
        }
    }

    /// Sets the instigator.
    pub fn with_instigator(mut self, instigator: Entity) -> Self {
        self.instigator = Some(instigator);
        self
    }

    /// Sets the target.
    pub fn with_target(mut self, target: Entity) -> Self {
        self.target = Some(target);
        self
    }

    /// Sets the magnitude.
    pub fn with_magnitude(mut self, magnitude: f32) -> Self {
        self.magnitude = Some(magnitude);
        self
    }

    /// Sets the target data.
    pub fn with_target_data(mut self, target_data: GameplayAbilityTargetData) -> Self {
        self.target_data = Some(target_data);
        self
    }
}
