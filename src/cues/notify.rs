//! GameplayCue notify traits and components.
//!
//! This module defines the traits for implementing custom cue handlers.

use super::manager::GameplayCueParameters;
use bevy::prelude::*;
use bevy_gameplay_tag::gameplay_tag::GameplayTag;

/// Trait for static gameplay cue notifies.
///
/// Static cues are function-based and don't spawn entities.
/// They're ideal for simple effects like playing sounds or spawning particles.
pub trait GameplayCueNotifyStatic: Send + Sync + 'static {
    /// Called when the cue is executed.
    fn on_execute(&self, target: Entity, params: &GameplayCueParameters, commands: &mut Commands);

    /// Called when the cue becomes active (for duration-based cues).
    fn on_active(&self, target: Entity, params: &GameplayCueParameters, commands: &mut Commands) {
        // Default implementation just calls on_execute
        self.on_execute(target, params, commands);
    }

    /// Called when the cue is removed (for duration-based cues).
    fn on_remove(&self, target: Entity, params: &GameplayCueParameters, commands: &mut Commands) {
        // Default: do nothing
        let _ = (target, params, commands);
    }

    /// Called every frame while the cue is active (for WhileActive cues).
    fn while_active(
        &self,
        target: Entity,
        params: &GameplayCueParameters,
        commands: &mut Commands,
    ) {
        // Default: do nothing
        let _ = (target, params, commands);
    }
}

/// Component for actor-based gameplay cue notifies.
///
/// Actor cues spawn entities that persist for the duration of the cue.
/// They're ideal for complex effects that need to track state or update over time.
#[derive(Component, Debug, Clone)]
pub struct GameplayCueNotifyActor {
    /// The cue tag this actor responds to.
    pub cue_tag: GameplayTag,
    /// The target entity.
    pub target: Entity,
    /// Whether to automatically destroy this actor when the cue is removed.
    pub auto_destroy_on_remove: bool,
    /// The time when this cue was activated.
    pub activation_time: f32,
}

impl GameplayCueNotifyActor {
    /// Creates a new cue notify actor.
    pub fn new(cue_tag: GameplayTag, target: Entity, activation_time: f32) -> Self {
        Self {
            cue_tag,
            target,
            auto_destroy_on_remove: true,
            activation_time,
        }
    }

    /// Sets whether to auto-destroy on remove.
    pub fn with_auto_destroy(mut self, auto_destroy: bool) -> Self {
        self.auto_destroy_on_remove = auto_destroy;
        self
    }
}

/// Marker component for cue actors that should be removed.
#[derive(Component, Debug)]
pub struct CueActorPendingRemoval;

/// Example static cue implementation.
///
/// This is a simple example showing how to implement a static cue.
pub struct ExampleStaticCue;

impl GameplayCueNotifyStatic for ExampleStaticCue {
    fn on_execute(&self, target: Entity, params: &GameplayCueParameters, _commands: &mut Commands) {
        info!(
            "Example cue executed on {:?} at {:?} with magnitude {}",
            target, params.location, params.raw_magnitude
        );
    }

    fn on_active(&self, target: Entity, params: &GameplayCueParameters, _commands: &mut Commands) {
        info!(
            "Example cue activated on {:?} at {:?}",
            target, params.location
        );
    }

    fn on_remove(&self, target: Entity, _params: &GameplayCueParameters, _commands: &mut Commands) {
        info!("Example cue removed from {:?}", target);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cue_notify_actor_creation() {
        let tag = GameplayTag::new("GameplayCue.Test");
        let target = Entity::from_bits(0);
        let actor = GameplayCueNotifyActor::new(tag.clone(), target, 0.0);

        assert_eq!(actor.cue_tag, tag);
        assert_eq!(actor.target, target);
        assert!(actor.auto_destroy_on_remove);
    }

    #[test]
    fn test_cue_notify_actor_builder() {
        let tag = GameplayTag::new("GameplayCue.Test");
        let target = Entity::from_bits(0);
        let actor = GameplayCueNotifyActor::new(tag, target, 0.0).with_auto_destroy(false);

        assert!(!actor.auto_destroy_on_remove);
    }
}
