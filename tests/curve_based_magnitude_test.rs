//! Tests for CurveBased magnitude calculations using Bevy's Curve system.
//!
//! Verifies that effects can scale their magnitude based on level using curves.

use bevy::ecs::relationship::Relationship;
use bevy::math::curve::{Curve, SampleCurve, interval};
use bevy::prelude::*;
use bevy_gameplay_ability_system::{GasPlugin, attributes::*, core::*, effects::*};
use bevy_gameplay_tag::GameplayTagsPlugin;
use std::sync::Arc;

struct TestAttributeSet;

impl AttributeSetDefinition for TestAttributeSet {
    fn attribute_names() -> &'static [&'static str] {
        &["Health"]
    }

    fn attribute_metadata(name: &str) -> Option<AttributeMetadata> {
        match name {
            "Health" => Some(
                AttributeMetadata::new("Health")
                    .with_min(0.0)
                    .with_max(1000.0),
            ),
            _ => None,
        }
    }

    fn default_value(name: &str) -> f32 {
        match name {
            "Health" => 100.0,
            _ => 0.0,
        }
    }
}

fn setup_test_app() -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        GameplayTagsPlugin::with_data_path("assets/gameplay_tags.json".to_string()),
        GasPlugin,
    ));

    // Wait for tags to load
    app.update();

    app
}

fn find_attribute_entity(world: &mut World, owner: Entity, name: &str) -> Entity {
    let mut query = world.query::<(Entity, &AttributeName, &ChildOf)>();
    query
        .iter(world)
        .find(|(_, attr_name, child_of)| child_of.get() == owner && attr_name.as_str() == name)
        .map(|(entity, _, _)| entity)
        .expect("attribute should exist")
}

fn set_attribute_base_value(world: &mut World, owner: Entity, name: &str, value: f32) {
    let attr_entity = find_attribute_entity(world, owner, name);
    let mut attr_data = world.get_mut::<AttributeData>(attr_entity).unwrap();
    attr_data.base_value = value;
    attr_data.current_value = value;
}

fn get_attribute_current_value(world: &mut World, owner: Entity, name: &str) -> Option<f32> {
    let mut query = world.query::<(&AttributeData, &AttributeName, &ChildOf)>();
    query
        .iter(world)
        .find(|(_, attr_name, child_of)| child_of.get() == owner && attr_name.as_str() == name)
        .map(|(data, _, _)| data.current_value)
}

fn spawn_attribute_set<T: AttributeSetDefinition>(world: &mut World, owner: Entity) {
    for &name in T::attribute_names() {
        let metadata = T::attribute_metadata(name).unwrap_or_else(|| AttributeMetadata::new(name));
        let default_value = T::default_value(name);

        let attr_entity = world
            .spawn((
                AttributeData::new(default_value),
                AttributeName::new(name),
                AttributeSetId(std::any::TypeId::of::<T>()),
                AttributeMetadataComponent(metadata),
            ))
            .id();

        world.entity_mut(attr_entity).set_parent_in_place(owner);
    }
}

#[test]
fn test_curve_based_linear_damage_scaling() {
    let mut app = setup_test_app();

    // Create a linear damage curve: level 1 = 10, level 5 = 50
    let samples = vec![10.0, 20.0, 30.0, 40.0, 50.0];
    // Use linear interpolation
    let curve = SampleCurve::new(
        interval(1.0, 5.0).unwrap(),
        samples,
        |a: &f32, b: &f32, t: f32| a * (1.0 - t) + b * t,
    )
    .unwrap();

    // Register effect with curve-based magnitude
    app.world_mut()
        .resource_mut::<GameplayEffectRegistry>()
        .register(
            GameplayEffectDefinition::new("CurveDamage")
                .with_duration_policy(DurationPolicy::Instant)
                .add_modifier(ModifierInfo::new(
                    "Health",
                    ModifierOperation::AddCurrent,
                    MagnitudeCalculation::curve(Arc::new(curve)),
                )),
        );

    // Spawn target with 100 Health
    let target = app
        .world_mut()
        .spawn((
            Name::new("Target"),
            OwnedTags::default(),
            BlockedAbilityTags::default(),
        ))
        .id();
    spawn_attribute_set::<TestAttributeSet>(&mut app.world_mut(), target);
    set_attribute_base_value(&mut app.world_mut(), target, "Health", 100.0);

    app.update();

    // Test level 1: 100 + 10 = 110
    app.world_mut().trigger(
        ApplyGameplayEffectEvent::new("CurveDamage", target).with_level(1),
    );
    app.update();
    let health = get_attribute_current_value(&mut app.world_mut(), target, "Health").unwrap();
    assert_eq!(health, 110.0, "Level 1 should add 10 damage");

    // Reset health
    set_attribute_base_value(&mut app.world_mut(), target, "Health", 100.0);
    app.update();

    // Test level 3: 100 + 30 = 130
    app.world_mut().trigger(
        ApplyGameplayEffectEvent::new("CurveDamage", target).with_level(3),
    );
    app.update();
    let health = get_attribute_current_value(&mut app.world_mut(), target, "Health").unwrap();
    assert_eq!(health, 130.0, "Level 3 should add 30 damage");

    // Reset health
    set_attribute_base_value(&mut app.world_mut(), target, "Health", 100.0);
    app.update();

    // Test level 5: 100 + 50 = 150
    app.world_mut().trigger(
        ApplyGameplayEffectEvent::new("CurveDamage", target).with_level(5),
    );
    app.update();
    let health = get_attribute_current_value(&mut app.world_mut(), target, "Health").unwrap();
    assert_eq!(health, 150.0, "Level 5 should add 50 damage");
}

