use bevy::ecs::relationship::Relationship;
use bevy::prelude::*;
use bevy_gameplay_ability_system::{GasPlugin, attributes::*, effects::*};
use bevy_gameplay_tag::GameplayTagsPlugin;
use string_cache::DefaultAtom as Atom;

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

struct MinTargetHealthRequirement(f32);

impl ApplicationRequirement for MinTargetHealthRequirement {
    fn can_apply(&self, ctx: &ApplicationContext) -> bool {
        ctx.get_target_attribute(&Atom::from("Health"))
            .is_some_and(|health| health >= self.0)
    }
}

fn get_health(world: &mut World, owner: Entity) -> f32 {
    let mut query = world.query::<(&AttributeData, &AttributeName, &ChildOf)>();
    query
        .iter(world)
        .find(|(_, name, child_of)| child_of.get() == owner && name.as_str() == "Health")
        .map(|(data, _, _)| data.current_value)
        .expect("health attribute should exist")
}

#[test]
fn test_application_requirement_blocks_effect_before_attribute_mutation() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        GameplayTagsPlugin::with_data_path("assets/gameplay_tags.json".to_string()),
        GasPlugin,
    ));
    app.update();

    app.world_mut()
        .resource_mut::<ApplicationRequirementRegistry>()
        .register("needs_high_health", Box::new(MinTargetHealthRequirement(150.0)));

    app.world_mut()
        .resource_mut::<GameplayEffectRegistry>()
        .register(
            GameplayEffectDefinition::new("conditional_damage")
                .add_application_requirement("needs_high_health")
                .add_modifier(ModifierInfo::new(
                    "Health",
                    ModifierOperation::AddCurrent,
                    MagnitudeCalculation::scalar(-25.0),
                )),
        );

    let target = {
        let mut commands = app.world_mut().commands();
        let target = commands.spawn_empty().id();
        TestAttributeSet::create_attributes(&mut commands, target);
        target
    };

    app.update();

    app.world_mut().trigger(ApplyGameplayEffectEvent {
        effect_id: "conditional_damage".into(),
        target,
        instigator: None,
        level: 1,
    });
    app.update();

    assert_eq!(get_health(app.world_mut(), target), 100.0);
}

#[test]
fn test_application_requirement_allows_effect_when_condition_passes() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        GameplayTagsPlugin::with_data_path("assets/gameplay_tags.json".to_string()),
        GasPlugin,
    ));
    app.update();

    app.world_mut()
        .resource_mut::<ApplicationRequirementRegistry>()
        .register("needs_low_health", Box::new(MinTargetHealthRequirement(50.0)));

    app.world_mut()
        .resource_mut::<GameplayEffectRegistry>()
        .register(
            GameplayEffectDefinition::new("conditional_damage")
                .add_application_requirement("needs_low_health")
                .add_modifier(ModifierInfo::new(
                    "Health",
                    ModifierOperation::AddCurrent,
                    MagnitudeCalculation::scalar(-25.0),
                )),
        );

    let target = {
        let mut commands = app.world_mut().commands();
        let target = commands.spawn_empty().id();
        TestAttributeSet::create_attributes(&mut commands, target);
        target
    };

    app.update();

    app.world_mut().trigger(ApplyGameplayEffectEvent {
        effect_id: "conditional_damage".into(),
        target,
        instigator: None,
        level: 1,
    });
    app.update();

    assert_eq!(get_health(app.world_mut(), target), 75.0);
}
