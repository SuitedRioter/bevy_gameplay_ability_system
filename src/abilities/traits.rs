//! Ability behavior traits.
//!
//! Defines the lifecycle hooks for custom ability implementations.

use bevy::prelude::*;

/// Ability behavior trait for custom ability logic.
///
/// Implement this trait to define custom behavior for abilities.
/// All methods have default implementations that do nothing.
pub trait AbilityBehavior: Send + Sync + 'static {
    /// Check if the ability can be activated.
    ///
    /// Called before any costs are applied. Return false to prevent activation.
    fn can_activate(
        &self,
        _world: &World,
        _ability_entity: Entity,
        _source: Entity,
        _target: Option<Entity>,
    ) -> bool {
        true
    }

    /// Called before activation begins.
    ///
    /// Use this for setup logic before the ability enters the Activating state.
    fn pre_activate(
        &self,
        _world: &mut World,
        _ability_entity: Entity,
        _source: Entity,
        _target: Option<Entity>,
    ) {
    }

    /// Called when the ability is activated.
    ///
    /// This is where the main ability logic should go (spawn projectiles, apply effects, etc).
    fn activate(
        &self,
        _world: &mut World,
        _ability_entity: Entity,
        _source: Entity,
        _target: Option<Entity>,
    ) {
    }

    /// Called when the ability is committed.
    ///
    /// This happens after costs and cooldowns are applied.
    /// Use this for logic that should only run if the ability successfully committed.
    fn commit(
        &self,
        _world: &mut World,
        _ability_entity: Entity,
    ) {
    }

    /// Called when the ability ends.
    ///
    /// Use this for cleanup logic. The `was_cancelled` parameter indicates
    /// whether the ability ended normally or was cancelled.
    fn end(
        &self,
        _world: &mut World,
        _ability_entity: Entity,
        _was_cancelled: bool,
    ) {
    }
}