#[test]
fn test_curve_based_nonlinear_scaling() {
    let mut app = setup_test_app();

    // Create a nonlinear curve: exponential growth
    // Level 1 = 10, Level 5 = 100, Level 10 = 500
    let samples = vec![10.0, 20.0, 40.0, 70.0, 100.0, 150.0, 220.0, 310.0, 400.0, 500.0];
    let curve = SampleCurve::new(
        interval(1.0, 10.0).unwrap(),
        samples,
        |a: &f32, b: &f32, t: f32| a * (1.0 - t) + b * t,
    )
    .unwrap();

    // Register effect
    app.world_mut()
        .resource_mut::<GameplayEffectRegistry>()
        .register(
            GameplayEffectDefinition::new("NonlinearDamage")
                .with_duration_policy(DurationPolicy::Instant)
                .add_modifier(ModifierInfo::new(
                    "Health",
                    ModifierOperation::AddCurrent,
                    MagnitudeCalculation::curve(Arc::new(curve)),
                )),
        );

    // Spawn target
    let target = app
        .world_mut()
        .spawn((
            Name::new("Target"),
            OwnedTags::default(),
            BlockedAbilityTags::default(),
        ))
        .id();
    spawn_attribute_set::<TestAttributeSet>(&mut app.world_mut(), target);
    set_attribute_base_value(&mut app.world_mut(), target, "Health", 1000.0);

    app.update();

    // Test level 1: 1000 + 10 = 1010
    app.world_mut().trigger(
        ApplyGameplayEffectEvent::new("NonlinearDamage", target).with_level(1),
    );
    app.update();
    let health = get_attribute_current_value(&mut app.world_mut(), target, "Health").unwrap();
    assert_eq!(health, 1010.0, "Level 1 should add 10");

    // Reset
    set_attribute_base_value(&mut app.world_mut(), target, "Health", 1000.0);
    app.update();

    // Test level 5: 1000 + 100 = 1100
    app.world_mut().trigger(
        ApplyGameplayEffectEvent::new("NonlinearDamage", target).with_level(5),
    );
    app.update();
    let health = get_attribute_current_value(&mut app.world_mut(), target, "Health").unwrap();
    assert_eq!(health, 1100.0, "Level 5 should add 100");

    // Reset
    set_attribute_base_value(&mut app.world_mut(), target, "Health", 1000.0);
    app.update();

    // Test level 10: 1000 + 500 = 1500
    app.world_mut().trigger(
        ApplyGameplayEffectEvent::new("NonlinearDamage", target).with_level(10),
    );
    app.update();
    let health = get_attribute_current_value(&mut app.world_mut(), target, "Health").unwrap();
    assert_eq!(health, 1500.0, "Level 10 should add 500");
}

#[test]
fn test_curve_based_clamping() {
    let mut app = setup_test_app();

    // Create a curve with domain [1, 5]
    let samples = vec![10.0, 20.0, 30.0, 40.0, 50.0];
    let curve = SampleCurve::new(
        interval(1.0, 5.0).unwrap(),
        samples,
        |a: &f32, b: &f32, t: f32| a * (1.0 - t) + b * t,
    )
    .unwrap();

    // Register effect
    app.world_mut()
        .resource_mut::<GameplayEffectRegistry>()
        .register(
            GameplayEffectDefinition::new("ClampedDamage")
                .with_duration_policy(DurationPolicy::Instant)
                .add_modifier(ModifierInfo::new(
                    "Health",
                    ModifierOperation::AddCurrent,
                    MagnitudeCalculation::curve(Arc::new(curve)),
                )),
        );

    // Spawn target
    let target = app
        .world_mut()
        .spawn((
            Name::new("Target"),
            OwnedTags::default(),
            BlockedAbilityTags::default(),
        ))
        .id();
    spawn_attribute_set::<TestAttributeSet>(&mut app.world_mut(), target);
    set_attribute_base_value(&mut app.world_mut(), target, "Health", 100.0);

    app.update();

    // Test level 0 (below domain): should clamp to level 1 = 10
    app.world_mut().trigger(
        ApplyGameplayEffectEvent::new("ClampedDamage", target).with_level(0),
    );
    app.update();
    let health = get_attribute_current_value(&mut app.world_mut(), target, "Health").unwrap();
    assert_eq!(health, 110.0, "Level 0 should clamp to level 1 (10 damage)");

    // Reset
    set_attribute_base_value(&mut app.world_mut(), target, "Health", 100.0);
    app.update();

    // Test level 10 (above domain): should clamp to level 5 = 50
    app.world_mut().trigger(
        ApplyGameplayEffectEvent::new("ClampedDamage", target).with_level(10),
    );
    app.update();
    let health = get_attribute_current_value(&mut app.world_mut(), target, "Health").unwrap();
    assert_eq!(health, 150.0, "Level 10 should clamp to level 5 (50 damage)");
}
