//! Integration tests for the ability activation flow.
//!
//! Tests the complete lifecycle: TryActivate → Activated → Commit → End → Ended

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use bevy_gameplay_ability_system::prelude::*;
use bevy_gameplay_tag::gameplay_tag::GameplayTag;
use bevy_gameplay_tag::gameplay_tag_count_container::GameplayTagCountContainer;
use bevy_gameplay_tag::{GameplayTagsManager, GameplayTagsPlugin};
use std::sync::{Arc, Mutex};
use string_cache::DefaultAtom as Atom;

/// Shared state for capturing events across observers.
#[derive(Resource, Clone, Default)]
struct TestEvents {
    activated: Arc<Mutex<Vec<AbilityActivatedEvent>>>,
    failed: Arc<Mutex<Vec<AbilityActivationFailedEvent>>>,
    commit_results: Arc<Mutex<Vec<CommitAbilityResultEvent>>>,
    ended: Arc<Mutex<Vec<AbilityEndedEvent>>>,
}

fn setup_app() -> App {
    let mut app = App::new();
    app.add_plugins(bevy::time::TimePlugin);
    app.add_plugins(GameplayTagsPlugin::with_data_path(
        "assets/gameplay_tags.json".to_string(),
    ));
    app.add_plugins(AttributePlugin);
    app.add_plugins(EffectPlugin);
    app.add_plugins(AbilityPlugin);
    app.init_resource::<TestEvents>();

    app.add_observer(
        |ev: On<AbilityActivatedEvent>, events: Res<TestEvents>| {
            events.activated.lock().unwrap().push(ev.event().clone());
        },
    );
    app.add_observer(
        |ev: On<AbilityActivationFailedEvent>, events: Res<TestEvents>| {
            events.failed.lock().unwrap().push(ev.event().clone());
        },
    );
    app.add_observer(
        |ev: On<CommitAbilityResultEvent>, events: Res<TestEvents>| {
            events.commit_results.lock().unwrap().push(ev.event().clone());
        },
    );
    app.add_observer(
        |ev: On<AbilityEndedEvent>, events: Res<TestEvents>| {
            events.ended.lock().unwrap().push(ev.event().clone());
        },
    );

    app.update();
    app
}

fn register_cost_and_cooldown(app: &mut App) {
    let cost = GameplayEffectDefinition::new("effect.cost.mana")
        .with_duration_policy(DurationPolicy::Instant)
        .add_modifier(ModifierInfo {
            attribute_name: Atom::from("Mana"),
            operation: ModifierOperation::AddBase,
            magnitude: MagnitudeCalculation::ScalableFloat { base_value: -20.0 },
        });

    app.world_mut()
        .resource_mut::<GameplayEffectRegistry>()
        .register(cost);

    app.world_mut()
        .run_system_once(
            |tags_manager: Res<GameplayTagsManager>,
             mut registry: ResMut<GameplayEffectRegistry>| {
                let cooldown = GameplayEffectDefinition::new("effect.cooldown.fireball")
                    .with_duration_policy(DurationPolicy::HasDuration)
                    .with_duration(3.0)
                    .grant_tag(GameplayTag::new("Cooldown.Fireball"), &tags_manager);
                registry.register(cooldown);
            },
        )
        .expect("System should run");
}

fn spawn_owner_with_mana(app: &mut App, mana: f32) -> Entity {
    let owner = app
        .world_mut()
        .spawn(GameplayTagCountContainer::default())
        .id();

    app.world_mut().spawn((
        AttributeData {
            base_value: mana,
            current_value: mana,
        },
        AttributeName::new("Mana"),
        AttributeOwner(owner),
    ));

    owner
}

fn spawn_ability_spec(app: &mut App, owner: Entity, definition_id: &str) -> Entity {
    app.world_mut()
        .spawn((
            AbilitySpec::new(definition_id.to_string(), 1),
            AbilityOwner(owner),
            AbilityState::Ready,
        ))
        .id()
}

fn add_tag_to_owner(app: &mut App, owner: Entity, tag_name: &str) {
    let tag_name = tag_name.to_string();
    app.world_mut()
        .run_system_once(
            move |tags_manager: Res<GameplayTagsManager>,
                  mut tag_containers: Query<(Entity, &mut GameplayTagCountContainer)>,
                  mut commands: Commands| {
                for (entity, mut container) in tag_containers.iter_mut() {
                    if entity == owner {
                        let mut tc = bevy_gameplay_tag::GameplayTagContainer::default();
                        tc.add_tag(GameplayTag::new(&tag_name), &tags_manager);
                        container.update_tag_container_count(
                            &tc,
                            1,
                            &tags_manager,
                            &mut commands,
                            entity,
                        );
                    }
                }
            },
        )
        .expect("System should run");
}

