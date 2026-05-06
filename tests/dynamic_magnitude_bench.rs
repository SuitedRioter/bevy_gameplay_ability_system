//! Performance benchmark for Dynamic magnitude calculations.
//!
//! Compares the performance of Snapshot vs Dynamic mode under different scenarios.

use bevy::ecs::relationship::Relationship;
use bevy::prelude::*;
use bevy_gameplay_ability_system::{GasPlugin, attributes::*, core::*, effects::*};
use bevy_gameplay_tag::GameplayTagsPlugin;

struct TestAttributeSet;

impl AttributeSetDefinition for TestAttributeSet {
    fn attribute_names() -> &'static [&'static str] {
        &["Attack", "Health"]
    }

    fn attribute_metadata(_name: &str) -> Option<AttributeMetadata> {
        None
    }

    fn default_value(name: &str) -> f32 {
        match name {
            "Attack" => 100.0,
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
    app.update();
    app
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

/// Benchmark: Static scenario (no attribute changes)
/// Expected: Dynamic mode should have ~0 overhead (no Changed<AttributeData>)
#[test]
fn bench_dynamic_static_scenario() {
    let mut app = setup_test_app();

    // Register Dynamic aura
    app.world_mut()
        .resource_mut::<GameplayEffectRegistry>()
        .register(
            GameplayEffectDefinition::new("DynamicAura")
                .with_duration_policy(DurationPolicy::Infinite)
                .add_modifier(ModifierInfo::new(
                    "Attack",
                    ModifierOperation::AddCurrent,
                    MagnitudeCalculation::AttributeBased {
                        attribute_name: "Attack".into(),
                        capture_source: AttributeCaptureSource::Source,
                        calculation_type: AttributeCalculationType::AttributeMagnitude,
                        capture_mode: AttributeCaptureMode::Dynamic,
                        coefficient: 0.1,
                        pre_multiply_additive: 0.0,
                        post_multiply_additive: 0.0,
                    },
                )),
        );

    // Create 10 sources and 10 targets (100 auras total)
    let mut sources = Vec::new();
    let mut targets = Vec::new();

    for i in 0..10 {
        let source = app
            .world_mut()
            .spawn((
                Name::new(format!("Source{}", i)),
                OwnedTags::default(),
                BlockedAbilityTags::default(),
            ))
            .id();
        spawn_attribute_set::<TestAttributeSet>(&mut app.world_mut(), source);
        set_attribute_base_value(&mut app.world_mut(), source, "Attack", 100.0);
        sources.push(source);

        let target = app
            .world_mut()
            .spawn((
                Name::new(format!("Target{}", i)),
                OwnedTags::default(),
                BlockedAbilityTags::default(),
            ))
            .id();
        spawn_attribute_set::<TestAttributeSet>(&mut app.world_mut(), target);
        targets.push(target);
    }

    // Apply auras
    for source in &sources {
        for target in &targets {
            app.world_mut().trigger(
                ApplyGameplayEffectEvent::new("DynamicAura", *target)
                    .with_source(*source)
                    .with_level(1),
            );
        }
    }

    app.update();

    // Benchmark: Run 100 frames with no attribute changes
    let start = std::time::Instant::now();
    for _ in 0..100 {
        app.update();
    }
    let elapsed = start.elapsed();

    println!(
        "Static scenario (100 auras, 100 frames, no changes): {:?}",
        elapsed
    );
    println!("Average per frame: {:?}", elapsed / 100);

    // Expected: <1ms per frame (no Changed<AttributeData> triggers)
    assert!(
        elapsed.as_millis() < 100,
        "Static scenario should be fast (no attribute changes)"
    );
}

/// Benchmark: Active scenario (1 attribute changes per frame)
/// Expected: Dynamic mode only recalculates affected modifiers
#[test]
fn bench_dynamic_active_scenario() {
    let mut app = setup_test_app();

    // Register Dynamic aura
    app.world_mut()
        .resource_mut::<GameplayEffectRegistry>()
        .register(
            GameplayEffectDefinition::new("DynamicAura")
                .with_duration_policy(DurationPolicy::Infinite)
                .add_modifier(ModifierInfo::new(
                    "Attack",
                    ModifierOperation::AddCurrent,
                    MagnitudeCalculation::AttributeBased {
                        attribute_name: "Attack".into(),
                        capture_source: AttributeCaptureSource::Source,
                        calculation_type: AttributeCalculationType::AttributeMagnitude,
                        capture_mode: AttributeCaptureMode::Dynamic,
                        coefficient: 0.1,
                        pre_multiply_additive: 0.0,
                        post_multiply_additive: 0.0,
                    },
                )),
        );

    // Create 10 sources and 10 targets (100 auras total)
    let mut sources = Vec::new();
    let mut targets = Vec::new();

    for i in 0..10 {
        let source = app
            .world_mut()
            .spawn((
                Name::new(format!("Source{}", i)),
                OwnedTags::default(),
                BlockedAbilityTags::default(),
            ))
            .id();
        spawn_attribute_set::<TestAttributeSet>(&mut app.world_mut(), source);
        set_attribute_base_value(&mut app.world_mut(), source, "Attack", 100.0);
        sources.push(source);

        let target = app
            .world_mut()
            .spawn((
                Name::new(format!("Target{}", i)),
                OwnedTags::default(),
                BlockedAbilityTags::default(),
            ))
            .id();
        spawn_attribute_set::<TestAttributeSet>(&mut app.world_mut(), target);
        targets.push(target);
    }

    // Apply auras
    for source in &sources {
        for target in &targets {
            app.world_mut().trigger(
                ApplyGameplayEffectEvent::new("DynamicAura", *target)
                    .with_source(*source)
                    .with_level(1),
            );
        }
    }

    app.update();

    // Benchmark: Run 100 frames, changing 1 source's Attack each frame
    let start = std::time::Instant::now();
    for i in 0..100 {
        let source = sources[i % sources.len()];
        set_attribute_base_value(&mut app.world_mut(), source, "Attack", 100.0 + i as f32);
        app.update();
    }
    let elapsed = start.elapsed();

    println!(
        "Active scenario (100 auras, 100 frames, 1 change/frame): {:?}",
        elapsed
    );
    println!("Average per frame: {:?}", elapsed / 100);

    // Expected: <5ms per frame (only 10 modifiers recalculated per frame)
    assert!(
        elapsed.as_millis() < 500,
        "Active scenario should be reasonably fast"
    );
}
