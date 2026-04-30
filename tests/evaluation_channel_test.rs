use bevy::prelude::*;
use bevy_gameplay_ability_system::{attributes::*, effects::*, GasPlugin};
use bevy_gameplay_tag::GameplayTagsPlugin;
use string_cache::DefaultAtom as Atom;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct TestAttributeSet;

impl AttributeSetDefinition for TestAttributeSet {
    fn attribute_names() -> &'static [&'static str] {
        &["Damage", "Health", "Attack"]
    }

    fn attribute_metadata(name: &str) -> Option<AttributeMetadata> {
        match name {
            "Damage" => Some(AttributeMetadata::new("Damage").with_min(0.0)),
            "Health" => Some(AttributeMetadata::new("Health").with_min(0.0).with_max(1000.0)),
            "Attack" => Some(AttributeMetadata::new("Attack").with_min(0.0)),
            _ => None,
        }
    }

    fn default_value(name: &str) -> f32 {
        match name {
            "Damage" => 100.0,
            "Health" => 100.0,
            "Attack" => 50.0,
            _ => 0.0,
        }
    }
}

/// Test that modifiers in different channels are evaluated in order.
#[test]
fn test_channel_evaluation_order() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        GameplayTagsPlugin::with_data_path("assets/gameplay_tags.json".to_string()),
        GasPlugin,
    ));
    app.update();

    let owner = {
        let mut commands = app.world_mut().commands();
        let owner = commands.spawn_empty().id();
        TestAttributeSet::create_attributes(&mut commands, owner);
        owner
    };

    app.world_mut()
        .resource_mut::<GameplayEffectRegistry>()
        .register(
            GameplayEffectDefinition::new("channel0_effect")
                .with_duration_policy(DurationPolicy::Infinite)
                .add_modifier(
                    ModifierInfo::new(
                        "Damage",
                        ModifierOperation::AddCurrent,
                        MagnitudeCalculation::scalar(20.0),
                    )
                    .with_channel(EvaluationChannel::Channel0),
                ),
        );

    app.world_mut()
        .resource_mut::<GameplayEffectRegistry>()
        .register(
            GameplayEffectDefinition::new("channel1_effect")
                .with_duration_policy(DurationPolicy::Infinite)
                .add_modifier(
                    ModifierInfo::new(
                        "Damage",
                        ModifierOperation::MultiplyAdditive,
                        MagnitudeCalculation::scalar(0.5),
                    )
                    .with_channel(EvaluationChannel::Channel1),
                ),
        );

    app.world_mut()
        .resource_mut::<GameplayEffectRegistry>()
        .register(
            GameplayEffectDefinition::new("channel2_effect")
                .with_duration_policy(DurationPolicy::Infinite)
                .add_modifier(
                    ModifierInfo::new(
                        "Damage",
                        ModifierOperation::AddCurrent,
                        MagnitudeCalculation::scalar(30.0),
                    )
                    .with_channel(EvaluationChannel::Channel2),
                ),
        );

    app.world_mut()
        .trigger(ApplyGameplayEffectEvent::new("channel0_effect", owner));
    app.world_mut()
        .trigger(ApplyGameplayEffectEvent::new("channel1_effect", owner));
    app.world_mut()
        .trigger(ApplyGameplayEffectEvent::new("channel2_effect", owner));

    app.update();

    let damage_value = app
        .world_mut()
        .query::<(&AttributeData, &AttributeName)>()
        .iter(app.world())
        .find(|(_, name)| name.0 == Atom::from("Damage"))
        .map(|(data, _)| data.current_value)
        .expect("Damage attribute should exist");

    assert!(
        (damage_value - 210.0).abs() < 0.01,
        "Expected 210.0, got {}. Formula: ((100 + 20) * 1.5) + 30 = 210",
        damage_value
    );
}

/// Test that multiple modifiers in the same channel are combined correctly.
#[test]
fn test_same_channel_modifier_combination() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        GameplayTagsPlugin::with_data_path("assets/gameplay_tags.json".to_string()),
        GasPlugin,
    ));
    app.update();

    let owner = {
        let mut commands = app.world_mut().commands();
        let owner = commands.spawn_empty().id();
        TestAttributeSet::create_attributes(&mut commands, owner);
        owner
    };

    app.world_mut()
        .resource_mut::<GameplayEffectRegistry>()
        .register(
            GameplayEffectDefinition::new("multi_modifier_effect")
                .with_duration_policy(DurationPolicy::Infinite)
                .add_modifier(
                    ModifierInfo::new(
                        "Health",
                        ModifierOperation::AddCurrent,
                        MagnitudeCalculation::scalar(10.0),
                    )
                    .with_channel(EvaluationChannel::Channel0),
                )
                .add_modifier(
                    ModifierInfo::new(
                        "Health",
                        ModifierOperation::AddCurrent,
                        MagnitudeCalculation::scalar(20.0),
                    )
                    .with_channel(EvaluationChannel::Channel0),
                )
                .add_modifier(
                    ModifierInfo::new(
                        "Health",
                        ModifierOperation::MultiplyAdditive,
                        MagnitudeCalculation::scalar(0.5),
                    )
                    .with_channel(EvaluationChannel::Channel0),
                ),
        );

    app.world_mut()
        .trigger(ApplyGameplayEffectEvent::new(
            "multi_modifier_effect",
            owner,
        ));

    app.update();

    let health_value = app
        .world_mut()
        .query::<(&AttributeData, &AttributeName)>()
        .iter(app.world())
        .find(|(_, name)| name.0 == Atom::from("Health"))
        .map(|(data, _)| data.current_value)
        .expect("Health attribute should exist");

    assert!(
        (health_value - 195.0).abs() < 0.01,
        "Expected 195.0, got {}. Formula: (100 + 10 + 20) * 1.5 = 195",
        health_value
    );
}

