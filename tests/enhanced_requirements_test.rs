use bevy::ecs::relationship::Relationship;
use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use bevy_gameplay_ability_system::{GasPlugin, attributes::*, core::OwnedTags, effects::*};
use bevy_gameplay_tag::{GameplayTagsPlugin, gameplay_tag::GameplayTag};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct TestAttributeSet;

impl AttributeSetDefinition for TestAttributeSet {
    fn attribute_names() -> &'static [&'static str] {
        &["Health", "MaxHealth", "AttackPower", "Defense"]
    }

    fn attribute_metadata(name: &str) -> Option<AttributeMetadata> {
        match name {
            "Health" => Some(AttributeMetadata::new("Health").with_min(0.0)),
            "MaxHealth" => Some(AttributeMetadata::new("MaxHealth").with_min(1.0)),
            "AttackPower" => Some(AttributeMetadata::new("AttackPower").with_min(0.0)),
            "Defense" => Some(AttributeMetadata::new("Defense").with_min(0.0)),
            _ => None,
        }
    }

    fn default_value(name: &str) -> f32 {
        match name {
            "Health" => 100.0,
            "MaxHealth" => 100.0,
            "AttackPower" => 50.0,
            "Defense" => 20.0,
            _ => 0.0,
        }
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

/// Test AttributePercentBelow requirement.
#[test]
fn test_attribute_percent_below() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        GameplayTagsPlugin::with_data_path("assets/gameplay_tags.json".to_string()),
        GasPlugin,
    ));
    app.update();

    app.world_mut()
        .resource_mut::<ApplicationRequirementRegistry>()
        .register(
            "low_health_only",
            Box::new(AttributePercentBelow::new("Health", "MaxHealth", 0.5)),
        );

    app.world_mut()
        .resource_mut::<GameplayEffectRegistry>()
        .register(
            GameplayEffectDefinition::new("emergency_heal")
                .with_duration_policy(DurationPolicy::Instant)
                .add_modifier(ModifierInfo::new(
                    "Health",
                    ModifierOperation::AddBase,
                    MagnitudeCalculation::scalar(50.0),
                ))
                .add_application_requirement("low_health_only"),
        );

    let owner = {
        let mut commands = app.world_mut().commands();
        let owner = commands.spawn_empty().id();
        TestAttributeSet::create_attributes(&mut commands, owner);
        owner
    };

    app.update();

    // Health at 100/100 (100%) - should be blocked
    app.world_mut()
        .trigger(ApplyGameplayEffectEvent::new("emergency_heal", owner));
    app.update();

    let health_attr = find_attribute_entity(app.world_mut(), owner, "Health");
    let health_value = app
        .world()
        .entity(health_attr)
        .get::<AttributeData>()
        .unwrap()
        .base_value;

    assert!(
        (health_value - 100.0).abs() < 0.01,
        "Effect should be blocked at 100% health"
    );

    // Reduce to 30/100 (30%) - should apply
    app.world_mut()
        .entity_mut(health_attr)
        .get_mut::<AttributeData>()
        .unwrap()
        .base_value = 30.0;

    app.update();

    app.world_mut()
        .trigger(ApplyGameplayEffectEvent::new("emergency_heal", owner));
    app.update();

    let health_value = app
        .world()
        .entity(health_attr)
        .get::<AttributeData>()
        .unwrap()
        .base_value;

    assert!(
        (health_value - 80.0).abs() < 0.01,
        "Effect should apply at 30% health, got {}",
        health_value
    );
}

/// Test SourceAttributeGreaterThanTarget requirement.
#[test]
fn test_source_attribute_greater_than_target() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        GameplayTagsPlugin::with_data_path("assets/gameplay_tags.json".to_string()),
        GasPlugin,
    ));
    app.update();

    app.world_mut()
        .resource_mut::<ApplicationRequirementRegistry>()
        .register(
            "attacker_stronger",
            Box::new(SourceAttributeGreaterThanTarget::new(
                "AttackPower",
                "Defense",
            )),
        );

    app.world_mut()
        .resource_mut::<GameplayEffectRegistry>()
        .register(
            GameplayEffectDefinition::new("armor_break")
                .with_duration_policy(DurationPolicy::Instant)
                .add_modifier(ModifierInfo::new(
                    "Defense",
                    ModifierOperation::AddBase,
                    MagnitudeCalculation::scalar(-10.0),
                ))
                .add_application_requirement("attacker_stronger"),
        );

    let source = {
        let mut commands = app.world_mut().commands();
        let source = commands.spawn_empty().id();
        TestAttributeSet::create_attributes(&mut commands, source);
        source
    };

    let target = {
        let mut commands = app.world_mut().commands();
        let target = commands.spawn_empty().id();
        TestAttributeSet::create_attributes(&mut commands, target);
        target
    };

    app.update();

    // Source AttackPower: 50, Target Defense: 20 (50 > 20, should apply)
    app.world_mut()
        .trigger(ApplyGameplayEffectEvent::new("armor_break", target).with_source(source));
    app.update();

    let defense_attr = find_attribute_entity(app.world_mut(), target, "Defense");
    let defense_value = app
        .world()
        .entity(defense_attr)
        .get::<AttributeData>()
        .unwrap()
        .base_value;

    assert!(
        (defense_value - 10.0).abs() < 0.01,
        "Effect should apply when source AttackPower > target Defense"
    );
}