// =============================================================================
// Task 3: Full activation flow
// =============================================================================

#[test]
fn test_full_activation_flow() {
    let mut app = setup_app();

    {
        let mut registry = app.world_mut().resource_mut::<AbilityRegistry>();
        registry.register(AbilityDefinition::new("test.ability"));
    }

    let owner = app
        .world_mut()
        .spawn(GameplayTagCountContainer::default())
        .id();
    let spec_entity = spawn_ability_spec(&mut app, owner, "test.ability");

    app.add_observer(
        move |ev: On<AbilityActivatedEvent>, mut commands: Commands| {
            let event = ev.event();
            commands.trigger(CommitAbilityEvent {
                ability_spec: event.ability_spec,
                owner: event.owner,
            });
            commands.trigger(EndAbilityEvent {
                ability_spec: event.ability_spec,
                owner: event.owner,
            });
        },
    );

    app.world_mut().trigger(TryActivateAbilityEvent {
        ability_spec: spec_entity,
        owner,
    });
    app.update();

    let events = app.world().resource::<TestEvents>();

    let activated = events.activated.lock().unwrap();
    assert_eq!(activated.len(), 1, "Expected 1 activated event");
    assert_eq!(activated[0].ability_spec, spec_entity);
    assert_eq!(activated[0].owner, owner);

    let commits = events.commit_results.lock().unwrap();
    assert_eq!(commits.len(), 1, "Expected 1 commit result event");
    assert!(commits[0].success, "Commit should succeed");

    let ended = events.ended.lock().unwrap();
    assert_eq!(ended.len(), 1, "Expected 1 ended event");
    assert!(!ended[0].was_cancelled, "Should not be cancelled");

    let spec = app.world().get::<AbilitySpec>(spec_entity).unwrap();
    assert!(!spec.is_active);
    assert_eq!(spec.active_count, 0);
}

#[test]
fn test_activation_flow_with_cost_and_cooldown() {
    let mut app = setup_app();
    register_cost_and_cooldown(&mut app);

    {
        let mut registry = app.world_mut().resource_mut::<AbilityRegistry>();
        registry.register(
            AbilityDefinition::new("test.fireball")
                .with_cost_effect("effect.cost.mana")
                .with_cooldown_effect("effect.cooldown.fireball"),
        );
    }

    let owner = spawn_owner_with_mana(&mut app, 100.0);
    let spec_entity = spawn_ability_spec(&mut app, owner, "test.fireball");

    app.add_observer(
        move |ev: On<AbilityActivatedEvent>, mut commands: Commands| {
            let event = ev.event();
            if event.ability_spec == spec_entity {
                commands.trigger(CommitAbilityEvent {
                    ability_spec: event.ability_spec,
                    owner: event.owner,
                });
                commands.trigger(EndAbilityEvent {
                    ability_spec: event.ability_spec,
                    owner: event.owner,
                });
            }
        },
    );

    app.world_mut().trigger(TryActivateAbilityEvent {
        ability_spec: spec_entity,
        owner,
    });
    app.update();

    let events = app.world().resource::<TestEvents>();
    assert_eq!(events.activated.lock().unwrap().len(), 1);
    assert!(events.commit_results.lock().unwrap()[0].success);
    assert_eq!(events.ended.lock().unwrap().len(), 1);
    assert!(events.failed.lock().unwrap().is_empty());

    let mut found_mana = false;
    for (attr, name, attr_owner) in app
        .world_mut()
        .query::<(&AttributeData, &AttributeName, &AttributeOwner)>()
        .iter(app.world())
    {
        if attr_owner.0 == owner && name.as_str() == "Mana" {
            assert!(
                (attr.base_value - 80.0).abs() < f32::EPSILON,
                "Mana should be 80.0 after cost, got {}",
                attr.base_value
            );
            found_mana = true;
        }
    }
    assert!(found_mana, "Should have found Mana attribute");
}

// =============================================================================
// Task 4: Activation failure scenarios
// =============================================================================

