//! Integration tests for ability tasks.
//!
//! Tests WaitDelay and WaitGameplayEvent task behavior.

use bevy::prelude::*;
use bevy_gameplay_ability_system::{
    GasPlugin,
    abilities::{
        AbilitySpec, AbilityTask, GameplayEvent, TaskCancelledEvent, TaskCompletedEvent, TaskState,
        WaitDelayTask, WaitGameplayEventTask,
    },
};
use bevy_gameplay_tag::{GameplayTag, GameplayTagsPlugin};
use std::sync::{Arc, Mutex};

/// Helper to capture task events.
#[derive(Resource, Default)]
struct TaskEvents {
    completed: Arc<Mutex<Vec<Entity>>>,
    cancelled: Arc<Mutex<Vec<Entity>>>,
}

fn setup_test_app() -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        GameplayTagsPlugin::with_data_path("assets/gameplay_tags.json".to_string()),
        GasPlugin,
    ))
    .insert_resource(TaskEvents::default())
    .add_observer(capture_task_completed)
    .add_observer(capture_task_cancelled);

    // Update once to initialize the tags manager
    app.update();
    app
}

fn capture_task_completed(trigger: On<TaskCompletedEvent>, events: Res<TaskEvents>) {
    let event = trigger.event();
    events.completed.lock().unwrap().push(event.task);
}

fn capture_task_cancelled(trigger: On<TaskCancelledEvent>, events: Res<TaskEvents>) {
    let event = trigger.event();
    events.cancelled.lock().unwrap().push(event.task);
}

#[test]
fn test_wait_delay_task_state_changes() {
    let mut app = setup_test_app();

    let owner = app.world_mut().spawn_empty().id();
    let spec = app
        .world_mut()
        .spawn(AbilitySpec::new("test_ability", 1))
        .id();

    // Spawn a WaitDelay task with 1.0 second duration
    let task = app
        .world_mut()
        .spawn((
            AbilityTask {
                ability_instance: None,
                ability_spec: spec,
                owner,
            },
            WaitDelayTask::new(1.0),
            TaskState::Running,
        ))
        .id();

    // Manually set remaining to 0.1 to simulate near completion
    {
        let mut wait_delay = app.world_mut().get_mut::<WaitDelayTask>(task).unwrap();
        wait_delay.remaining = 0.1;
    }

    app.update();

    // Task should still be running
    let state = app.world().get::<TaskState>(task).copied();
    assert_eq!(
        state,
        Some(TaskState::Running),
        "Task should still be running"
    );

    // Manually set remaining to -0.1 to simulate completion
    {
        let mut wait_delay = app.world_mut().get_mut::<WaitDelayTask>(task).unwrap();
        wait_delay.remaining = -0.1;
    }

    app.update();

    // Check that TaskCompletedEvent was triggered
    {
        let events = app.world().resource::<TaskEvents>();
        let completed = events.completed.lock().unwrap();
        assert!(
            completed.contains(&task),
            "TaskCompletedEvent should be triggered"
        );
    }

    // Task should be despawned by cleanup system in the same update
    assert!(
        app.world().get_entity(task).is_err(),
        "Task should be despawned after completion"
    );
}

#[test]
fn test_wait_gameplay_event_task_triggers_on_matching_event() {
    let mut app = setup_test_app();

    let owner = app.world_mut().spawn_empty().id();
    let spec = app
        .world_mut()
        .spawn(AbilitySpec::new("test_ability", 1))
        .id();

    let event_tag = GameplayTag::new("Event.Test.Trigger");

    // Spawn a WaitGameplayEvent task
    let task = app
        .world_mut()
        .spawn((
            AbilityTask {
                ability_instance: None,
                ability_spec: spec,
                owner,
            },
            WaitGameplayEventTask::new(event_tag.clone()),
            TaskState::Running,
        ))
        .id();

    // Trigger a non-matching event
    app.world_mut().trigger(GameplayEvent {
        event_tag: GameplayTag::new("Event.Test.Other"),
        instigator: Some(owner),
        target: Some(owner),
        magnitude: None,
        target_data: None,
    });
    app.update();

    let state = app.world().get::<TaskState>(task).copied();
    assert_eq!(
        state,
        Some(TaskState::Running),
        "Task should still be running after non-matching event"
    );

    // Trigger the matching event
    app.world_mut().trigger(GameplayEvent {
        event_tag: event_tag.clone(),
        instigator: Some(owner),
        target: Some(owner),
        magnitude: None,
        target_data: None,
    });
    app.update();

    // Check that TaskCompletedEvent was triggered
    {
        let events = app.world().resource::<TaskEvents>();
        let completed = events.completed.lock().unwrap();
        assert!(
            completed.contains(&task),
            "TaskCompletedEvent should be triggered"
        );
    }

    // Task should be despawned by cleanup system in the same update
    assert!(
        app.world().get_entity(task).is_err(),
        "Task should be despawned after completion"
    );
}

