//! Ability activation context.
//!
//! This module provides the activation context for abilities, which contains
//! information about the ability activation such as the source, target, and
//! activation parameters.

use bevy::prelude::*;

use super::target_data::GameplayAbilityTargetData;

/// Context information for ability activation.
///
/// Mirrors UE's `FGameplayAbilityActivationInfo` and `FGameplayAbilityActorInfo`.
#[derive(Component, Clone, Debug)]
pub struct AbilityActivationContext {
    /// The entity that owns the ability.
    pub owner: Entity,
    /// The entity that activated the ability (may differ from owner).
    pub activator: Entity,
    /// Optional target entity (legacy, prefer using target_data).
    pub target: Option<Entity>,
    /// Activation level (for scaling).
    pub level: i32,
    /// Target data handle (UE GAS style).
    pub target_data: Option<GameplayAbilityTargetData>,
}

impl AbilityActivationContext {
    /// Create a new activation context.
    pub fn new(owner: Entity, activator: Entity) -> Self {
        Self {
            owner,
            activator,
            target: None,
            level: 1,
            target_data: None,
        }
    }

    /// Set the target entity (legacy).
    pub fn with_target(mut self, target: Entity) -> Self {
        self.target = Some(target);
        self
    }

    /// Set the activation level.
    pub fn with_level(mut self, level: i32) -> Self {
        self.level = level;
        self
    }

    /// Set the target data handle.
    pub fn with_target_data(mut self, target_data: GameplayAbilityTargetData) -> Self {
        self.target_data = Some(target_data);
        self
    }

    /// Get the first targeted actor from target data, or fall back to legacy target field.
    pub fn get_primary_target(&self) -> Option<Entity> {
        if let Some(ref td) = self.target_data {
            td.all_targets().first().copied()
        } else {
            self.target
        }
    }

    /// Get all targeted actors from target data, or fall back to legacy target field.
    pub fn get_all_targets(&self) -> Vec<Entity> {
        if let Some(ref td) = self.target_data {
            td.all_targets().to_vec()
        } else {
            self.target.into_iter().collect()
        }
    }
}
