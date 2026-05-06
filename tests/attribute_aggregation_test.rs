use bevy::ecs::relationship::Relationship;
use bevy::prelude::*;
use bevy_gameplay_ability_system::{GasPlugin, attributes::*, effects::*};
use bevy_gameplay_tag::GameplayTagsPlugin;

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

fn find_attribute_entity(world: &mut World, owner: Entity, name: &str) -> Entity {
    let mut query = world.query::<(Entity, &AttributeName, &ChildOf)>();
    query
        .iter(world)
        .find(|(_, attr_name, child_of)| child_of.get() == owner && attr_name.as_str() == name)
        .map(|(entity, _, _)| entity)
        .expect("attribute should exist")
}

#[test]
fn test_set_base_value_preserves_active_modifier_effect_until_reaggregation() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        GameplayTagsPlugin::with_data_path("assets/gameplay_tags.json".to_string()),
        GasPlugin,
    ));
    app.update();

    app.world_mut()
        .resource_mut::<GameplayEffectRegistry>()
        .register(
            GameplayEffectDefinition::new("base_buff")
                .with_duration(5.0)
                .add_modifier(ModifierInfo::new(
                    "Health",
                    ModifierOperation::AddCurrent,
                    MagnitudeCalculation::scalar(25.0),
                )),
        );

    let owner = {
        let mut commands = app.world_mut().commands();
        let owner = commands.spawn_empty().id();
        TestAttributeSet::create_attributes(&mut commands, owner);
        owner
    };

    app.world_mut()
        .trigger(ApplyGameplayEffectEvent::new("base_buff", owner).with_level(1));
    app.update();

    let health_attr = find_attribute_entity(app.world_mut(), owner, "Health");

    {
        let mut entity = app.world_mut().entity_mut(health_attr);
        let attr = entity.get_mut::<AttributeData>().unwrap();
        assert_eq!(attr.base_value, 100.0);
        assert_eq!(attr.current_value, 125.0);
    }

    {
        let mut entity = app.world_mut().entity_mut(health_attr);
        entity
            .get_mut::<AttributeData>()
            .unwrap()
            .set_base_value(150.0);
    }

    {
        let attr = app
            .world()
            .entity(health_attr)
            .get::<AttributeData>()
            .unwrap();
        assert_eq!(attr.base_value, 150.0);
        assert_eq!(
            attr.current_value, 125.0,
            "set_base_value should not directly clobber current_value"
        );
    }

    app.update();

    let attr = app
        .world()
        .entity(health_attr)
        .get::<AttributeData>()
        .unwrap();
    assert_eq!(attr.base_value, 150.0);
    assert_eq!(
        attr.current_value, 175.0,
        "aggregation should recompute current_value from the new base plus active modifiers"
    );
}

#[test]
fn test_add_base_modifier_recomputes_from_base_value() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        GameplayTagsPlugin::with_data_path("assets/gameplay_tags.json".to_string()),
        GasPlugin,
    ));
    app.update();

    app.world_mut()
        .resource_mut::<GameplayEffectRegistry>()
        .register(
            GameplayEffectDefinition::new("permanent_style_buff")
                .with_duration(5.0)
                .add_modifier(ModifierInfo::new(
                    "Health",
                    ModifierOperation::AddBase,
                    MagnitudeCalculation::scalar(30.0),
                )),
        );

    let owner = {
        let mut commands = app.world_mut().commands();
        let owner = commands.spawn_empty().id();
        TestAttributeSet::create_attributes(&mut commands, owner);
        owner
    };

    app.world_mut()
        .trigger(ApplyGameplayEffectEvent::new("permanent_style_buff", owner).with_level(1));
    app.update();

    let health_attr = find_attribute_entity(app.world_mut(), owner, "Health");
    let attr = app
        .world()
        .entity(health_attr)
        .get::<AttributeData>()
        .unwrap();

    assert_eq!(attr.base_value, 100.0);
    assert_eq!(
        attr.current_value, 130.0,
        "AddBase modifiers should contribute during aggregation without mutating stored base_value"
    );
}