/// Test RequireAllTags requirement.
#[test]
fn test_require_all_tags() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        GameplayTagsPlugin::with_data_path("assets/gameplay_tags.json".to_string()),
        GasPlugin,
    ));
    app.update();

    app.world_mut()
        .resource_mut::<ApplicationRequirementRegistry>()
        .register(
            "alive_and_not_stunned",
            Box::new(RequireAllTags::new(vec![GameplayTag::new("State.Alive")])),
        );

    app.world_mut()
        .resource_mut::<GameplayEffectRegistry>()
        .register(
            GameplayEffectDefinition::new("heal")
                .with_duration_policy(DurationPolicy::Instant)
                .add_modifier(ModifierInfo::new(
                    "Health",
                    ModifierOperation::AddBase,
                    MagnitudeCalculation::scalar(25.0),
                ))
                .add_application_requirement("alive_and_not_stunned"),
        );

    let owner = {
        let mut commands = app.world_mut().commands();
        let owner = commands.spawn(OwnedTags::default()).id();
        TestAttributeSet::create_attributes(&mut commands, owner);
        owner
    };

    app.update();

    // Without State.Alive tag - should be blocked
    app.world_mut()
        .trigger(ApplyGameplayEffectEvent::new("heal", owner));
    app.update();

    let health_attr = find_attribute_entity(app.world_mut(), owner, "Health");
    let health_value = app
        .world()
        .entity(health_attr)
        .get::<AttributeData>()
        .unwrap()
        .base_value;

    assert!(
        (health_value - 100.0).abs() < 0.01,
        "Effect should be blocked without required tags"
    );

    // Add State.Alive tag
    app.world_mut()
        .run_system_once(
            move |mut query: Query<&mut OwnedTags>,
                  tags_manager: Res<bevy_gameplay_tag::GameplayTagsManager>| {
                if let Ok(mut tags) = query.get_mut(owner) {
                    tags.0
                        .explicit_tags
                        .add_tag(GameplayTag::new("State.Alive"), &tags_manager);
                }
            },
        )
        .unwrap();

    app.update();

    // With State.Alive tag - should apply
    app.world_mut()
        .trigger(ApplyGameplayEffectEvent::new("heal", owner));
    app.update();

    let health_value = app
        .world()
        .entity(health_attr)
        .get::<AttributeData>()
        .unwrap()
        .base_value;

    assert!(
        (health_value - 125.0).abs() < 0.01,
        "Effect should apply with required tags"
    );
}

/// Test LevelRangeRequirement.
#[test]
fn test_level_range_requirement() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        GameplayTagsPlugin::with_data_path("assets/gameplay_tags.json".to_string()),
        GasPlugin,
    ));
    app.update();

    app.world_mut()
        .resource_mut::<ApplicationRequirementRegistry>()
        .register(
            "mid_level_only",
            Box::new(LevelRangeRequirement::new(5, 10)),
        );

    app.world_mut()
        .resource_mut::<GameplayEffectRegistry>()
        .register(
            GameplayEffectDefinition::new("level_scaled_buff")
                .with_duration_policy(DurationPolicy::Instant)
                .add_modifier(ModifierInfo::new(
                    "Health",
                    ModifierOperation::AddBase,
                    MagnitudeCalculation::scalar(25.0),
                ))
                .add_application_requirement("mid_level_only"),
        );

    let owner = {
        let mut commands = app.world_mut().commands();
        let owner = commands.spawn_empty().id();
        TestAttributeSet::create_attributes(&mut commands, owner);
        owner
    };

    app.update();

    let health_attr = find_attribute_entity(app.world_mut(), owner, "Health");

    // Level 3 - should be blocked
    app.world_mut()
        .trigger(ApplyGameplayEffectEvent::new("level_scaled_buff", owner).with_level(3));
    app.update();

    let health_value = app
        .world()
        .entity(health_attr)
        .get::<AttributeData>()
        .unwrap()
        .base_value;

    assert!(
        (health_value - 100.0).abs() < 0.01,
        "Effect should be blocked at level 3"
    );

    // Level 7 - should apply
    app.world_mut()
        .trigger(ApplyGameplayEffectEvent::new("level_scaled_buff", owner).with_level(7));
    app.update();

    let health_value = app
        .world()
        .entity(health_attr)
        .get::<AttributeData>()
        .unwrap()
        .base_value;

    assert!(
        (health_value - 125.0).abs() < 0.01,
        "Effect should apply at level 7"
    );
}
