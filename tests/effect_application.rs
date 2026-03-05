//! Integration tests for gameplay effect application.
//!
//! Tests effect entity spawning, granted tags, expiration cleanup, and instant effects.

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use bevy_gameplay_ability_system::prelude::*;
use bevy_gameplay_tag::gameplay_tag::GameplayTag;
use bevy_gameplay_tag::gameplay_tag_count_container::GameplayTagCountContainer;
use bevy_gameplay_tag::{GameplayTagsManager, GameplayTagsPlugin};
use std::sync::{Arc, Mutex};
use string_cache::DefaultAtom as Atom;

#[derive(Resource, Clone, Default)]
struct EffectTestEvents {
    applied: Arc<Mutex<Vec<GameplayEffectAppliedEvent>>>,
    removed: Arc<Mutex<Vec<GameplayEffectRemovedEvent>>>,
}

fn setup_effect_app() -> App {
    let mut app = App::new();
    app.add_plugins(bevy::time::TimePlugin);
    app.add_plugins(GameplayTagsPlugin::with_data_path(
        "assets/gameplay_tags.json".to_string(),
    ));
    app.add_plugins(AttributePlugin);
    app.add_plugins(EffectPlugin);
    app.init_resource::<EffectTestEvents>();

    app.add_observer(
        |ev: On<GameplayEffectAppliedEvent>, events: Res<EffectTestEvents>| {
            events.applied.lock().unwrap().push(ev.event().clone());
        },
    );
    app.add_observer(
        |ev: On<GameplayEffectRemovedEvent>, events: Res<EffectTestEvents>| {
            events.removed.lock().unwrap().push(ev.event().clone());
        },
    );

    app.update();
    app
}

#[test]
fn test_duration_effect_spawns_entity() {
    let mut app = setup_effect_app();

    let effect = GameplayEffectDefinition::new("effect.test.duration")
        .with_duration(5.0);
    app.world_mut()
        .resource_mut::<GameplayEffectRegistry>()
        .register(effect);

    let target = app
        .world_mut()
        .spawn(GameplayTagCountContainer::default())
        .id();

    app.world_mut().trigger(ApplyGameplayEffectEvent {
        effect_id: Atom::from("effect.test.duration"),
        target,
        instigator: None,
        level: 1,
    });
    app.update();

    // Verify ActiveGameplayEffect entity was spawned
    let mut found = false;
    for (active_effect, effect_target) in app
        .world_mut()
        .query::<(&ActiveGameplayEffect, &EffectTarget)>()
        .iter(app.world())
    {
        if effect_target.0 == target && active_effect.definition_id.as_ref() == "effect.test.duration" {
            found = true;
        }
    }
    assert!(found, "ActiveGameplayEffect entity should exist");

    let events = app.world().resource::<EffectTestEvents>();
    assert_eq!(events.applied.lock().unwrap().len(), 1);
}

#[test]
fn test_effect_adds_granted_tags() {
    let mut app = setup_effect_app();

    app.world_mut()
        .run_system_once(
            |tags_manager: Res<GameplayTagsManager>,
             mut registry: ResMut<GameplayEffectRegistry>| {
                let effect = GameplayEffectDefinition::new("effect.test.buff")
                    .with_duration(10.0)
                    .grant_tag(GameplayTag::new("Effect.Buff.Attack"), &tags_manager);
                registry.register(effect);
            },
        )
        .expect("System should run");

    let target = app
        .world_mut()
        .spawn(GameplayTagCountContainer::default())
        .id();

    app.world_mut().trigger(ApplyGameplayEffectEvent {
        effect_id: Atom::from("effect.test.buff"),
        target,
        instigator: None,
        level: 1,
    });
    app.update();

    // Target should have the granted tag
    let target_tags = app.world().get::<GameplayTagCountContainer>(target).unwrap();
    let registry = app.world().resource::<GameplayEffectRegistry>();
    let def = registry.get("effect.test.buff").unwrap();
    assert!(
        target_tags.has_any_matching_gameplay_tags(&def.granted_tags),
        "Target should have Effect.Buff.Attack tag"
    );
}