#[test]
fn test_activation_fails_on_cooldown() {
    let mut app = setup_app();
    register_cost_and_cooldown(&mut app);

    {
        let mut registry = app.world_mut().resource_mut::<AbilityRegistry>();
        registry.register(
            AbilityDefinition::new("test.fireball")
                .with_cooldown_effect("effect.cooldown.fireball"),
        );
    }

    let owner = app
        .world_mut()
        .spawn(GameplayTagCountContainer::default())
        .id();

    app.world_mut().trigger(ApplyGameplayEffectEvent {
        effect_id: Atom::from("effect.cooldown.fireball"),
        target: owner,
        instigator: None,
        level: 1,
    });
    app.update();

    let spec_entity = spawn_ability_spec(&mut app, owner, "test.fireball");

    app.world_mut().trigger(TryActivateAbilityEvent {
        ability_spec: spec_entity,
        owner,
    });
    app.update();

    let events = app.world().resource::<TestEvents>();
    let failed = events.failed.lock().unwrap();
    assert_eq!(failed.len(), 1, "Expected 1 failure event");
    assert_eq!(failed[0].reason, ActivationFailureReason::OnCooldown);
    assert!(events.activated.lock().unwrap().is_empty());
}

#[test]
fn test_activation_fails_insufficient_cost() {
    let mut app = setup_app();

    let cost = GameplayEffectDefinition::new("effect.cost.expensive")
        .with_duration_policy(DurationPolicy::Instant)
        .add_modifier(ModifierInfo {
            attribute_name: Atom::from("Mana"),
            operation: ModifierOperation::AddBase,
            magnitude: MagnitudeCalculation::ScalableFloat { base_value: -50.0 },
        });
    app.world_mut()
        .resource_mut::<GameplayEffectRegistry>()
        .register(cost);

    {
        let mut registry = app.world_mut().resource_mut::<AbilityRegistry>();
        registry.register(
            AbilityDefinition::new("test.expensive")
                .with_cost_effect("effect.cost.expensive"),
        );
    }

    let owner = spawn_owner_with_mana(&mut app, 10.0);
    let spec_entity = spawn_ability_spec(&mut app, owner, "test.expensive");

    app.world_mut().trigger(TryActivateAbilityEvent {
        ability_spec: spec_entity,
        owner,
    });
    app.update();

    let events = app.world().resource::<TestEvents>();
    let failed = events.failed.lock().unwrap();
    assert_eq!(failed.len(), 1, "Expected 1 failure event");
    assert_eq!(failed[0].reason, ActivationFailureReason::InsufficientCost);
    assert!(events.activated.lock().unwrap().is_empty());
}

#[test]
fn test_activation_fails_missing_required_tags() {
    let mut app = setup_app();

    app.world_mut()
        .run_system_once(
            |tags_manager: Res<GameplayTagsManager>,
             mut registry: ResMut<AbilityRegistry>| {
                let ability = AbilityDefinition::new("test.requires_alive")
                    .add_activation_required_tag(
                        GameplayTag::new("State.Alive"),
                        &tags_manager,
                    );
                registry.register(ability);
            },
        )
        .expect("System should run");

    let owner = app
        .world_mut()
        .spawn(GameplayTagCountContainer::default())
        .id();
    let spec_entity = spawn_ability_spec(&mut app, owner, "test.requires_alive");

    app.world_mut().trigger(TryActivateAbilityEvent {
        ability_spec: spec_entity,
        owner,
    });
    app.update();

    let events = app.world().resource::<TestEvents>();
    let failed = events.failed.lock().unwrap();
    assert_eq!(failed.len(), 1, "Expected 1 failure event");
    assert_eq!(
        failed[0].reason,
        ActivationFailureReason::MissingRequiredTags
    );
    assert!(events.activated.lock().unwrap().is_empty());
}

#[test]
fn test_activation_fails_blocked_tags() {
    let mut app = setup_app();

    app.world_mut()
        .run_system_once(
            |tags_manager: Res<GameplayTagsManager>,
             mut registry: ResMut<AbilityRegistry>| {
                let ability = AbilityDefinition::new("test.blocked_by_stun")
                    .add_activation_blocked_tag(
                        GameplayTag::new("State.Stunned"),
                        &tags_manager,
                    );
                registry.register(ability);
            },
        )
        .expect("System should run");

    let owner = app
        .world_mut()
        .spawn(GameplayTagCountContainer::default())
        .id();

    add_tag_to_owner(&mut app, owner, "State.Stunned");

    let spec_entity = spawn_ability_spec(&mut app, owner, "test.blocked_by_stun");

    app.world_mut().trigger(TryActivateAbilityEvent {
        ability_spec: spec_entity,
        owner,
    });
    app.update();

    let events = app.world().resource::<TestEvents>();
    let failed = events.failed.lock().unwrap();
    assert_eq!(failed.len(), 1, "Expected 1 failure event");
    assert_eq!(failed[0].reason, ActivationFailureReason::BlockedByTags);
    assert!(events.activated.lock().unwrap().is_empty());
}

