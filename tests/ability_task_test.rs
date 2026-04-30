//! Integration tests for ability tasks.
//!
//! Tests WaitDelay and WaitGameplayEvent task behavior.

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use bevy_gameplay_ability_system::{
    GasPlugin,
    abilities::{
        AbilityActiveState, AbilityDefinition, AbilityOwner, AbilityRegistry, AbilitySpec,
        AbilitySpecInstance, AbilityTask, ApplyEffectToTargetDataTask, AttributeComparison,
        GameplayAbilityTargetData, GameplayEvent, InputAction, InputPressedEvent, OverlapEvent,
        TaskCancelledEvent, TaskCompletedEvent, TaskState, TryActivateAbilityEvent,
        WaitAttributeChangeTask, WaitDelayTask, WaitEffectAppliedTask, WaitEffectRemovedTask,
        WaitGameplayEventTask, WaitInputPressTask, WaitOverlapTask, WaitTargetDataTask,
    },
    attributes::{AttributeData, AttributeMetadata, AttributeName, AttributeSetDefinition},
    effects::{
        ApplyGameplayEffectEvent, DurationPolicy, GameplayEffectDefinition, GameplayEffectRegistry,
        MagnitudeCalculation, ModifierInfo, ModifierOperation,
    },
};
use bevy_gameplay_tag::{GameplayTag, GameplayTagsManager, GameplayTagsPlugin};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use string_cache::DefaultAtom as Atom;

struct TestAttributeSet;

impl AttributeSetDefinition for TestAttributeSet {
    fn attribute_names() -> &'static [&'static str] {
        &["Health", "AttackPower"]
    }

    fn attribute_metadata(name: &str) -> Option<AttributeMetadata> {
        match name {
            "Health" => Some(
                AttributeMetadata::new("Health")
                    .with_min(0.0)
                    .with_max(1000.0),
            ),
            "AttackPower" => Some(AttributeMetadata::new("AttackPower").with_min(0.0)),
            _ => None,
        }
    }

    fn default_value(name: &str) -> f32 {
        match name {
            "Health" => 100.0,
            "AttackPower" => 10.0,
            _ => 0.0,
        }
    }
}

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
fn test_wait_attribute_change_task() {
    let mut app = setup_test_app();

    // Create player with Health attribute
    let player = app
        .world_mut()
        .spawn((
            Name::new("Player"),
            bevy_gameplay_ability_system::core::OwnedTags::default(),
            bevy_gameplay_ability_system::core::BlockedAbilityTags::default(),
        ))
        .id();

    app.world_mut()
        .run_system_once(move |mut commands: Commands| {
            TestAttributeSet::create_attributes(&mut commands, player);
        });
    app.update();

    // Register and grant ability
    let ability_id = Atom::from("test_ability");
    let ability_id_clone = ability_id.clone();
    app.world_mut()
        .run_system_once(move |mut registry: ResMut<AbilityRegistry>| {
            let def = AbilityDefinition::new(ability_id.clone());
            registry.register(def);
        });

    app.world_mut()
        .run_system_once(move |mut commands: Commands| {
            commands.spawn((
                AbilitySpec::new(ability_id_clone.clone(), 1),
                AbilityOwner(player),
                AbilityActiveState::default(),
            ));
        });
    app.update();

    // Get ability spec
    let ability_spec = {
        let mut query = app
            .world_mut()
            .query_filtered::<Entity, With<AbilitySpec>>();
        query.iter(app.world()).next().unwrap()
    };

    // Activate ability
    app.world_mut()
        .run_system_once(move |mut commands: Commands| {
            commands.trigger(TryActivateAbilityEvent::new(ability_spec, player));
        });
    app.update();

    // Get ability instance
    let ability_instance = {
        let mut query = app
            .world_mut()
            .query_filtered::<Entity, With<AbilitySpecInstance>>();
        query.iter(app.world()).next().unwrap()
    };

    // Spawn WaitAttributeChange task (wait for Health > 150)
    let task = app
        .world_mut()
        .spawn((
            AbilityTask {
                ability_instance: Some(ability_instance),
                ability_spec,
                owner: player,
            },
            WaitAttributeChangeTask::new("Health", AttributeComparison::GreaterThan, 150.0),
            TaskState::Running,
        ))
        .id();

    app.update();

    // Task should still be running (Health is 100)
    let state = app.world().get::<TaskState>(task).unwrap();
    assert_eq!(*state, TaskState::Running);

    // Increase Health to 200
    app.world_mut()
        .run_system_once(move |mut attributes: Query<&mut AttributeData>| {
            for mut attr in attributes.iter_mut() {
                attr.set_base_value(200.0);
            }
        });
    app.update();

    // Task should complete and be despawned
    {
        let events = app.world().resource::<TaskEvents>();
        let completed = events.completed.lock().unwrap();
        assert!(
            completed.contains(&task),
            "TaskCompletedEvent should be triggered"
        );
    }

    // Task should be despawned by cleanup system
    assert!(
        app.world().get_entity(task).is_err(),
        "Task should be despawned after completion"
    );
}

