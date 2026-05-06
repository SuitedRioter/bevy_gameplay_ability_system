//! Tests for AttributeBased magnitude calculations.
//!
//! Verifies that effects can scale their magnitude based on source or target attributes.

use bevy::ecs::relationship::Relationship;
use bevy::prelude::*;
use bevy_gameplay_ability_system::{GasPlugin, attributes::*, core::*, effects::*};
use bevy_gameplay_tag::GameplayTagsPlugin;

struct TestAttributeSet;

impl AttributeSetDefinition for TestAttributeSet {
    fn attribute_names() -> &'static [&'static str] {
        &["Attack", "Defense", "Health", "MaxHealth"]
    }

    fn attribute_metadata(name: &str) -> Option<AttributeMetadata> {
        match name {
            "Health" => Some(
                AttributeMetadata::new("Health")
                    .with_min(0.0)
                    .with_max(100.0),
            ),
            "MaxHealth" => Some(AttributeMetadata::new("MaxHealth").with_min(1.0)),
            _ => None,
        }
    }

    fn default_value(name: &str) -> f32 {
        match name {
            "Attack" => 0.0,
            "Defense" => 0.0,
            "Health" => 100.0,
            "MaxHealth" => 100.0,
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
fn test_attribute_based_damage_from_source_attack() {
    let mut app = setup_test_app();

    // Register effect: damage = source.Attack * -1.5 (negative for damage)
    app.world_mut()
        .resource_mut::<GameplayEffectRegistry>()
        .register(
            GameplayEffectDefinition::new("AttackDamage")
                .with_duration_policy(DurationPolicy::Instant)
                .add_modifier(ModifierInfo::new(
                    "Health",
                    ModifierOperation::AddCurrent,
                    MagnitudeCalculation::from_source_attribute("Attack", -1.5),
                )),
        );

    // Spawn attacker with 50 Attack
    let attacker = app
        .world_mut()
        .spawn((
            Name::new("Attacker"),
            OwnedTags::default(),
            BlockedAbilityTags::default(),
        ))
        .id();
    spawn_attribute_set::<TestAttributeSet>(&mut app.world_mut(), attacker);
    set_attribute_base_value(&mut app.world_mut(), attacker, "Attack", 50.0);

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

    // Apply damage effect from attacker to target
    app.world_mut().trigger(
        ApplyGameplayEffectEvent::new("AttackDamage", target)
            .with_source(attacker)
            .with_level(1),
    );

    app.update();

    // Verify damage: 100 + (50 * -1.5) = 25
    let health = get_attribute_current_value(&mut app.world_mut(), target, "Health").unwrap();
    assert_eq!(health, 25.0, "Health should be reduced by Attack * 1.5");
}

#[test]
fn test_attribute_based_healing_from_target_max_health() {
    let mut app = setup_test_app();

    // Register effect: heal = target.MaxHealth * 0.3
    app.world_mut()
        .resource_mut::<GameplayEffectRegistry>()
        .register(
            GameplayEffectDefinition::new("PercentHeal")
                .with_duration_policy(DurationPolicy::Instant)
                .add_modifier(ModifierInfo::new(
                    "Health",
                    ModifierOperation::AddCurrent,
                    MagnitudeCalculation::from_target_attribute("MaxHealth", 0.3),
                )),
        );

    // Spawn target with 200 MaxHealth, 50 Health
    let target = app
        .world_mut()
        .spawn((
            Name::new("Target"),
            OwnedTags::default(),
            BlockedAbilityTags::default(),
        ))
        .id();
    spawn_attribute_set::<TestAttributeSet>(&mut app.world_mut(), target);
    set_attribute_base_value(&mut app.world_mut(), target, "MaxHealth", 100.0); // Keep MaxHealth at 100
    set_attribute_base_value(&mut app.world_mut(), target, "Health", 50.0);

    app.update();

    // Apply heal effect
    app.world_mut()
        .trigger(ApplyGameplayEffectEvent::new("PercentHeal", target));

    app.update();

    // Verify heal: 50 + (100 * 0.3) = 80
    let health = get_attribute_current_value(&mut app.world_mut(), target, "Health").unwrap();
    assert_eq!(health, 80.0, "Health should be healed by MaxHealth * 0.3");
}

#[test]
fn test_attribute_based_with_base_value_calculation() {
    let mut app = setup_test_app();

    // Register effect: damage = source.Attack (base value only) * -2.0
    app.world_mut()
        .resource_mut::<GameplayEffectRegistry>()
        .register(
            GameplayEffectDefinition::new("BaseAttackDamage")
                .with_duration_policy(DurationPolicy::Instant)
                .add_modifier(ModifierInfo::new(
                    "Health",
                    ModifierOperation::AddCurrent,
                    MagnitudeCalculation::AttributeBased {
                        attribute_name: "Attack".into(),
                        capture_source: AttributeCaptureSource::Source,
                        calculation_type: AttributeCalculationType::AttributeBaseValue,
                        capture_mode: AttributeCaptureMode::Snapshot,
                        coefficient: -2.0,
                        pre_multiply_additive: 0.0,
                        post_multiply_additive: 0.0,
                    },
                )),
        );

    // Spawn attacker with 30 base Attack
    let attacker = app
        .world_mut()
        .spawn((
            Name::new("Attacker"),
            OwnedTags::default(),
            BlockedAbilityTags::default(),
        ))
        .id();
    spawn_attribute_set::<TestAttributeSet>(&mut app.world_mut(), attacker);
    set_attribute_base_value(&mut app.world_mut(), attacker, "Attack", 30.0);

    // Apply a buff to increase current Attack
    app.world_mut()
        .resource_mut::<GameplayEffectRegistry>()
        .register(
            GameplayEffectDefinition::new("AttackBuff")
                .with_duration_policy(DurationPolicy::Infinite)
                .add_modifier(ModifierInfo::new(
                    "Attack",
                    ModifierOperation::AddCurrent,
                    MagnitudeCalculation::scalar(20.0),
                )),
        );
    app.world_mut()
        .trigger(ApplyGameplayEffectEvent::new("AttackBuff", attacker));

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

    // Apply damage effect (should use base Attack = 30, not current = 50)
    app.world_mut().trigger(
        ApplyGameplayEffectEvent::new("BaseAttackDamage", target)
            .with_source(attacker)
            .with_level(1),
    );

    app.update();

    // Verify damage: 100 + (30 * -2.0) = 40
    let health = get_attribute_current_value(&mut app.world_mut(), target, "Health").unwrap();
    assert_eq!(health, 40.0, "Damage should use base Attack, not current");
}

#[test]
fn test_attribute_based_with_bonus_magnitude() {
    let mut app = setup_test_app();

    // Register effect: damage = source.Attack (bonus only) * -3.0
    app.world_mut()
        .resource_mut::<GameplayEffectRegistry>()
        .register(
            GameplayEffectDefinition::new("BonusAttackDamage")
                .with_duration_policy(DurationPolicy::Instant)
                .add_modifier(ModifierInfo::new(
                    "Health",
                    ModifierOperation::AddCurrent,
                    MagnitudeCalculation::AttributeBased {
                        attribute_name: "Attack".into(),
                        capture_source: AttributeCaptureSource::Source,
                        calculation_type: AttributeCalculationType::AttributeBonusMagnitude,
                        capture_mode: AttributeCaptureMode::Snapshot,
                        coefficient: -3.0,
                        pre_multiply_additive: 0.0,
                        post_multiply_additive: 0.0,
                    },
                )),
        );

    // Spawn attacker with 30 base Attack
    let attacker = app
        .world_mut()
        .spawn((
            Name::new("Attacker"),
            OwnedTags::default(),
            BlockedAbilityTags::default(),
        ))
        .id();
    spawn_attribute_set::<TestAttributeSet>(&mut app.world_mut(), attacker);
    set_attribute_base_value(&mut app.world_mut(), attacker, "Attack", 30.0);

    // Apply a buff (bonus = 20)
    app.world_mut()
        .resource_mut::<GameplayEffectRegistry>()
        .register(
            GameplayEffectDefinition::new("AttackBuff")
                .with_duration_policy(DurationPolicy::Infinite)
                .add_modifier(ModifierInfo::new(
                    "Attack",
                    ModifierOperation::AddCurrent,
                    MagnitudeCalculation::scalar(20.0),
                )),
        );
    app.world_mut()
        .trigger(ApplyGameplayEffectEvent::new("AttackBuff", attacker));

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

    // Apply damage effect (should use bonus = 20)
    app.world_mut().trigger(
        ApplyGameplayEffectEvent::new("BonusAttackDamage", target)
            .with_source(attacker)
            .with_level(1),
    );

    app.update();

    // Verify damage: 100 + (20 * -3.0) = 40
    let health = get_attribute_current_value(&mut app.world_mut(), target, "Health").unwrap();
    assert_eq!(health, 40.0, "Damage should use bonus Attack only");
}

#[test]
fn test_attribute_based_with_pre_and_post_multiply() {
    let mut app = setup_test_app();

    // Register effect: damage = (source.Attack + 10) * -2.0 + 5
    app.world_mut()
        .resource_mut::<GameplayEffectRegistry>()
        .register(
            GameplayEffectDefinition::new("ComplexDamage")
                .with_duration_policy(DurationPolicy::Instant)
                .add_modifier(ModifierInfo::new(
                    "Health",
                    ModifierOperation::AddCurrent,
                    MagnitudeCalculation::from_source_attribute("Attack", -2.0)
                        .with_pre_multiply_add(10.0)
                        .with_post_multiply_add(5.0),
                )),
        );

    // Spawn attacker with 20 Attack
    let attacker = app
        .world_mut()
        .spawn((
            Name::new("Attacker"),
            OwnedTags::default(),
            BlockedAbilityTags::default(),
        ))
        .id();
    spawn_attribute_set::<TestAttributeSet>(&mut app.world_mut(), attacker);
    set_attribute_base_value(&mut app.world_mut(), attacker, "Attack", 20.0);

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

    // Apply damage effect
    app.world_mut().trigger(
        ApplyGameplayEffectEvent::new("ComplexDamage", target)
            .with_source(attacker)
            .with_level(1),
    );

    app.update();

    // Verify damage: 100 + ((20 + 10) * -2.0 + 5) = 100 + (-60 + 5) = 45
    let health = get_attribute_current_value(&mut app.world_mut(), target, "Health").unwrap();
    assert_eq!(
        health, 45.0,
        "Complex formula should be evaluated correctly"
    );
}
