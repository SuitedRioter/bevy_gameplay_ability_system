use bevy::prelude::*;
use bevy_gameplay_ability_system::{
    GasPlugin,
    abilities::*,
    core::{BlockedAbilityTags, OwnedTags},
};
use bevy_gameplay_tag::GameplayTagsPlugin;

#[test]
fn test_instanced_per_execution_creates_new_instance_each_time() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        GameplayTagsPlugin::with_data_path("assets/gameplay_tags.json".to_string()),
        GasPlugin,
    ));
    app.update();

    // Register ability with InstancedPerExecution policy (default)
    app.world_mut().resource_mut::<AbilityRegistry>().register(
        AbilityDefinition::new("test_ability")
            .with_instancing_policy(InstancingPolicy::InstancedPerExecution),
    );

    // Spawn owner and grant ability
    let owner = app
        .world_mut()
        .spawn((OwnedTags::default(), BlockedAbilityTags::default()))
        .id();

    let spec_entity = app
        .world_mut()
        .spawn((
            AbilitySpec::new("test_ability", 1),
            AbilityActiveState::default(),
            AbilityOwner(owner),
        ))
        .id();

    // First activation
    app.world_mut()
        .trigger(TryActivateAbilityEvent::new(spec_entity, owner));
    app.update();

    let first_instance = {
        let mut query = app
            .world_mut()
            .query::<(Entity, &AbilitySpecInstance, &InstanceControlState)>();
        let (entity, _, ctrl) = query.single(app.world()).expect("Should have one instance");
        assert!(ctrl.is_active, "First instance should be active");
        entity
    };

    // End first activation
    app.world_mut().trigger(EndAbilityEvent {
        instance: Some(first_instance),
        ability_spec: spec_entity,
        owner,
    });
    app.update();

    // First instance should be despawned
    assert!(
        app.world().get_entity(first_instance).is_err(),
        "InstancedPerExecution instance should be despawned after ending"
    );

    // Second activation
    app.world_mut()
        .trigger(TryActivateAbilityEvent::new(spec_entity, owner));
    app.update();

    let second_instance = {
        let mut query = app
            .world_mut()
            .query::<(Entity, &AbilitySpecInstance, &InstanceControlState)>();
        let (entity, _, ctrl) = query.single(app.world()).expect("Should have one instance");
        assert!(ctrl.is_active, "Second instance should be active");
        entity
    };

    // Second instance should be different from first
    assert_ne!(
        first_instance, second_instance,
        "InstancedPerExecution should create new instance each time"
    );
}

#[test]
fn test_instanced_per_actor_reuses_same_instance() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        GameplayTagsPlugin::with_data_path("assets/gameplay_tags.json".to_string()),
        GasPlugin,
    ));
    app.update();

    // Register ability with InstancedPerActor policy
    app.world_mut().resource_mut::<AbilityRegistry>().register(
        AbilityDefinition::new("test_ability")
            .with_instancing_policy(InstancingPolicy::InstancedPerActor),
    );

    // Spawn owner and grant ability
    let owner = app
        .world_mut()
        .spawn((OwnedTags::default(), BlockedAbilityTags::default()))
        .id();

    let spec_entity = app
        .world_mut()
        .spawn((
            AbilitySpec::new("test_ability", 1),
            AbilityActiveState::default(),
            AbilityOwner(owner),
        ))
        .id();

    // First activation
    app.world_mut()
        .trigger(TryActivateAbilityEvent::new(spec_entity, owner));
    app.update();

    let first_instance = {
        let mut query = app
            .world_mut()
            .query::<(Entity, &AbilitySpecInstance, &InstanceControlState)>();
        let (entity, _, ctrl) = query.single(app.world()).expect("Should have one instance");
        assert!(ctrl.is_active, "First instance should be active");
        entity
    };

    // End first activation
    app.world_mut().trigger(EndAbilityEvent {
        instance: Some(first_instance),
        ability_spec: spec_entity,
        owner,
    });
    app.update();

    // First instance should still exist but marked inactive
    {
        let mut query = app
            .world_mut()
            .query::<(Entity, &AbilitySpecInstance, &InstanceControlState)>();
        let (entity, _, ctrl) = query.single(app.world()).expect("Should have one instance");
        assert_eq!(
            entity, first_instance,
            "InstancedPerActor instance should not be despawned"
        );
        assert!(
            !ctrl.is_active,
            "Instance should be marked inactive after ending"
        );
    }

    // Second activation
    app.world_mut()
        .trigger(TryActivateAbilityEvent::new(spec_entity, owner));
    app.update();

    let second_instance = {
        let mut query = app
            .world_mut()
            .query::<(Entity, &AbilitySpecInstance, &InstanceControlState)>();
        let (entity, _, ctrl) = query.single(app.world()).expect("Should have one instance");
        assert!(ctrl.is_active, "Reused instance should be active again");
        entity
    };

    // Second instance should be the same as first
    assert_eq!(
        first_instance, second_instance,
        "InstancedPerActor should reuse the same instance"
    );
}

#[test]
fn test_non_instanced_has_no_instance_entity() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        GameplayTagsPlugin::with_data_path("assets/gameplay_tags.json".to_string()),
        GasPlugin,
    ));
    app.update();

    // Register ability with NonInstanced policy
    app.world_mut().resource_mut::<AbilityRegistry>().register(
        AbilityDefinition::new("test_ability")
            .with_instancing_policy(InstancingPolicy::NonInstanced),
    );

    // Spawn owner and grant ability
    let owner = app
        .world_mut()
        .spawn((OwnedTags::default(), BlockedAbilityTags::default()))
        .id();

    let spec_entity = app
        .world_mut()
        .spawn((
            AbilitySpec::new("test_ability", 1),
            AbilityActiveState::default(),
            AbilityOwner(owner),
        ))
        .id();

    // Activate ability
    app.world_mut()
        .trigger(TryActivateAbilityEvent::new(spec_entity, owner));
    app.update();

    // No instance entity should be created
    let instance_count = app
        .world_mut()
        .query::<&AbilitySpecInstance>()
        .iter(app.world())
        .count();

    assert_eq!(
        instance_count, 0,
        "NonInstanced ability should not create instance entity"
    );

    // Check that AbilityActivatedEvent was triggered with None instance
    // (This would require event capture, which we don't have in this simple test)
    // For now, we just verify no instance exists
}