#[test]
fn test_wait_effect_applied_task() {
    let mut app = setup_test_app();

    // Create player with required components
    let player = app
        .world_mut()
        .spawn((
            Name::new("Player"),
            bevy_gameplay_ability_system::core::OwnedTags::default(),
            bevy_gameplay_ability_system::core::BlockedAbilityTags::default(),
        ))
        .id();

    app.world_mut().run_system_once(
        move |mut commands: Commands, tags: Res<GameplayTagsManager>| {
            TestAttributeSet::create_attributes(&mut commands, player);
        },
    );
    app.update();

    // Register effect
    let effect_id = Atom::from("test_effect");
    let effect_id_clone1 = effect_id.clone();
    let effect_id_clone2 = effect_id.clone();
    app.world_mut()
        .run_system_once(move |mut registry: ResMut<GameplayEffectRegistry>| {
            let def = GameplayEffectDefinition::new(effect_id.clone())
                .with_duration_policy(DurationPolicy::HasDuration)
                .with_duration(5.0);
            registry.register(def);
        });

    // Register and grant ability
    let ability_id = Atom::from("test_ability");
    let ability_id_clone = ability_id.clone();
    app.world_mut()
        .run_system_once(move |mut registry: ResMut<AbilityRegistry>| {
            let def = AbilityDefinition::new(ability_id.clone());
            registry.register(def);
        });

    app.world_mut()
        .run_system_once(move |mut commands: Commands| {
            commands.spawn((
                AbilitySpec::new(ability_id_clone.clone(), 1),
                AbilityOwner(player),
                AbilityActiveState::default(),
            ));
        });
    app.update();

    // Get ability spec and activate
    let ability_spec = {
        let mut query = app
            .world_mut()
            .query_filtered::<Entity, With<AbilitySpec>>();
        query.iter(app.world()).next().unwrap()
    };

    app.world_mut()
        .run_system_once(move |mut commands: Commands| {
            commands.trigger(TryActivateAbilityEvent::new(ability_spec, player));
        });
    app.update();

    let ability_instance = {
        let mut query = app
            .world_mut()
            .query_filtered::<Entity, With<AbilitySpecInstance>>();
        query.iter(app.world()).next().unwrap()
    };

    // Spawn WaitEffectApplied task
    let task = app
        .world_mut()
        .spawn((
            AbilityTask {
                ability_instance: Some(ability_instance),
                ability_spec,
                owner: player,
            },
            WaitEffectAppliedTask::for_effect(effect_id_clone1.as_ref()),
            TaskState::Running,
        ))
        .id();

    app.update();

    // Task should still be running
    let state = app.world().get::<TaskState>(task).unwrap();
    assert_eq!(*state, TaskState::Running);

    // Apply effect
    app.world_mut()
        .run_system_once(move |mut commands: Commands| {
            commands.trigger(
                ApplyGameplayEffectEvent::new(effect_id_clone2.clone(), player)
                    .with_instigator(player)
                    .with_level(1),
            );
        });
    app.update();

    // Task should complete and be despawned
    {
        let events = app.world().resource::<TaskEvents>();
        let completed = events.completed.lock().unwrap();
        assert!(
            completed.contains(&task),
            "TaskCompletedEvent should be triggered"
        );
    }

    // Task should be despawned by cleanup system
    assert!(
        app.world().get_entity(task).is_err(),
        "Task should be despawned after completion"
    );
}

