//! Tests for Dynamic AttributeBased magnitude calculations.
//!
//! Verifies that Dynamic mode modifiers update their magnitude when source attributes change.

use bevy::ecs::relationship::Relationship;
use bevy::prelude::*;
use bevy_gameplay_ability_system::{GasPlugin, attributes::*, core::*, effects::*};
use bevy_gameplay_tag::GameplayTagsPlugin;

struct TestAttributeSet;

impl AttributeSetDefinition for TestAttributeSet {
    fn attribute_names() -> &'static [&'static str] {
        &["Attack", "Health"]
    }

    fn attribute_metadata(name: &str) -> Option<AttributeMetadata> {
        match name {
            "Health" => Some(
                AttributeMetadata::new("Health")
                    .with_min(0.0)
                    .with_max(200.0),
            ),
            _ => None,
        }
    }

    fn default_value(name: &str) -> f32 {
        match name {
            "Attack" => 0.0,
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
fn test_dynamic_aura_updates_when_source_changes() {
    let mut app = setup_test_app();

    // Register aura effect: target gets +10% of source's Attack (Dynamic mode)
    app.world_mut()
        .resource_mut::<GameplayEffectRegistry>()
        .register(
            GameplayEffectDefinition::new("AttackAura")
                .with_duration_policy(DurationPolicy::Infinite)
                .add_modifier(ModifierInfo::new(
                    "Attack",
                    ModifierOperation::AddCurrent,
                    MagnitudeCalculation::AttributeBased {
                        attribute_name: "Attack".into(),
                        capture_source: AttributeCaptureSource::Source,
                        calculation_type: AttributeCalculationType::AttributeMagnitude,
                        capture_mode: AttributeCaptureMode::Dynamic, // Dynamic mode
                        coefficient: 0.1,
                        pre_multiply_additive: 0.0,
                        post_multiply_additive: 0.0,
                    },
                )),
        );

    // Spawn source with 100 Attack
    let source = app
        .world_mut()
        .spawn((
            Name::new("Source"),
            OwnedTags::default(),
            BlockedAbilityTags::default(),
        ))
        .id();
    spawn_attribute_set::<TestAttributeSet>(app.world_mut(), source);
    set_attribute_base_value(app.world_mut(), source, "Attack", 100.0);

    // Spawn target with 50 Attack
    let target = app
        .world_mut()
        .spawn((
            Name::new("Target"),
            OwnedTags::default(),
            BlockedAbilityTags::default(),
        ))
        .id();
    spawn_attribute_set::<TestAttributeSet>(app.world_mut(), target);
    set_attribute_base_value(app.world_mut(), target, "Attack", 50.0);

    app.update();

    // Apply aura from source to target
    app.world_mut().trigger(
        ApplyGameplayEffectEvent::new("AttackAura", target)
            .with_source(source)
            .with_level(1),
    );

    app.update();

    // Verify initial bonus: 50 + (100 * 0.1) = 60
    let attack = get_attribute_current_value(app.world_mut(), target, "Attack").unwrap();
    assert_eq!(attack, 60.0, "Target should have +10 Attack from aura");

    // Source's Attack increases to 200
    set_attribute_base_value(app.world_mut(), source, "Attack", 200.0);

    app.update();

    // Verify dynamic update: 50 + (200 * 0.1) = 70
    let attack = get_attribute_current_value(app.world_mut(), target, "Attack").unwrap();
    assert_eq!(
        attack, 70.0,
        "Target's bonus should update when source's Attack changes"
    );

    // Source's Attack decreases to 50
    set_attribute_base_value(app.world_mut(), source, "Attack", 50.0);

    app.update();

    // Verify dynamic update: 50 + (50 * 0.1) = 55
    let attack = get_attribute_current_value(app.world_mut(), target, "Attack").unwrap();
    assert_eq!(
        attack, 55.0,
        "Target's bonus should decrease when source's Attack decreases"
    );
}

#[test]
fn test_snapshot_mode_does_not_update() {
    let mut app = setup_test_app();

    // Register effect with Snapshot mode (default)
    app.world_mut()
        .resource_mut::<GameplayEffectRegistry>()
        .register(
            GameplayEffectDefinition::new("SnapshotBuff")
                .with_duration_policy(DurationPolicy::Infinite)
                .add_modifier(ModifierInfo::new(
                    "Attack",
                    ModifierOperation::AddCurrent,
                    MagnitudeCalculation::from_source_attribute("Attack", 0.1), // Snapshot mode
                )),
        );

    // Spawn source with 100 Attack
    let source = app
        .world_mut()
        .spawn((
            Name::new("Source"),
            OwnedTags::default(),
            BlockedAbilityTags::default(),
        ))
        .id();
    spawn_attribute_set::<TestAttributeSet>(app.world_mut(), source);
    set_attribute_base_value(app.world_mut(), source, "Attack", 100.0);

    // Spawn target with 50 Attack
    let target = app
        .world_mut()
        .spawn((
            Name::new("Target"),
            OwnedTags::default(),
            BlockedAbilityTags::default(),
        ))
        .id();
    spawn_attribute_set::<TestAttributeSet>(app.world_mut(), target);
    set_attribute_base_value(app.world_mut(), target, "Attack", 50.0);

    app.update();

    // Apply buff
    app.world_mut().trigger(
        ApplyGameplayEffectEvent::new("SnapshotBuff", target)
            .with_source(source)
            .with_level(1),
    );

    app.update();

    // Verify initial bonus: 50 + (100 * 0.1) = 60
    let attack = get_attribute_current_value(app.world_mut(), target, "Attack").unwrap();
    assert_eq!(attack, 60.0, "Target should have +10 Attack from buff");

    // Source's Attack increases to 200
    set_attribute_base_value(app.world_mut(), source, "Attack", 200.0);

    app.update();

    // Verify Snapshot mode: bonus stays at 10 (does NOT update)
    let attack = get_attribute_current_value(app.world_mut(), target, "Attack").unwrap();
    assert_eq!(
        attack, 60.0,
        "Snapshot mode should NOT update when source changes"
    );
}

#[test]
fn test_dynamic_with_base_value_calculation() {
    let mut app = setup_test_app();

    // Register effect: target gets +20% of source's base Attack (Dynamic mode)
    app.world_mut()
        .resource_mut::<GameplayEffectRegistry>()
        .register(
            GameplayEffectDefinition::new("BaseAttackAura")
                .with_duration_policy(DurationPolicy::Infinite)
                .add_modifier(ModifierInfo::new(
                    "Attack",
                    ModifierOperation::AddCurrent,
                    MagnitudeCalculation::AttributeBased {
                        attribute_name: "Attack".into(),
                        capture_source: AttributeCaptureSource::Source,
                        calculation_type: AttributeCalculationType::AttributeBaseValue,
                        capture_mode: AttributeCaptureMode::Dynamic,
                        coefficient: 0.2,
                        pre_multiply_additive: 0.0,
                        post_multiply_additive: 0.0,
                    },
                )),
        );

    // Spawn source with 100 base Attack
    let source = app
        .world_mut()
        .spawn((
            Name::new("Source"),
            OwnedTags::default(),
            BlockedAbilityTags::default(),
        ))
        .id();
    spawn_attribute_set::<TestAttributeSet>(app.world_mut(), source);
    set_attribute_base_value(app.world_mut(), source, "Attack", 100.0);

    // Apply a temporary buff to source (increases current but not base)
    app.world_mut()
        .resource_mut::<GameplayEffectRegistry>()
        .register(
            GameplayEffectDefinition::new("TempBuff")
                .with_duration_policy(DurationPolicy::Infinite)
                .add_modifier(ModifierInfo::new(
                    "Attack",
                    ModifierOperation::AddCurrent,
                    MagnitudeCalculation::scalar(50.0),
                )),
        );
    app.world_mut()
        .trigger(ApplyGameplayEffectEvent::new("TempBuff", source));

    // Spawn target with 50 Attack
    let target = app
        .world_mut()
        .spawn((
            Name::new("Target"),
            OwnedTags::default(),
            BlockedAbilityTags::default(),
        ))
        .id();
    spawn_attribute_set::<TestAttributeSet>(app.world_mut(), target);
    set_attribute_base_value(app.world_mut(), target, "Attack", 50.0);

    app.update();

    // Apply aura
    app.world_mut().trigger(
        ApplyGameplayEffectEvent::new("BaseAttackAura", target)
            .with_source(source)
            .with_level(1),
    );

    app.update();

    // Verify: uses base Attack (100), not current (150)
    // 50 + (100 * 0.2) = 70
    let attack = get_attribute_current_value(app.world_mut(), target, "Attack").unwrap();
    assert_eq!(
        attack, 70.0,
        "Should use base Attack, not current (with temp buff)"
    );

    // Source's base Attack increases to 200
    set_attribute_base_value(app.world_mut(), source, "Attack", 200.0);

    app.update();

    // Verify dynamic update: 50 + (200 * 0.2) = 90
    let attack = get_attribute_current_value(app.world_mut(), target, "Attack").unwrap();
    assert_eq!(
        attack, 90.0,
        "Should update when source's base Attack changes"
    );
}
