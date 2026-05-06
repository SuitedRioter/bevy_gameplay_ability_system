//! GameplayCue systems.
//!
//! This module contains the systems that handle gameplay cue execution.

use super::manager::{GameplayCueEvent, GameplayCueManager, GameplayCueParameters};
use super::notify::{
    BurstCue, CueActorPendingRemoval, GameplayCueNotifyActor, HitImpactCue, LoopingCue,
    LoopingCuePendingRemoval,
};
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

/// System that processes burst cues.
///
/// Burst cues are one-shot effects that execute immediately and then despawn.
pub fn process_burst_cues_system(
    mut commands: Commands,
    mut burst_cues: Query<(Entity, &mut BurstCue)>,
) {
    for (entity, mut burst) in burst_cues.iter_mut() {
        if burst.executed {
            continue;
        }

        // Mark as executed
        burst.executed = true;

        // Log execution (in a real implementation, this would trigger VFX/SFX)
        info!(
            "Burst cue executed: {:?} on target {:?}",
            burst.cue_tag, burst.target
        );

        // Despawn the burst cue entity after execution
        commands.entity(entity).despawn();
    }
}

/// System that processes looping cues.
///
/// Looping cues persist and can fire recurring events at intervals.
pub fn process_looping_cues_system(
    commands: Commands,
    mut looping_cues: Query<(Entity, &mut LoopingCue), Without<LoopingCuePendingRemoval>>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs_f64();

    for (_entity, mut looping) in looping_cues.iter_mut() {
        // Start looping effects on first update
        if !looping.looping_started {
            looping.looping_started = true;
            info!(
                "Looping cue started: {:?} on target {:?}",
                looping.cue_tag, looping.target
            );
        }

        // Check for recurring events
        if looping.should_recur(current_time) {
            info!(
                "Looping cue recurring: {:?} on target {:?}",
                looping.cue_tag, looping.target
            );
            looping.mark_recurred(current_time);
        }
    }

    let _ = commands;
}

/// System that cleans up looping cues marked for removal.
///
/// This system handles the OnRemoval event and despawns the cue entity.
pub fn cleanup_looping_cues_system(
    mut commands: Commands,
    mut looping_cues: Query<(Entity, &mut LoopingCue), With<LoopingCuePendingRemoval>>,
) {
    for (entity, mut looping) in looping_cues.iter_mut() {
        if !looping.looping_removed {
            looping.looping_removed = true;
            info!(
                "Looping cue removed: {:?} on target {:?}",
                looping.cue_tag, looping.target
            );
        }

        // Despawn the looping cue entity
        commands.entity(entity).despawn();
    }
}

/// System that processes hit impact cues.
///
/// Hit impact cues are specialized burst cues with collision information.
pub fn process_hit_impact_cues_system(
    mut commands: Commands,
    mut impact_cues: Query<(Entity, &mut HitImpactCue)>,
) {
    for (entity, mut impact) in impact_cues.iter_mut() {
        if impact.executed {
            continue;
        }

        // Mark as executed
        impact.executed = true;

        // Log execution with impact details
        info!(
            "Hit impact cue executed: {:?} at {:?} (normal: {:?}, surface: {:?}, velocity: {:?})",
            impact.cue_tag,
            impact.hit_location,
            impact.hit_normal,
            impact.surface_type,
            impact.impact_velocity
        );

        // Despawn the impact cue entity after execution
        commands.entity(entity).despawn();
    }
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