#[test]
fn test_wait_effect_removed_task() {
    let mut app = setup_test_app();

    // Create player with required components
    let player = app
        .world_mut()
        .spawn((
            Name::new("Player"),
            bevy_gameplay_ability_system::core::OwnedTags::default(),
            bevy_gameplay_ability_system::core::BlockedAbilityTags::default(),
        ))
        .id();

    app.world_mut().run_system_once(
        move |mut commands: Commands, tags: Res<GameplayTagsManager>| {
            TestAttributeSet::create_attributes(&mut commands, player);
        },
    );
    app.update();

    // Register effect
    let effect_id = Atom::from("test_effect");
    let effect_id_clone1 = effect_id.clone();
    let effect_id_clone2 = effect_id.clone();
    app.world_mut()
        .run_system_once(move |mut registry: ResMut<GameplayEffectRegistry>| {
            let def = GameplayEffectDefinition::new(effect_id.clone())
                .with_duration_policy(DurationPolicy::HasDuration)
                .with_duration(1.0);
            registry.register(def);
        });

    // Register and grant ability FIRST
    let ability_id = Atom::from("test_ability");
    let ability_id_clone = ability_id.clone();
    app.world_mut()
        .run_system_once(move |mut registry: ResMut<AbilityRegistry>| {
            let def = AbilityDefinition::new(ability_id.clone());
            registry.register(def);
        });

    app.world_mut()
        .run_system_once(move |mut commands: Commands| {
            commands.spawn((
                AbilitySpec::new(ability_id_clone.clone(), 1),
                AbilityOwner(player),
                AbilityActiveState::default(),
            ));
        });
    app.update();

    let ability_spec = {
        let mut query = app
            .world_mut()
            .query_filtered::<Entity, With<AbilitySpec>>();
        query.iter(app.world()).next().unwrap()
    };

    app.world_mut()
        .run_system_once(move |mut commands: Commands| {
            commands.trigger(TryActivateAbilityEvent::new(ability_spec, player));
        });
    app.update();

    let ability_instance = {
        let mut query = app
            .world_mut()
            .query_filtered::<Entity, With<AbilitySpecInstance>>();
        query.iter(app.world()).next().unwrap()
    };

    // Spawn WaitEffectRemoved task BEFORE applying effect
    let task = app
        .world_mut()
        .spawn((
            AbilityTask {
                ability_instance: Some(ability_instance),
                ability_spec,
                owner: player,
            },
            WaitEffectRemovedTask::for_effect(effect_id_clone2.as_ref()),
            TaskState::Running,
        ))
        .id();

    app.update();

    // Now apply effect AFTER task is created
    app.world_mut()
        .run_system_once(move |mut commands: Commands| {
            commands.trigger(
                ApplyGameplayEffectEvent::new(effect_id_clone1.clone(), player)
                    .with_instigator(player)
                    .with_level(1),
            );
        });
    app.update();

    // Need another update for effect to be applied (observer triggers in Effects)
    app.update();

    // Task should still be running (effect is active)
    let state = app.world().get::<TaskState>(task).unwrap();
    assert_eq!(*state, TaskState::Running);

    // Wait for effect to expire (duration = 1.0s)
    // Manually update the effect duration since Time::advance_by doesn't affect delta_secs
    {
        let mut query = app
            .world_mut()
            .query::<&mut bevy_gameplay_ability_system::effects::EffectDuration>();
        for mut duration in query.iter_mut(app.world_mut()) {
            duration.tick(1.1);  // Manually tick the duration
        }
    }

    // Update to process the expired effect
    app.update();

    // Task should complete and be despawned
    {
        let events = app.world().resource::<TaskEvents>();
        let completed = events.completed.lock().unwrap();
        assert!(
            completed.contains(&task),
            "TaskCompletedEvent should be triggered"
        );
    }

    // Task should be despawned by cleanup system
    assert!(
        app.world().get_entity(task).is_err(),
        "Task should be despawned after completion"
    );
}

