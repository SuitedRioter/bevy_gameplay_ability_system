//! Integration tests for specialized GameplayCue types.
//!
//! Tests BurstCue, LoopingCue, and HitImpactCue behavior.

use bevy::prelude::*;
use bevy_gameplay_ability_system::{
    GasPlugin,
    cues::{BurstCue, HitImpactCue, LoopingCue, LoopingCuePendingRemoval},
};
use bevy_gameplay_tag::{GameplayTag, GameplayTagsPlugin};

fn setup_test_app() -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        GameplayTagsPlugin::with_data_path("assets/gameplay_tags.json".to_string()),
        GasPlugin,
    ));

    // Update once to initialize
    app.update();
    app
}

#[test]
fn test_burst_cue_executes_and_despawns() {
    let mut app = setup_test_app();

    let tag = GameplayTag::new("GameplayCue.Impact");
    let target = app.world_mut().spawn_empty().id();

    // Spawn a burst cue
    let burst_entity = app
        .world_mut()
        .spawn((
            BurstCue::new(tag.clone()).with_target(target),
            Transform::default(),
        ))
        .id();

    // Verify burst cue exists
    assert!(app.world().get_entity(burst_entity).is_ok());

    // Update to process burst cue
    app.update();

    // Burst cue should be despawned after execution
    assert!(app.world().get_entity(burst_entity).is_err());
}

#[test]
fn test_looping_cue_starts_and_persists() {
    let mut app = setup_test_app();

    let tag = GameplayTag::new("GameplayCue.Buff.Shield");
    let target = app.world_mut().spawn_empty().id();

    // Spawn a looping cue
    let looping_entity = app
        .world_mut()
        .spawn((
            LoopingCue::new(tag.clone(), target, 0.0),
            Transform::default(),
        ))
        .id();

    // Verify looping cue exists
    assert!(app.world().get_entity(looping_entity).is_ok());

    // Update to start looping
    app.update();

    // Looping cue should still exist
    assert!(app.world().get_entity(looping_entity).is_ok());

    // Verify looping started
    let looping = app.world().get::<LoopingCue>(looping_entity).unwrap();
    assert!(looping.looping_started);
    assert!(!looping.looping_removed);
}

#[test]
fn test_looping_cue_recurring_events() {
    let mut app = setup_test_app();

    let tag = GameplayTag::new("GameplayCue.DoT.Poison");
    let target = app.world_mut().spawn_empty().id();

    // Spawn a looping cue with 1 second recurring interval
    let looping_entity = app
        .world_mut()
        .spawn((
            LoopingCue::new(tag.clone(), target, 0.0).with_recurring_interval(1.0),
            Transform::default(),
        ))
        .id();

    // Update to start looping
    app.update();

    // Get initial state
    let looping = app.world().get::<LoopingCue>(looping_entity).unwrap();
    let initial_time = looping.last_recurring_time;

    // Advance time by 1.5 seconds
    app.world_mut()
        .resource_mut::<Time<Virtual>>()
        .advance_by(std::time::Duration::from_secs_f32(1.5));

    // Update to process recurring event
    app.update();

    // Verify recurring event occurred
    let looping = app.world().get::<LoopingCue>(looping_entity).unwrap();
    assert!(looping.last_recurring_time > initial_time);
}

#[test]
fn test_looping_cue_removal() {
    let mut app = setup_test_app();

    let tag = GameplayTag::new("GameplayCue.Buff.Speed");
    let target = app.world_mut().spawn_empty().id();

    // Spawn a looping cue
    let looping_entity = app
        .world_mut()
        .spawn((
            LoopingCue::new(tag.clone(), target, 0.0),
            Transform::default(),
        ))
        .id();

    // Update to start looping
    app.update();

    // Mark for removal
    app.world_mut()
        .entity_mut(looping_entity)
        .insert(LoopingCuePendingRemoval);

    // Update to process removal
    app.update();

    // Looping cue should be despawned
    assert!(app.world().get_entity(looping_entity).is_err());
}

#[test]
fn test_hit_impact_cue_executes_with_collision_data() {
    let mut app = setup_test_app();

    let tag = GameplayTag::new("GameplayCue.Impact.Bullet");
    let target = app.world_mut().spawn_empty().id();
    let hit_location = Vec3::new(10.0, 5.0, 0.0);
    let hit_normal = Vec3::new(0.0, 1.0, 0.0);

    // Spawn a hit impact cue
    let impact_entity = app
        .world_mut()
        .spawn((
            HitImpactCue::new(tag.clone(), hit_location, hit_normal)
                .with_target(target)
                .with_surface_type("Metal")
                .with_impact_velocity(25.0),
            Transform::default(),
        ))
        .id();

    // Verify impact cue exists
    assert!(app.world().get_entity(impact_entity).is_ok());

    // Update to process impact cue
    app.update();

    // Impact cue should be despawned after execution
    assert!(app.world().get_entity(impact_entity).is_err());
}

#[test]
fn test_multiple_burst_cues_execute_independently() {
    let mut app = setup_test_app();

    let tag1 = GameplayTag::new("GameplayCue.Impact.Sword");
    let tag2 = GameplayTag::new("GameplayCue.Impact.Arrow");

    // Spawn multiple burst cues
    let burst1 = app
        .world_mut()
        .spawn((BurstCue::new(tag1), Transform::default()))
        .id();

    let burst2 = app
        .world_mut()
        .spawn((BurstCue::new(tag2), Transform::default()))
        .id();

    // Update to process burst cues
    app.update();

    // Both burst cues should be despawned
    assert!(app.world().get_entity(burst1).is_err());
    assert!(app.world().get_entity(burst2).is_err());
}

#[test]
fn test_looping_cue_without_recurring_interval() {
    let mut app = setup_test_app();

    let tag = GameplayTag::new("GameplayCue.Buff.Aura");
    let target = app.world_mut().spawn_empty().id();

    // Spawn a looping cue without recurring interval
    let looping_entity = app
        .world_mut()
        .spawn((
            LoopingCue::new(tag.clone(), target, 0.0),
            Transform::default(),
        ))
        .id();

    // Update to start looping
    app.update();

    // Verify looping started but no recurring events
    let looping = app.world().get::<LoopingCue>(looping_entity).unwrap();
    assert!(looping.looping_started);
    assert!(looping.recurring_interval.is_none());
    assert!(!looping.should_recur(100.0)); // Should never recur
}