#[test]
fn test_expired_effect_removes_tags() {
    let mut app = setup_effect_app();

    app.world_mut()
        .run_system_once(
            |tags_manager: Res<GameplayTagsManager>,
             mut registry: ResMut<GameplayEffectRegistry>| {
                let effect = GameplayEffectDefinition::new("effect.test.short")
                    .with_duration(0.1)
                    .grant_tag(GameplayTag::new("Effect.Buff.Defense"), &tags_manager);
                registry.register(effect);
            },
        )
        .expect("System should run");

    let target = app
        .world_mut()
        .spawn(GameplayTagCountContainer::default())
        .id();

    app.world_mut().trigger(ApplyGameplayEffectEvent {
        effect_id: Atom::from("effect.test.short"),
        target,
        instigator: None,
        level: 1,
    });
    app.update();

    // Verify tag is present
    let target_tags = app.world().get::<GameplayTagCountContainer>(target).unwrap();
    let registry = app.world().resource::<GameplayEffectRegistry>();
    let def = registry.get("effect.test.short").unwrap();
    assert!(
        target_tags.has_any_matching_gameplay_tags(&def.granted_tags),
        "Tag should be present before expiration"
    );

    // Manually expire the effect by setting remaining to 0
    for mut duration in app
        .world_mut()
        .query::<&mut EffectDuration>()
        .iter_mut(app.world_mut())
    {
        duration.remaining = 0.0;
    }

    // Run update to trigger removal
    app.update();

    // Tag should be removed
    let target_tags = app.world().get::<GameplayTagCountContainer>(target).unwrap();
    let registry = app.world().resource::<GameplayEffectRegistry>();
    let def = registry.get("effect.test.short").unwrap();
    assert!(
        !target_tags.has_any_matching_gameplay_tags(&def.granted_tags),
        "Tag should be removed after effect expires"
    );

    // Should have received removal event
    let events = app.world().resource::<EffectTestEvents>();
    assert_eq!(events.removed.lock().unwrap().len(), 1);
}

#[test]
fn test_instant_effect_modifies_base_value() {
    let mut app = setup_effect_app();

    let effect = GameplayEffectDefinition::new("effect.test.instant_heal")
        .with_duration_policy(DurationPolicy::Instant)
        .add_modifier(ModifierInfo {
            attribute_name: Atom::from("Health"),
            operation: ModifierOperation::AddBase,
            magnitude: MagnitudeCalculation::ScalableFloat { base_value: 25.0 },
        });
    app.world_mut()
        .resource_mut::<GameplayEffectRegistry>()
        .register(effect);

    let target = app
        .world_mut()
        .spawn(GameplayTagCountContainer::default())
        .id();

    app.world_mut().spawn((
        AttributeData {
            base_value: 75.0,
            current_value: 75.0,
        },
        AttributeName::new("Health"),
        AttributeOwner(target),
    ));

    app.world_mut().trigger(ApplyGameplayEffectEvent {
        effect_id: Atom::from("effect.test.instant_heal"),
        target,
        instigator: None,
        level: 1,
    });
    app.update();

    // Health base_value should be 100.0
    let mut found = false;
    for (attr, name, owner) in app
        .world_mut()
        .query::<(&AttributeData, &AttributeName, &AttributeOwner)>()
        .iter(app.world())
    {
        if owner.0 == target && name.as_str() == "Health" {
            assert!(
                (attr.base_value - 100.0).abs() < f32::EPSILON,
                "Health should be 100.0, got {}",
                attr.base_value
            );
            found = true;
        }
    }
    assert!(found, "Should have found Health attribute");

    let events = app.world().resource::<EffectTestEvents>();
    assert_eq!(events.applied.lock().unwrap().len(), 1);
}