#[test]
fn test_apply_effect_to_target_data_task() {
    let mut app = setup_test_app();

    // Create player and enemy with required components
    let player = app
        .world_mut()
        .spawn((
            Name::new("Player"),
            bevy_gameplay_ability_system::core::OwnedTags::default(),
            bevy_gameplay_ability_system::core::BlockedAbilityTags::default(),
        ))
        .id();

    let enemy = app
        .world_mut()
        .spawn((
            Name::new("Enemy"),
            bevy_gameplay_ability_system::core::OwnedTags::default(),
        ))
        .id();

    app.world_mut().run_system_once(
        move |mut commands: Commands, tags: Res<GameplayTagsManager>| {
            TestAttributeSet::create_attributes(&mut commands, player);
            TestAttributeSet::create_attributes(&mut commands, enemy);
        },
    );
    app.update();

    // Register damage effect
    let effect_id = Atom::from("damage_effect");
    let effect_id_clone = effect_id.clone();
    app.world_mut()
        .run_system_once(move |mut registry: ResMut<GameplayEffectRegistry>| {
            let def = GameplayEffectDefinition::new(effect_id.clone())
                .with_duration_policy(DurationPolicy::Instant)
                .add_modifier(ModifierInfo::new(
                    "Health",
                    ModifierOperation::AddCurrent,
                    MagnitudeCalculation::scalar(-50.0),
                ));
            registry.register(def);
        });

    // Register and grant ability
    let ability_id = Atom::from("test_ability");
    let ability_id_clone = ability_id.clone();
    app.world_mut()
        .run_system_once(move |mut registry: ResMut<AbilityRegistry>| {
            let def = AbilityDefinition::new(ability_id.clone());
            registry.register(def);
        });

    app.world_mut()
        .run_system_once(move |mut commands: Commands| {
            commands.spawn((
                AbilitySpec::new(ability_id_clone.clone(), 1),
                AbilityOwner(player),
                AbilityActiveState::default(),
            ));
        });
    app.update();

    let ability_spec = {
        let mut query = app
            .world_mut()
            .query_filtered::<Entity, With<AbilitySpec>>();
        query.iter(app.world()).next().unwrap()
    };

    app.world_mut()
        .run_system_once(move |mut commands: Commands| {
            commands.trigger(TryActivateAbilityEvent::new(ability_spec, player));
        });
    app.update();

    let ability_instance = {
        let mut query = app
            .world_mut()
            .query_filtered::<Entity, With<AbilitySpecInstance>>();
        query.iter(app.world()).next().unwrap()
    };

    // Create target data with enemy
    let target_data = GameplayAbilityTargetData {
        actors: vec![enemy],
        origin: Some(Transform::from_translation(Vec3::ZERO)),
        end_point: None,
    };

    // Spawn ApplyEffectToTargetData task
    let _task = app
        .world_mut()
        .spawn((
            AbilityTask {
                ability_instance: Some(ability_instance),
                ability_spec,
                owner: player,
            },
            ApplyEffectToTargetDataTask::new(effect_id_clone.clone(), target_data, 1),
            TaskState::Running,
        ))
        .id();

    app.update();

    // Need another update for effect to be applied (task triggers in Abilities, effect applies in Effects next frame)
    app.update();

    // Check enemy Health was reduced
    let enemy_health = {
        let mut query = app
            .world_mut()
            .query_filtered::<(&AttributeData, &AttributeName, &ChildOf), ()>();
        query
            .iter(app.world())
            .find(|(_, name, child_of)| name.as_str() == "Health" && child_of.parent() == enemy)
            .map(|(attr, _, _)| attr.current_value)
            .unwrap()
    };

    assert_eq!(enemy_health, 50.0); // 100 - 50
}

