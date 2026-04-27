use bevy::prelude::*;
use bevy_gameplay_ability_system::{GasPlugin, abilities::*, core::OwnedTags, effects::*};
use bevy_gameplay_tag::GameplayTagsPlugin;

#[test]
fn test_effect_grants_and_removes_ability_on_expiration() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        GameplayTagsPlugin::with_data_path("assets/gameplay_tags.json".to_string()),
        GasPlugin,
    ));
    app.update();

    app.world_mut()
        .resource_mut::<AbilityRegistry>()
        .register(AbilityDefinition::new("temporary_ability"));

    app.world_mut()
        .resource_mut::<GameplayEffectRegistry>()
        .register(
            GameplayEffectDefinition::new("temporary_grant")
                .with_duration(5.0)
                .grant_ability(GrantedAbilityConfig::new("temporary_ability")),
        );

    let target = app.world_mut().spawn(OwnedTags::default()).id();

    app.world_mut().trigger(ApplyGameplayEffectEvent {
        effect_id: "temporary_grant".into(),
        target,
        instigator: None,
        level: 3,
    });
    app.update();

    let granted_ability = {
        let mut query = app
            .world_mut()
            .query::<(Entity, &AbilitySpec, &AbilityOwner)>();
        query
            .iter(app.world())
            .find(|(_, spec, owner)| {
                owner.0 == target && spec.definition_id.as_ref() == "temporary_ability"
            })
            .map(|(entity, spec, _)| {
                assert_eq!(spec.level, 3);
                entity
            })
            .expect("duration effect should grant the configured ability")
    };

    {
        let mut query = app.world_mut().query::<&mut EffectDuration>();
        let mut duration = query
            .single_mut(app.world_mut())
            .expect("temporary grant effect should have a duration");
        duration.remaining = 0.0;
    }

    app.update();

    assert!(
        app.world().get_entity(granted_ability).is_err(),
        "expired effect should remove granted ability with default CancelAbilityImmediately policy"
    );
}