/// Test complex buff/debuff stacking with multiple channels.
#[test]
fn test_complex_buff_debuff_stacking() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        GameplayTagsPlugin::with_data_path("assets/gameplay_tags.json".to_string()),
        GasPlugin,
    ));
    app.update();

    let owner = {
        let mut commands = app.world_mut().commands();
        let owner = commands.spawn_empty().id();
        TestAttributeSet::create_attributes(&mut commands, owner);
        owner
    };

    app.world_mut()
        .resource_mut::<GameplayEffectRegistry>()
        .register(
            GameplayEffectDefinition::new("weapon_bonus")
                .with_duration_policy(DurationPolicy::Infinite)
                .add_modifier(
                    ModifierInfo::new(
                        "Attack",
                        ModifierOperation::AddCurrent,
                        MagnitudeCalculation::scalar(30.0),
                    )
                    .with_channel(EvaluationChannel::Channel0),
                ),
        );

    app.world_mut()
        .resource_mut::<GameplayEffectRegistry>()
        .register(
            GameplayEffectDefinition::new("strength_bonus")
                .with_duration_policy(DurationPolicy::Infinite)
                .add_modifier(
                    ModifierInfo::new(
                        "Attack",
                        ModifierOperation::AddCurrent,
                        MagnitudeCalculation::scalar(20.0),
                    )
                    .with_channel(EvaluationChannel::Channel0),
                ),
        );

    app.world_mut()
        .resource_mut::<GameplayEffectRegistry>()
        .register(
            GameplayEffectDefinition::new("buff")
                .with_duration_policy(DurationPolicy::Infinite)
                .add_modifier(
                    ModifierInfo::new(
                        "Attack",
                        ModifierOperation::MultiplyAdditive,
                        MagnitudeCalculation::scalar(0.5),
                    )
                    .with_channel(EvaluationChannel::Channel1),
                ),
        );

    app.world_mut()
        .resource_mut::<GameplayEffectRegistry>()
        .register(
            GameplayEffectDefinition::new("debuff")
                .with_duration_policy(DurationPolicy::Infinite)
                .add_modifier(
                    ModifierInfo::new(
                        "Attack",
                        ModifierOperation::MultiplyAdditive,
                        MagnitudeCalculation::scalar(-0.25),
                    )
                    .with_channel(EvaluationChannel::Channel1),
                ),
        );

    app.world_mut()
        .resource_mut::<GameplayEffectRegistry>()
        .register(
            GameplayEffectDefinition::new("critical_hit")
                .with_duration_policy(DurationPolicy::Infinite)
                .add_modifier(
                    ModifierInfo::new(
                        "Attack",
                        ModifierOperation::MultiplyAdditive,
                        MagnitudeCalculation::scalar(1.0),
                    )
                    .with_channel(EvaluationChannel::Channel2),
                ),
        );

    app.world_mut()
        .resource_mut::<GameplayEffectRegistry>()
        .register(
            GameplayEffectDefinition::new("final_bonus")
                .with_duration_policy(DurationPolicy::Infinite)
                .add_modifier(
                    ModifierInfo::new(
                        "Attack",
                        ModifierOperation::AddCurrent,
                        MagnitudeCalculation::scalar(15.0),
                    )
                    .with_channel(EvaluationChannel::Channel3),
                ),
        );

    app.world_mut()
        .trigger(ApplyGameplayEffectEvent::new("weapon_bonus", owner));
    app.world_mut()
        .trigger(ApplyGameplayEffectEvent::new("strength_bonus", owner));
    app.world_mut()
        .trigger(ApplyGameplayEffectEvent::new("buff", owner));
    app.world_mut()
        .trigger(ApplyGameplayEffectEvent::new("debuff", owner));
    app.world_mut()
        .trigger(ApplyGameplayEffectEvent::new("critical_hit", owner));
    app.world_mut()
        .trigger(ApplyGameplayEffectEvent::new("final_bonus", owner));

    app.update();

    let attack_value = app
        .world_mut()
        .query::<(&AttributeData, &AttributeName)>()
        .iter(app.world())
        .find(|(_, name)| name.0 == Atom::from("Attack"))
        .map(|(data, _)| data.current_value)
        .expect("Attack attribute should exist");

    assert!(
        (attack_value - 265.0).abs() < 0.01,
        "Expected 265.0, got {}. Formula: (((50+30+20)*1.25)*2.0)+15 = 265",
        attack_value
    );
}
