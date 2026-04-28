//! Ability activation information and context.
//!
//! This module provides the unified activation context that carries all necessary
//! information when an ability is activated, similar to UE GAS's FGameplayAbilityActivationInfo
//! and FGameplayAbilityActorInfo.

use bevy::prelude::*;

use super::target_data::GameplayAbilityTargetData;

/// Unified activation information passed through the ability activation flow.
///
/// This replaces the previous scattered approach where target/source/context were passed
/// via temporary resources or separate parameters. Now all activation-time data flows
/// through this single value object.
///
/// Similar to UE GAS's `FGameplayAbilityActivationInfo` + `FGameplayAbilityActorInfo`.
#[derive(Debug, Clone, Component)]
pub struct AbilityActivationInfo {
    /// The entity that owns and is activating this ability (the "avatar").
    pub owner: Entity,

    /// The entity that originally granted or triggered this ability (may be same as owner).
    pub instigator: Entity,

    /// Target data captured at activation time.
    pub target_data: GameplayAbilityTargetData,

    /// Ability level at activation time.
    pub level: i32,

    /// Optional event payload that triggered this ability (for event-driven abilities).
    pub event_payload: Option<GameplayEventData>,
}

impl AbilityActivationInfo {
    /// Create a new activation info with the given owner and target data.
    pub fn new(owner: Entity, target_data: GameplayAbilityTargetData) -> Self {
        Self {
            owner,
            instigator: owner,
            target_data,
            level: 1,
            event_payload: None,
        }
    }

    /// Set the instigator (if different from owner).
    pub fn with_instigator(mut self, instigator: Entity) -> Self {
        self.instigator = instigator;
        self
    }

    /// Set the ability level.
    pub fn with_level(mut self, level: i32) -> Self {
        self.level = level;
        self
    }

    /// Set the event payload.
    pub fn with_event_payload(mut self, payload: GameplayEventData) -> Self {
        self.event_payload = Some(payload);
        self
    }

    /// Get the primary target entity, if any.
    pub fn primary_target(&self) -> Option<Entity> {
        self.target_data.primary_target()
    }

    /// Get all target entities.
    pub fn all_targets(&self) -> &[Entity] {
        self.target_data.all_targets()
    }
}

/// Event payload data for event-driven ability activation.
///
/// Similar to UE GAS's `FGameplayEventData`.
#[derive(Debug, Clone)]
pub struct GameplayEventData {
    /// The entity that triggered this event.
    pub instigator: Entity,

    /// The primary target of this event.
    pub target: Option<Entity>,

    /// Optional magnitude value associated with this event.
    pub magnitude: Option<f32>,

    /// Optional tag associated with this event.
    pub event_tag: Option<String>,
}

impl GameplayEventData {
    /// Create a new event data with the given instigator.
    pub fn new(instigator: Entity) -> Self {
        Self {
            instigator,
            target: None,
            magnitude: None,
            event_tag: None,
        }
    }

    /// Set the target.
    pub fn with_target(mut self, target: Entity) -> Self {
        self.target = Some(target);
        self
    }

    /// Set the magnitude.
    pub fn with_magnitude(mut self, magnitude: f32) -> Self {
        self.magnitude = Some(magnitude);
        self
    }

    /// Set the event tag.
    pub fn with_event_tag(mut self, tag: String) -> Self {
        self.event_tag = Some(tag);
        self
    }
}
