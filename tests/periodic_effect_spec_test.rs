use bevy::ecs::relationship::Relationship;
use bevy::prelude::*;
use bevy_gameplay_ability_system::{GasPlugin, attributes::*, effects::*};
use bevy_gameplay_tag::{GameplayTag, GameplayTagsPlugin};

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

fn attribute_value(world: &mut World, owner: Entity, attribute: &str) -> f32 {
    let mut query = world.query::<(&AttributeData, &AttributeName, &ChildOf)>();
    query
        .iter(world)
        .find(|(_, name, child_of)| child_of.get() == owner && name.as_str() == attribute)
        .map(|(data, _, _)| data.current_value)
        .expect("attribute should exist")
}

fn set_periodic_ready(world: &mut World) {
    let mut query = world.query::<&mut PeriodicEffect>();
    let mut periodic = query
        .single_mut(world)
        .expect("expected exactly one periodic effect");
    periodic.time_until_next = 0.0;
}

#[test]
fn test_periodic_effect_uses_persisted_set_by_caller() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        GameplayTagsPlugin::with_data_path("assets/gameplay_tags.json".to_string()),
        GasPlugin,
    ));
    app.update();

    let tick_damage = GameplayTag::new("Data.TickDamage");
    app.world_mut()
        .resource_mut::<GameplayEffectRegistry>()
        .register(
            GameplayEffectDefinition::new("periodic_caller_damage")
                .with_duration(5.0)
                .with_period(1.0)
                .add_modifier(ModifierInfo::new(
                    "Health",
                    ModifierOperation::AddCurrent,
                    MagnitudeCalculation::set_by_caller(tick_damage.clone()),
                )),
        );

    let target = {
        let mut commands = app.world_mut().commands();
        let target = commands.spawn_empty().id();
        TestAttributeSet::create_attributes(&mut commands, target);
        target
    };
    app.update();

    app.world_mut().trigger(ApplyGameplayEffectEvent::from_spec(
        GameplayEffectSpec::new("periodic_caller_damage", target)
            .with_set_by_caller_magnitude(tick_damage, -12.0),
    ));
    app.update();

    set_periodic_ready(app.world_mut());
    app.update();

    assert_eq!(attribute_value(app.world_mut(), target, "Health"), 88.0);
}

#[test]
fn test_periodic_effect_uses_persisted_source_context_for_attribute_based() {
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
            GameplayEffectDefinition::new("periodic_attack_damage")
                .with_duration(5.0)
                .with_period(1.0)
                .add_modifier(ModifierInfo::new(
                    "Health",
                    ModifierOperation::AddCurrent,
                    MagnitudeCalculation::from_source_attribute("AttackPower", -1.0),
                )),
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

    app.world_mut().trigger(
        ApplyGameplayEffectEvent::new("periodic_attack_damage", target)
            .with_source(source)
            .with_instigator(source)
            .with_level(1),
    );
    app.update();

    set_periodic_ready(app.world_mut());
    app.update();

    assert_eq!(attribute_value(app.world_mut(), target, "Health"), 90.0);
}
