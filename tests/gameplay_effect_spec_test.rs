use bevy::ecs::relationship::Relationship;
use bevy::prelude::*;
use bevy_gameplay_ability_system::{GasPlugin, attributes::*, effects::*};
use bevy_gameplay_tag::{GameplayTag, GameplayTagsPlugin};

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

fn health(world: &mut World, owner: Entity) -> f32 {
    let mut query = world.query::<(&AttributeData, &AttributeName, &ChildOf)>();
    query
        .iter(world)
        .find(|(_, name, child_of)| child_of.get() == owner && name.as_str() == "Health")
        .map(|(data, _, _)| data.current_value)
        .expect("health attribute should exist")
}

#[test]
fn test_instant_effect_uses_set_by_caller_from_spec() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        GameplayTagsPlugin::with_data_path("assets/gameplay_tags.json".to_string()),
        GasPlugin,
    ));
    app.update();

    let damage_tag = GameplayTag::new("Data.Damage");
    app.world_mut()
        .resource_mut::<GameplayEffectRegistry>()
        .register(
            GameplayEffectDefinition::new("caller_damage").add_modifier(ModifierInfo::new(
                "Health",
                ModifierOperation::AddCurrent,
                MagnitudeCalculation::set_by_caller(damage_tag.clone()),
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
        GameplayEffectSpec::new("caller_damage", target)
            .with_set_by_caller_magnitude(damage_tag, -35.0),
    ));
    app.update();

    assert_eq!(health(app.world_mut(), target), 65.0);
}

#[test]
fn test_duration_effect_persists_spec_context_and_set_by_caller() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        GameplayTagsPlugin::with_data_path("assets/gameplay_tags.json".to_string()),
        GasPlugin,
    ));
    app.update();

    let buff_tag = GameplayTag::new("Data.Buff");
    app.world_mut()
        .resource_mut::<GameplayEffectRegistry>()
        .register(
            GameplayEffectDefinition::new("caller_buff")
                .with_duration(5.0)
                .add_modifier(ModifierInfo::new(
                    "Health",
                    ModifierOperation::AddCurrent,
                    MagnitudeCalculation::set_by_caller(buff_tag.clone()),
                )),
        );

    let source = app.world_mut().spawn_empty().id();
    let target = {
        let mut commands = app.world_mut().commands();
        let target = commands.spawn_empty().id();
        TestAttributeSet::create_attributes(&mut commands, target);
        target
    };
    app.update();

    app.world_mut().trigger(ApplyGameplayEffectEvent::from_spec(
        GameplayEffectSpec::new("caller_buff", target)
            .with_source(source)
            .with_instigator(source)
            .with_level(7)
            .with_set_by_caller_magnitude(buff_tag, 20.0),
    ));
    app.update();

    assert_eq!(health(app.world_mut(), target), 120.0);

    let mut query = app.world_mut().query::<(
        &ActiveGameplayEffect,
        &EffectInstigator,
        &GameplayEffectContext,
        &SetByCallerMagnitudes,
    )>();
    let (active_effect, instigator, context, magnitudes) = query
        .single(app.world())
        .expect("duration effect should persist runtime spec data");

    assert_eq!(active_effect.level, 7);
    assert_eq!(instigator.0, Some(source));
    assert_eq!(context.source, Some(source));
    assert_eq!(context.instigator, Some(source));
    assert!(!magnitudes.is_empty());
}
