//! Test for StackCount stacking policy.
//!
//! Verifies that modifiers are correctly spawned and removed as stacks increase/decrease.

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
                    .with_max(100.0),
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

#[test]
fn test_stack_count_spawns_correct_modifiers() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        GameplayTagsPlugin::with_data_path("assets/gameplay_tags.json".to_string()),
        GasPlugin,
    ));

    // Wait for tags to load
    app.update();

    // Register effect with StackCount policy
    app.world_mut()
        .resource_mut::<GameplayEffectRegistry>()
        .register(
            GameplayEffectDefinition::new("stack_buff")
                .with_duration_policy(DurationPolicy::HasDuration)
                .with_duration(5.0)
                .with_stacking_policy(StackingPolicy::StackCount { max_stacks: 3 })
                .add_modifier(ModifierInfo::new(
                    "AttackPower",
                    ModifierOperation::AddCurrent,
                    MagnitudeCalculation::scalar(5.0),
                )),
        );

    // Create target entity
    let target = {
        let mut commands = app.world_mut().commands();
        let target = commands.spawn_empty().id();
        TestAttributeSet::create_attributes(&mut commands, target);
        target
    };

    // Apply effect first time (stack = 1)
    app.world_mut()
        .trigger(ApplyGameplayEffectEvent::new("stack_buff", target).with_level(1));

    app.update();

    // Check: should have 1 modifier
    let modifier_count = app
        .world_mut()
        .query::<&AttributeModifier>()
        .iter(app.world())
        .count();
    assert_eq!(
        modifier_count, 1,
        "Should have 1 modifier after first application"
    );

    // Apply effect second time (stack = 2)
    app.world_mut()
        .trigger(ApplyGameplayEffectEvent::new("stack_buff", target).with_level(1));

    app.update();

    // Check: should have 2 modifiers
    let modifier_count = app
        .world_mut()
        .query::<&AttributeModifier>()
        .iter(app.world())
        .count();
    assert_eq!(
        modifier_count, 2,
        "Should have 2 modifiers after second application"
    );

    // Apply effect third time (stack = 3, max)
    app.world_mut()
        .trigger(ApplyGameplayEffectEvent::new("stack_buff", target).with_level(1));

    app.update();

    // Check: should have 3 modifiers
    let modifier_count = app
        .world_mut()
        .query::<&AttributeModifier>()
        .iter(app.world())
        .count();
    assert_eq!(modifier_count, 3, "Should have 3 modifiers at max stacks");

    // Try to apply fourth time (should not increase beyond max)
    app.world_mut()
        .trigger(ApplyGameplayEffectEvent::new("stack_buff", target).with_level(1));

    app.update();

    // Check: should still have 3 modifiers
    let modifier_count = app
        .world_mut()
        .query::<&AttributeModifier>()
        .iter(app.world())
        .count();
    assert_eq!(modifier_count, 3, "Should not exceed max stacks");
}

#[test]
fn test_stack_count_removes_modifiers_on_decrease() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        GameplayTagsPlugin::with_data_path("assets/gameplay_tags.json".to_string()),
        GasPlugin,
    ));

    app.update();

    // Register effect
    app.world_mut()
        .resource_mut::<GameplayEffectRegistry>()
        .register(
            GameplayEffectDefinition::new("stack_buff_removable")
                .with_duration_policy(DurationPolicy::HasDuration)
                .with_duration(5.0)
                .with_stacking_policy(StackingPolicy::StackCount { max_stacks: 5 })
                .add_modifier(ModifierInfo::new(
                    "AttackPower",
                    ModifierOperation::AddCurrent,
                    MagnitudeCalculation::scalar(5.0),
                )),
        );

    let target = {
        let mut commands = app.world_mut().commands();
        let target = commands.spawn_empty().id();
        TestAttributeSet::create_attributes(&mut commands, target);
        target
    };

    // Apply effect 3 times
    for _ in 0..3 {
        app.world_mut()
            .trigger(ApplyGameplayEffectEvent::new("stack_buff_removable", target).with_level(1));
        app.update();
    }

    // Verify 3 modifiers
    let modifier_count = app
        .world_mut()
        .query::<&AttributeModifier>()
        .iter(app.world())
        .count();
    assert_eq!(modifier_count, 3, "Should have 3 modifiers");

    // Manually decrease stack count
    let effect_entity = {
        let world = app.world_mut();
        let mut query = world.query_filtered::<Entity, With<ActiveGameplayEffect>>();
        let entities: Vec<Entity> = query.iter(world).collect();
        entities[0]
    };

    {
        let world = app.world_mut();
        let mut effect = world.entity_mut(effect_entity);
        effect
            .get_mut::<ActiveGameplayEffect>()
            .unwrap()
            .stack_count = 1;
    }

    app.update();

    // Check: should now have only 1 modifier
    let modifier_count = app
        .world_mut()
        .query::<&AttributeModifier>()
        .iter(app.world())
        .count();
    assert_eq!(
        modifier_count, 1,
        "Should have 1 modifier after stack decrease"
    );
}
