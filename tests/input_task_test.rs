//! Integration tests for input-related ability tasks.
//!
//! Tests WaitInputPress and WaitInputRelease task behavior.

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use bevy_gameplay_ability_system::{
    GasPlugin,
    abilities::{
        AbilityActiveState, AbilityDefinition, AbilityOwner, AbilityRegistry, AbilitySpec,
        AbilitySpecInstance, AbilityTask, InputAction, InputPressedEvent, InputReleasedEvent,
        TaskCompletedEvent, TaskState, TryActivateAbilityEvent, WaitInputPressTask,
        WaitInputReleaseTask,
    },
};
use bevy_gameplay_tag::GameplayTagsPlugin;
use std::sync::{Arc, Mutex};

/// Helper to capture task events.
#[derive(Resource, Default)]
struct TaskEvents {
    completed: Arc<Mutex<Vec<Entity>>>,
}

fn setup_test_app() -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        GameplayTagsPlugin::with_data_path("assets/gameplay_tags.json".to_string()),
        GasPlugin,
    ))
    .insert_resource(TaskEvents::default())
    .add_observer(capture_task_completed);

    // Update once to initialize the tags manager
    app.update();
    app
}

fn capture_task_completed(trigger: On<TaskCompletedEvent>, events: Res<TaskEvents>) {
    let event = trigger.event();
    events.completed.lock().unwrap().push(event.task);
}

#[test]
fn test_wait_input_press_task_completes_on_input() {
    let mut app = setup_test_app();

    let owner = app.world_mut().spawn_empty().id();
    let spec = app
        .world_mut()
        .spawn(AbilitySpec::new("test_ability", 1))
        .id();

    // Spawn a WaitInputPress task waiting for Confirm action
    let task = app
        .world_mut()
        .spawn((
            AbilityTask {
                ability_instance: None,
                ability_spec: spec,
                owner,
            },
            WaitInputPressTask::confirm(),
            TaskState::Running,
        ))
        .id();

    // Update once to process systems
    app.update();

    // Verify task is still running
    let state = app.world().get::<TaskState>(task).unwrap();
    assert_eq!(*state, TaskState::Running);

    // Trigger input pressed event
    app.world_mut().trigger(InputPressedEvent {
        entity: owner,
        action: InputAction::Confirm,
    });

    // Update to process the event
    app.update();

    // Verify completion event was triggered
    let events = app.world().resource::<TaskEvents>();
    let completed = events.completed.lock().unwrap();
    assert_eq!(completed.len(), 1);
    assert_eq!(completed[0], task);

    // Task should be despawned after completion
    assert!(app.world().get_entity(task).is_err());
}

#[test]
fn test_wait_input_press_task_ignores_wrong_action() {
    let mut app = setup_test_app();

    let owner = app.world_mut().spawn_empty().id();
    let spec = app
        .world_mut()
        .spawn(AbilitySpec::new("test_ability", 1))
        .id();

    // Spawn a WaitInputPress task waiting for Confirm action
    let task = app
        .world_mut()
        .spawn((
            AbilityTask {
                ability_instance: None,
                ability_spec: spec,
                owner,
            },
            WaitInputPressTask::confirm(),
            TaskState::Running,
        ))
        .id();

    // Trigger wrong input action (Cancel instead of Confirm)
    app.world_mut().trigger(InputPressedEvent {
        entity: owner,
        action: InputAction::Cancel,
    });

    app.update();

    // Verify task is still running
    let state = app.world().get::<TaskState>(task).unwrap();
    assert_eq!(*state, TaskState::Running);

    // Verify no completion event
    let events = app.world().resource::<TaskEvents>();
    let completed = events.completed.lock().unwrap();
    assert_eq!(completed.len(), 0);
}

#[test]
fn test_wait_input_release_task_completes_on_release() {
    let mut app = setup_test_app();

    let owner = app.world_mut().spawn_empty().id();
    let spec = app
        .world_mut()
        .spawn(AbilitySpec::new("test_ability", 1))
        .id();

    let press_time = 1.0;

    // Spawn a WaitInputRelease task waiting for Confirm action
    let task = app
        .world_mut()
        .spawn((
            AbilityTask {
                ability_instance: None,
                ability_spec: spec,
                owner,
            },
            WaitInputReleaseTask::confirm(press_time),
            TaskState::Running,
        ))
        .id();

    // Update once to process systems
    app.update();

    // Verify task is still running
    let state = app.world().get::<TaskState>(task).unwrap();
    assert_eq!(*state, TaskState::Running);

    // Trigger input released event
    let release_time = 2.5;
    app.world_mut().trigger(InputReleasedEvent {
        entity: owner,
        action: InputAction::Confirm,
        release_time,
    });

    // Update to process the event
    app.update();

    // Verify completion event was triggered
    let events = app.world().resource::<TaskEvents>();
    let completed = events.completed.lock().unwrap();
    assert_eq!(completed.len(), 1);
    assert_eq!(completed[0], task);

    // Task should be despawned after completion
    assert!(app.world().get_entity(task).is_err());
}

#[test]
fn test_wait_input_release_task_ignores_wrong_entity() {
    let mut app = setup_test_app();

    let owner = app.world_mut().spawn_empty().id();
    let other_entity = app.world_mut().spawn_empty().id();
    let spec = app
        .world_mut()
        .spawn(AbilitySpec::new("test_ability", 1))
        .id();

    // Spawn a WaitInputRelease task
    let task = app
        .world_mut()
        .spawn((
            AbilityTask {
                ability_instance: None,
                ability_spec: spec,
                owner,
            },
            WaitInputReleaseTask::confirm(1.0),
            TaskState::Running,
        ))
        .id();

    // Trigger input released event from wrong entity
    app.world_mut().trigger(InputReleasedEvent {
        entity: other_entity,
        action: InputAction::Confirm,
        release_time: 2.0,
    });

    app.update();

    // Verify task is still running
    let state = app.world().get::<TaskState>(task).unwrap();
    assert_eq!(*state, TaskState::Running);

    // Verify no completion event
    let events = app.world().resource::<TaskEvents>();
    let completed = events.completed.lock().unwrap();
    assert_eq!(completed.len(), 0);
}