#[test]
fn test_wait_target_data_task() {
    let mut app = setup_test_app();

    let player = app
        .world_mut()
        .spawn((
            Name::new("Player"),
            bevy_gameplay_ability_system::core::OwnedTags::default(),
            bevy_gameplay_ability_system::core::BlockedAbilityTags::default(),
        ))
        .id();

    // Register and grant ability
    let ability_id = Atom::from("test_ability");
    let ability_id_clone = ability_id.clone();
    app.world_mut()
        .run_system_once(move |mut registry: ResMut<AbilityRegistry>| {
            let def = AbilityDefinition::new(ability_id.clone());
            registry.register(def);
        });

    app.world_mut()
        .run_system_once(move |mut commands: Commands| {
            commands.spawn((
                AbilitySpec::new(ability_id_clone.clone(), 1),
                AbilityOwner(player),
                AbilityActiveState::default(),
            ));
        });
    app.update();

    let ability_spec = {
        let mut query = app
            .world_mut()
            .query_filtered::<Entity, With<AbilitySpec>>();
        query.iter(app.world()).next().unwrap()
    };

    app.world_mut()
        .run_system_once(move |mut commands: Commands| {
            commands.trigger(TryActivateAbilityEvent::new(ability_spec, player));
        });
    app.update();

    let ability_instance = {
        let mut query = app
            .world_mut()
            .query_filtered::<Entity, With<AbilitySpecInstance>>();
        query.iter(app.world()).next().unwrap()
    };

    // Spawn WaitTargetData task
    let task = app
        .world_mut()
        .spawn((
            AbilityTask {
                ability_instance: Some(ability_instance),
                ability_spec,
                owner: player,
            },
            WaitTargetDataTask::new(),
            TaskState::Running,
        ))
        .id();

    app.update();

    // Task should still be running
    let state = app.world().get::<TaskState>(task).unwrap();
    assert_eq!(*state, TaskState::Running);

    // Provide target data
    let target_data = GameplayAbilityTargetData {
        actors: vec![player],
        origin: Some(Transform::from_translation(Vec3::ZERO)),
        end_point: Some(Transform::from_translation(Vec3::new(10.0, 0.0, 0.0))),
    };

    {
        let mut tasks = app.world_mut().query::<&mut WaitTargetDataTask>();
        if let Ok(mut wait_target) = tasks.get_mut(app.world_mut(), task) {
            wait_target.provide_target_data(target_data);
        }
    }
    app.update();

    // Task should complete and be despawned
    {
        let events = app.world().resource::<TaskEvents>();
        let completed = events.completed.lock().unwrap();
        assert!(
            completed.contains(&task),
            "TaskCompletedEvent should be triggered"
        );
    }

    // Task should be despawned by cleanup system
    assert!(
        app.world().get_entity(task).is_err(),
        "Task should be despawned after completion"
    );
}

