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

#[test]
fn test_refresh_duration_reapply_updates_persisted_spec_data() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        GameplayTagsPlugin::with_data_path("assets/gameplay_tags.json".to_string()),
        GasPlugin,
    ));
    app.update();

    let caller_tag = GameplayTag::new("Data.CallerValue");
    app.world_mut()
        .resource_mut::<GameplayEffectRegistry>()
        .register(
            GameplayEffectDefinition::new("refreshing_effect")
                .with_duration(5.0)
                .with_stacking_policy(StackingPolicy::RefreshDuration)
                .add_modifier(ModifierInfo::new(
                    "Health",
                    ModifierOperation::AddCurrent,
                    MagnitudeCalculation::set_by_caller(caller_tag.clone()),
                )),
        );

    let source_a = app.world_mut().spawn_empty().id();
    let source_b = app.world_mut().spawn_empty().id();
    let target = {
        let mut commands = app.world_mut().commands();
        let target = commands.spawn_empty().id();
        TestAttributeSet::create_attributes(&mut commands, target);
        target
    };
    app.update();

    app.world_mut().trigger(ApplyGameplayEffectEvent::from_spec(
        GameplayEffectSpec::new("refreshing_effect", target)
            .with_source(source_a)
            .with_instigator(source_a)
            .with_set_by_caller_magnitude(caller_tag.clone(), 5.0),
    ));
    app.update();

    app.world_mut().trigger(ApplyGameplayEffectEvent::from_spec(
        GameplayEffectSpec::new("refreshing_effect", target)
            .with_source(source_b)
            .with_instigator(source_b)
            .with_set_by_caller_magnitude(caller_tag.clone(), 10.0),
    ));
    app.update();

    let mut query = app.world_mut().query::<(
        &ActiveGameplayEffect,
        &EffectInstigator,
        &GameplayEffectContext,
        &SetByCallerMagnitudes,
        &EffectDuration,
    )>();
    let (active_effect, instigator, context, set_by_caller, duration) = query
        .single(app.world())
        .expect("refresh-duration policy should keep one active effect");

    assert_eq!(active_effect.stack_count, 1);
    assert_eq!(instigator.0, Some(source_b));
    assert_eq!(context.source, Some(source_b));
    assert_eq!(context.instigator, Some(source_b));
    assert_eq!(set_by_caller.get_magnitude(&caller_tag), Some(10.0));
    assert!((duration.remaining - 5.0).abs() < 0.01);
}

#[test]
fn test_stack_count_reapply_updates_persisted_spec_data() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        GameplayTagsPlugin::with_data_path("assets/gameplay_tags.json".to_string()),
        GasPlugin,
    ));
    app.update();

    let caller_tag = GameplayTag::new("Data.StackValue");
    app.world_mut()
        .resource_mut::<GameplayEffectRegistry>()
        .register(
            GameplayEffectDefinition::new("stacking_effect")
                .with_duration(5.0)
                .with_stacking_policy(StackingPolicy::StackCount { max_stacks: 3 })
                .add_modifier(ModifierInfo::new(
                    "Health",
                    ModifierOperation::AddCurrent,
                    MagnitudeCalculation::set_by_caller(caller_tag.clone()),
                )),
        );

    let source_a = app.world_mut().spawn_empty().id();
    let source_b = app.world_mut().spawn_empty().id();
    let target = {
        let mut commands = app.world_mut().commands();
        let target = commands.spawn_empty().id();
        TestAttributeSet::create_attributes(&mut commands, target);
        target
    };
    app.update();

    app.world_mut().trigger(ApplyGameplayEffectEvent::from_spec(
        GameplayEffectSpec::new("stacking_effect", target)
            .with_source(source_a)
            .with_instigator(source_a)
            .with_set_by_caller_magnitude(caller_tag.clone(), 5.0),
    ));
    app.update();

    app.world_mut().trigger(ApplyGameplayEffectEvent::from_spec(
        GameplayEffectSpec::new("stacking_effect", target)
            .with_source(source_b)
            .with_instigator(source_b)
            .with_set_by_caller_magnitude(caller_tag.clone(), 9.0),
    ));
    app.update();

    let mut query = app.world_mut().query::<(
        &ActiveGameplayEffect,
        &EffectInstigator,
        &GameplayEffectContext,
        &SetByCallerMagnitudes,
        &EffectDuration,
    )>();
    let (active_effect, instigator, context, set_by_caller, duration) = query
        .single(app.world())
        .expect("stack-count policy should keep one active effect entity");

    assert_eq!(active_effect.stack_count, 2);
    assert_eq!(instigator.0, Some(source_b));
    assert_eq!(context.source, Some(source_b));
    assert_eq!(context.instigator, Some(source_b));
    assert_eq!(set_by_caller.get_magnitude(&caller_tag), Some(9.0));
    assert!((duration.remaining - 5.0).abs() < 0.01);
}