#[test]
fn test_wait_gameplay_event_task_filters_by_target() {
    let mut app = setup_test_app();

    let owner = app.world_mut().spawn_empty().id();
    let other_entity = app.world_mut().spawn_empty().id();
    let spec = app
        .world_mut()
        .spawn(AbilitySpec::new("test_ability", 1))
        .id();

    let event_tag = GameplayTag::new("Event.Test.Trigger");

    // Spawn a WaitGameplayEvent task
    let task = app
        .world_mut()
        .spawn((
            AbilityTask {
                ability_instance: None,
                ability_spec: spec,
                owner,
            },
            WaitGameplayEventTask::new(event_tag.clone()),
            TaskState::Running,
        ))
        .id();

    // Trigger event targeting a different entity
    app.world_mut().trigger(GameplayEvent {
        event_tag: event_tag.clone(),
        instigator: Some(owner),
        target: Some(other_entity),
        magnitude: None,
        target_data: None,
    });
    app.update();

    let state = app.world().get::<TaskState>(task).copied();
    assert_eq!(
        state,
        Some(TaskState::Running),
        "Task should still be running when event targets different entity"
    );

    // Trigger event targeting the owner
    app.world_mut().trigger(GameplayEvent {
        event_tag: event_tag.clone(),
        instigator: Some(owner),
        target: Some(owner),
        magnitude: None,
        target_data: None,
    });
    app.update();

    // Check that task completed
    {
        let events = app.world().resource::<TaskEvents>();
        let completed = events.completed.lock().unwrap();
        assert!(
            completed.contains(&task),
            "Task should complete when event targets owner"
        );
    }
}

#[test]
fn test_wait_gameplay_event_task_accepts_event_without_target() {
    let mut app = setup_test_app();

    let owner = app.world_mut().spawn_empty().id();
    let spec = app
        .world_mut()
        .spawn(AbilitySpec::new("test_ability", 1))
        .id();

    let event_tag = GameplayTag::new("Event.Test.Trigger");

    // Spawn a WaitGameplayEvent task
    let task = app
        .world_mut()
        .spawn((
            AbilityTask {
                ability_instance: None,
                ability_spec: spec,
                owner,
            },
            WaitGameplayEventTask::new(event_tag.clone()),
            TaskState::Running,
        ))
        .id();

    // Trigger event without target (broadcast event)
    app.world_mut().trigger(GameplayEvent {
        event_tag: event_tag.clone(),
        instigator: Some(owner),
        target: None,
        magnitude: None,
        target_data: None,
    });
    app.update();

    // Check that task completed
    {
        let events = app.world().resource::<TaskEvents>();
        let completed = events.completed.lock().unwrap();
        assert!(
            completed.contains(&task),
            "Task should complete for broadcast event (no target)"
        );
    }
}

#[test]
fn test_task_cancelled_when_ability_instance_removed() {
    let mut app = setup_test_app();

    let owner = app.world_mut().spawn_empty().id();
    let spec = app
        .world_mut()
        .spawn(AbilitySpec::new("test_ability", 1))
        .id();

    // Spawn an ability instance
    let instance = app
        .world_mut()
        .spawn(
            bevy_gameplay_ability_system::abilities::AbilitySpecInstance {
                definition_id: "test_ability".into(),
                level: 1,
                behavior: None,
                owner,
                instigator: Some(owner),
                target_data: None,
            },
        )
        .id();

    // Spawn a task associated with the instance
    let task = app
        .world_mut()
        .spawn((
            AbilityTask {
                ability_instance: Some(instance),
                ability_spec: spec,
                owner,
            },
            WaitDelayTask::new(10.0),
            TaskState::Running,
        ))
        .id();

    // Remove the ability instance
    app.world_mut().despawn(instance);
    app.update();

    // Check that TaskCancelledEvent was triggered
    {
        let events = app.world().resource::<TaskEvents>();
        let cancelled = events.cancelled.lock().unwrap();
        assert!(
            cancelled.contains(&task),
            "TaskCancelledEvent should be triggered"
        );
    }

    // Task should be despawned by cleanup system in the same update
    assert!(
        app.world().get_entity(task).is_err(),
        "Task should be despawned after cancellation"
    );
}