// =============================================================================
// Task 5: Tag management
// =============================================================================

#[test]
fn test_owned_tags_added_on_activate() {
    let mut app = setup_app();

    app.world_mut()
        .run_system_once(
            |tags_manager: Res<GameplayTagsManager>,
             mut registry: ResMut<AbilityRegistry>| {
                let ability = AbilityDefinition::new("test.casting")
                    .add_activation_owned_tag(
                        GameplayTag::new("Ability.Casting"),
                        &tags_manager,
                    );
                registry.register(ability);
            },
        )
        .expect("System should run");

    let owner = app
        .world_mut()
        .spawn(GameplayTagCountContainer::default())
        .id();
    let spec_entity = spawn_ability_spec(&mut app, owner, "test.casting");

    // Activate but do NOT end — leave it active
    app.world_mut().trigger(TryActivateAbilityEvent {
        ability_spec: spec_entity,
        owner,
    });
    app.update();

    let events = app.world().resource::<TestEvents>();
    assert_eq!(events.activated.lock().unwrap().len(), 1);

    let owner_tags = app.world().get::<GameplayTagCountContainer>(owner).unwrap();
    assert!(
        owner_tags.has_any_matching_gameplay_tags(
            &app.world()
                .resource::<AbilityRegistry>()
                .get("test.casting")
                .unwrap()
                .activation_owned_tags
        ),
        "Owner should have Ability.Casting tag while ability is active"
    );
}

#[test]
fn test_owned_tags_removed_on_end() {
    let mut app = setup_app();

    app.world_mut()
        .run_system_once(
            |tags_manager: Res<GameplayTagsManager>,
             mut registry: ResMut<AbilityRegistry>| {
                let ability = AbilityDefinition::new("test.casting")
                    .add_activation_owned_tag(
                        GameplayTag::new("Ability.Casting"),
                        &tags_manager,
                    );
                registry.register(ability);
            },
        )
        .expect("System should run");

    let owner = app
        .world_mut()
        .spawn(GameplayTagCountContainer::default())
        .id();
    let spec_entity = spawn_ability_spec(&mut app, owner, "test.casting");

    app.add_observer(
        move |ev: On<AbilityActivatedEvent>, mut commands: Commands| {
            let event = ev.event();
            if event.ability_spec == spec_entity {
                commands.trigger(CommitAbilityEvent {
                    ability_spec: event.ability_spec,
                    owner: event.owner,
                });
                commands.trigger(EndAbilityEvent {
                    ability_spec: event.ability_spec,
                    owner: event.owner,
                });
            }
        },
    );

    app.world_mut().trigger(TryActivateAbilityEvent {
        ability_spec: spec_entity,
        owner,
    });
    app.update();

    let events = app.world().resource::<TestEvents>();
    assert_eq!(events.ended.lock().unwrap().len(), 1);

    let owner_tags = app.world().get::<GameplayTagCountContainer>(owner).unwrap();
    assert!(
        !owner_tags.has_any_matching_gameplay_tags(
            &app.world()
                .resource::<AbilityRegistry>()
                .get("test.casting")
                .unwrap()
                .activation_owned_tags
        ),
        "Owner should NOT have Ability.Casting tag after ability ends"
    );
}

#[test]
fn test_block_tags_added_and_removed() {
    let mut app = setup_app();

    app.world_mut()
        .run_system_once(
            |tags_manager: Res<GameplayTagsManager>,
             mut registry: ResMut<AbilityRegistry>| {
                let ability = AbilityDefinition::new("test.blocker")
                    .add_block_abilities_with_tag(
                        GameplayTag::new("Ability.Casting"),
                        &tags_manager,
                    );
                registry.register(ability);
            },
        )
        .expect("System should run");

    let owner = app
        .world_mut()
        .spawn(GameplayTagCountContainer::default())
        .id();
    let spec_entity = spawn_ability_spec(&mut app, owner, "test.blocker");

    app.world_mut().trigger(TryActivateAbilityEvent {
        ability_spec: spec_entity,
        owner,
    });
    app.update();

    let events = app.world().resource::<TestEvents>();
    assert_eq!(events.activated.lock().unwrap().len(), 1);

    let owner_tags = app.world().get::<GameplayTagCountContainer>(owner).unwrap();
    assert!(
        owner_tags.has_any_matching_gameplay_tags(
            &app.world()
                .resource::<AbilityRegistry>()
                .get("test.blocker")
                .unwrap()
                .block_abilities_with_tags
        ),
        "Owner should have block tag while ability is active"
    );

    // End the ability
    app.world_mut().trigger(EndAbilityEvent {
        ability_spec: spec_entity,
        owner,
    });
    app.update();

    let owner_tags = app.world().get::<GameplayTagCountContainer>(owner).unwrap();
    assert!(
        !owner_tags.has_any_matching_gameplay_tags(
            &app.world()
                .resource::<AbilityRegistry>()
                .get("test.blocker")
                .unwrap()
                .block_abilities_with_tags
        ),
        "Owner should NOT have block tag after ability ends"
    );
}

