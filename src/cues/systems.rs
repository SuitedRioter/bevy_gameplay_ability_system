//! GameplayCue systems.
//!
//! This module contains the systems that handle gameplay cue execution.

use super::manager::{GameplayCueEvent, GameplayCueManager, GameplayCueParameters};
use super::notify::{CueActorPendingRemoval, GameplayCueNotifyActor};
use bevy::prelude::*;
use bevy_gameplay_tag::gameplay_tag::GameplayTag;

/// Event for triggering a gameplay cue.
#[derive(Event, Debug, Clone)]
pub struct TriggerGameplayCueEvent {
    /// The cue tag to trigger.
    pub cue_tag: GameplayTag,
    /// The event type.
    pub event_type: GameplayCueEvent,
    /// The parameters.
    pub parameters: GameplayCueParameters,
}

/// System that handles gameplay cue triggers.
///
/// This system processes TriggerGameplayCueEvent and routes them to the manager.
pub fn handle_gameplay_cue_system(manager: ResMut<GameplayCueManager>, _commands: Commands) {
    // TODO: Implement with Bevy 0.18 observer pattern
    // This will read TriggerGameplayCueEvent and call manager.execute_cue()
    let _ = manager;
}

/// System that routes gameplay cues to appropriate handlers.
///
/// This system finds matching cue handlers based on tag hierarchy and
/// executes them with the provided parameters.
pub fn route_gameplay_cue_system(
    manager: Res<GameplayCueManager>,
    _commands: Commands,
    _time: Res<Time>,
) {
    // TODO: Implement cue routing logic
    // This will:
    // 1. Check for pending cues in the manager
    // 2. Find matching cue handlers (including parent tags)
    // 3. Execute static cues or spawn actor cues
    let _ = manager;
}

/// System that executes static gameplay cues.
///
/// Static cues are function-based and don't spawn entities.
pub fn execute_static_cues_system(_manager: Res<GameplayCueManager>, _commands: Commands) {
    // TODO: Implement static cue execution
    // This will call the appropriate GameplayCueNotifyStatic trait methods
}

/// System that manages gameplay cue actors.
///
/// This system handles the lifecycle of actor-based cues, including
/// spawning, updating, and cleanup.
pub fn manage_cue_actors_system(
    mut commands: Commands,
    mut manager: ResMut<GameplayCueManager>,
    cue_actors: Query<(Entity, &GameplayCueNotifyActor)>,
    pending_removal: Query<Entity, With<CueActorPendingRemoval>>,
) {
    // Clean up actors marked for removal
    for entity in pending_removal.iter() {
        if let Ok((_, actor)) = cue_actors.get(entity) {
            manager.remove_active_cue(&actor.cue_tag, entity);
        }
        commands.entity(entity).despawn();
    }
}

/// System that cleans up finished gameplay cues.
///
/// This system removes cue actors that have completed their execution.
pub fn cleanup_finished_cues_system(
    commands: Commands,
    cue_actors: Query<(Entity, &GameplayCueNotifyActor)>,
    _time: Res<Time>,
) {
    // TODO: Implement cleanup logic
    // This will check for cues that should be removed based on:
    // - Duration expired
    // - Effect removed
    // - Manual cancellation
    for (_entity, _actor) in cue_actors.iter() {
        // Check if cue should be removed
        // If so, add CueActorPendingRemoval component
    }
    let _ = commands;
}

/// System that updates WhileActive cues every frame.
///
/// This system calls the while_active method on static cues that are
/// currently active.
pub fn update_while_active_cues_system(_manager: Res<GameplayCueManager>, _commands: Commands) {
    // TODO: Implement WhileActive cue updates
    // This will call GameplayCueNotifyStatic::while_active() for active cues
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trigger_cue_event_creation() {
        let tag = GameplayTag::new("GameplayCue.Test");
        let event = TriggerGameplayCueEvent {
            cue_tag: tag.clone(),
            event_type: GameplayCueEvent::Executed,
            parameters: GameplayCueParameters::new(),
        };

        assert_eq!(event.cue_tag, tag);
        assert_eq!(event.event_type, GameplayCueEvent::Executed);
    }
}