#[test]
fn test_wait_input_press_task() {
    let mut app = setup_test_app();

    let player = app
        .world_mut()
        .spawn((
            Name::new("Player"),
            bevy_gameplay_ability_system::core::OwnedTags::default(),
            bevy_gameplay_ability_system::core::BlockedAbilityTags::default(),
        ))
        .id();

    // Register and grant ability
    let ability_id = Atom::from("test_ability");
    let ability_id_clone = ability_id.clone();
    app.world_mut()
        .run_system_once(move |mut registry: ResMut<AbilityRegistry>| {
            let def = AbilityDefinition::new(ability_id.clone());
            registry.register(def);
        });

    app.world_mut()
        .run_system_once(move |mut commands: Commands| {
            commands.spawn((
                AbilitySpec::new(ability_id_clone.clone(), 1),
                AbilityOwner(player),
                AbilityActiveState::default(),
            ));
        });
    app.update();

    let ability_spec = {
        let mut query = app
            .world_mut()
            .query_filtered::<Entity, With<AbilitySpec>>();
        query.iter(app.world()).next().unwrap()
    };

    app.world_mut()
        .run_system_once(move |mut commands: Commands| {
            commands.trigger(TryActivateAbilityEvent::new(ability_spec, player));
        });
    app.update();

    let ability_instance = {
        let mut query = app
            .world_mut()
            .query_filtered::<Entity, With<AbilitySpecInstance>>();
        query.iter(app.world()).next().unwrap()
    };

    // Spawn WaitInputPress task
    let task = app
        .world_mut()
        .spawn((
            AbilityTask {
                ability_instance: Some(ability_instance),
                ability_spec,
                owner: player,
            },
            WaitInputPressTask::confirm(),
            TaskState::Running,
        ))
        .id();

    app.update();

    // Task should still be running
    let state = app.world().get::<TaskState>(task).unwrap();
    assert_eq!(*state, TaskState::Running);

    // Trigger input pressed event
    app.world_mut()
        .run_system_once(move |mut commands: Commands| {
            commands.trigger(InputPressedEvent {
                entity: player,
                action: InputAction::Confirm,
            });
        });
    app.update();

    // Task should complete and be despawned
    {
        let events = app.world().resource::<TaskEvents>();
        let completed = events.completed.lock().unwrap();
        assert!(
            completed.contains(&task),
            "TaskCompletedEvent should be triggered"
        );
    }

    // Task should be despawned by cleanup system
    assert!(
        app.world().get_entity(task).is_err(),
        "Task should be despawned after completion"
    );
}

#[test]
fn test_wait_overlap_task() {
    let mut app = setup_test_app();

    let player = app
        .world_mut()
        .spawn((
            Name::new("Player"),
            bevy_gameplay_ability_system::core::OwnedTags::default(),
            bevy_gameplay_ability_system::core::BlockedAbilityTags::default(),
        ))
        .id();

    let enemy = app.world_mut().spawn(Name::new("Enemy")).id();

    // Register and grant ability
    let ability_id = Atom::from("test_ability");
    let ability_id_clone = ability_id.clone();
    app.world_mut()
        .run_system_once(move |mut registry: ResMut<AbilityRegistry>| {
            let def = AbilityDefinition::new(ability_id.clone());
            registry.register(def);
        });

    app.world_mut()
        .run_system_once(move |mut commands: Commands| {
            commands.spawn((
                AbilitySpec::new(ability_id_clone.clone(), 1),
                AbilityOwner(player),
                AbilityActiveState::default(),
            ));
        });
    app.update();

    let ability_spec = {
        let mut query = app
            .world_mut()
            .query_filtered::<Entity, With<AbilitySpec>>();
        query.iter(app.world()).next().unwrap()
    };

    app.world_mut()
        .run_system_once(move |mut commands: Commands| {
            commands.trigger(TryActivateAbilityEvent::new(ability_spec, player));
        });
    app.update();

    let ability_instance = {
        let mut query = app
            .world_mut()
            .query_filtered::<Entity, With<AbilitySpecInstance>>();
        query.iter(app.world()).next().unwrap()
    };

    // Spawn WaitOverlap task
    let task = app
        .world_mut()
        .spawn((
            AbilityTask {
                ability_instance: Some(ability_instance),
                ability_spec,
                owner: player,
            },
            WaitOverlapTask::new(),
            TaskState::Running,
        ))
        .id();

    app.update();

    // Task should still be running
    let state = app.world().get::<TaskState>(task).unwrap();
    assert_eq!(*state, TaskState::Running);

    // Trigger overlap event
    app.world_mut()
        .run_system_once(move |mut commands: Commands| {
            commands.trigger(OverlapEvent {
                entity_a: player,
                entity_b: enemy,
                component_type: None,
            });
        });
    app.update();

    // Task should complete and be despawned
    {
        let events = app.world().resource::<TaskEvents>();
        let completed = events.completed.lock().unwrap();
        assert!(
            completed.contains(&task),
            "TaskCompletedEvent should be triggered"
        );
    }

    // Task should be despawned by cleanup system
    assert!(
        app.world().get_entity(task).is_err(),
        "Task should be despawned after completion"
    );
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