// =============================================================================
// Task 6: Blocking and cancellation
// =============================================================================

#[test]
fn test_ability_blocks_another() {
    let mut app = setup_app();

    // Ability A: adds block tag Ability.Casting when active
    // Ability B: blocked by Ability.Casting
    app.world_mut()
        .run_system_once(
            |tags_manager: Res<GameplayTagsManager>,
             mut registry: ResMut<AbilityRegistry>| {
                let ability_a = AbilityDefinition::new("test.ability_a")
                    .add_block_abilities_with_tag(
                        GameplayTag::new("Ability.Casting"),
                        &tags_manager,
                    );
                let ability_b = AbilityDefinition::new("test.ability_b")
                    .add_activation_blocked_tag(
                        GameplayTag::new("Ability.Casting"),
                        &tags_manager,
                    );
                registry.register(ability_a);
                registry.register(ability_b);
            },
        )
        .expect("System should run");

    let owner = app
        .world_mut()
        .spawn(GameplayTagCountContainer::default())
        .id();
    let spec_a = spawn_ability_spec(&mut app, owner, "test.ability_a");
    let spec_b = spawn_ability_spec(&mut app, owner, "test.ability_b");

    // Activate A (stays active, adds block tag)
    app.world_mut().trigger(TryActivateAbilityEvent {
        ability_spec: spec_a,
        owner,
    });
    app.update();

    let events = app.world().resource::<TestEvents>();
    assert_eq!(events.activated.lock().unwrap().len(), 1);

    // Try to activate B — should be blocked
    app.world_mut().trigger(TryActivateAbilityEvent {
        ability_spec: spec_b,
        owner,
    });
    app.update();

    let events = app.world().resource::<TestEvents>();
    let failed = events.failed.lock().unwrap();
    assert_eq!(failed.len(), 1, "Expected 1 failure event");
    assert_eq!(failed[0].reason, ActivationFailureReason::BlockedByTags);
    assert_eq!(failed[0].ability_spec, spec_b);
}

#[test]
fn test_ability_cancels_another() {
    let mut app = setup_app();

    // Ability B: has ability_tags = Ability.Blocking
    // Ability A: cancel_abilities_with_tags = Ability.Blocking
    app.world_mut()
        .run_system_once(
            |tags_manager: Res<GameplayTagsManager>,
             mut registry: ResMut<AbilityRegistry>| {
                let ability_b = AbilityDefinition::new("test.block_ability")
                    .add_ability_tag(GameplayTag::new("Ability.Blocking"), &tags_manager);
                let ability_a = AbilityDefinition::new("test.attack_ability")
                    .add_cancel_abilities_with_tag(
                        GameplayTag::new("Ability.Blocking"),
                        &tags_manager,
                    );
                registry.register(ability_b);
                registry.register(ability_a);
            },
        )
        .expect("System should run");

    let owner = app
        .world_mut()
        .spawn(GameplayTagCountContainer::default())
        .id();
    let spec_b = spawn_ability_spec(&mut app, owner, "test.block_ability");
    let spec_a = spawn_ability_spec(&mut app, owner, "test.attack_ability");

    // Activate B first (stays active)
    app.world_mut().trigger(TryActivateAbilityEvent {
        ability_spec: spec_b,
        owner,
    });
    app.update();

    let events = app.world().resource::<TestEvents>();
    assert_eq!(events.activated.lock().unwrap().len(), 1);
    assert!(events.ended.lock().unwrap().is_empty());

    // Activate A — should cancel B
    app.world_mut().trigger(TryActivateAbilityEvent {
        ability_spec: spec_a,
        owner,
    });
    app.update();

    let events = app.world().resource::<TestEvents>();
    assert_eq!(
        events.activated.lock().unwrap().len(),
        2,
        "Both abilities should have activated"
    );

    let ended = events.ended.lock().unwrap();
    assert_eq!(ended.len(), 1, "B should have been cancelled");
    assert_eq!(ended[0].ability_spec, spec_b);
    assert!(ended[0].was_cancelled, "B should be marked as cancelled");
}
