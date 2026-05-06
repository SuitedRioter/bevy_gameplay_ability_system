//! Entity lifecycle safety tests.
//!
//! Tests to verify that Bevy 0.18's ChildOf relationship correctly cleans up
//! child entities when the parent is despawned.
//!
//! After migrating Effects and Abilities to use ChildOf, all child entities
//! should be automatically cleaned up when the owner is despawned.

use bevy::ecs::relationship::Relationship;
use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use bevy_gameplay_ability_system::abilities::grant_ability;
use bevy_gameplay_ability_system::core::{BlockedAbilityTags, OwnedTags};
use bevy_gameplay_ability_system::prelude::*;
use bevy_gameplay_tag::{GameplayTagCountContainer, GameplayTagsPlugin};

/// Test attribute set for lifecycle tests.
struct TestAttributes;

impl AttributeSetDefinition for TestAttributes {
    fn attribute_names() -> &'static [&'static str] {
        &["Health", "Mana"]
    }

    fn attribute_metadata(name: &str) -> Option<AttributeMetadata> {
        match name {
            "Health" => Some(
                AttributeMetadata::new("Health")
                    .with_min(0.0)
                    .with_max(100.0),
            ),
            "Mana" => Some(AttributeMetadata::new("Mana").with_min(0.0).with_max(100.0)),
            _ => None,
        }
    }

    fn default_value(name: &str) -> f32 {
        match name {
            "Health" => 100.0,
            "Mana" => 100.0,
            _ => 0.0,
        }
    }
}

#[test]
fn test_attributes_cleaned_up_on_owner_despawn() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(GameplayTagsPlugin::with_data_path(
            "assets/gameplay_tags.json".to_string(),
        ))
        .add_plugins(GasPlugin);

    app.update();

    // Spawn owner with attributes
    let owner = app
        .world_mut()
        .run_system_once(|mut commands: Commands| {
            let owner = commands.spawn_empty().id();
            TestAttributes::create_attributes(&mut commands, owner);
            owner
        })
        .expect("Failed to spawn owner");

    app.update();

    // Verify attributes exist
    let attribute_count = app
        .world_mut()
        .run_system_once(|attributes: Query<&AttributeName>| {
            attributes
                .iter()
                .filter(|name| name.as_str() == "Health" || name.as_str() == "Mana")
                .count()
        })
        .expect("Failed to count attributes");
    assert_eq!(
        attribute_count, 2,
        "Should have 2 attributes before despawn"
    );

    // Despawn owner (Bevy 0.18 automatically despawns children via ChildOf)
    app.world_mut().despawn(owner);
    app.update();

    // Verify attributes are cleaned up
    let attribute_count_after = app
        .world_mut()
        .run_system_once(|attributes: Query<&AttributeName>| {
            attributes
                .iter()
                .filter(|name| name.as_str() == "Health" || name.as_str() == "Mana")
                .count()
        })
        .expect("Failed to count attributes after despawn");
    assert_eq!(
        attribute_count_after, 0,
        "Attributes should be cleaned up after owner despawn (via ChildOf)"
    );
}

#[test]
fn test_child_of_relationship_cleanup() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // Spawn parent and child with ChildOf relationship
    let (parent, child) = app
        .world_mut()
        .run_system_once(|mut commands: Commands| {
            let parent = commands.spawn_empty().id();
            let child = commands.spawn(ChildOf(parent)).id();
            (parent, child)
        })
        .expect("Failed to spawn entities");

    app.update();

    // Verify both exist
    assert!(
        app.world().get_entity(parent).is_ok(),
        "Parent should exist"
    );
    assert!(app.world().get_entity(child).is_ok(), "Child should exist");

    // Despawn parent
    app.world_mut().despawn(parent);
    app.update();

    // Verify both are cleaned up
    assert!(
        app.world().get_entity(parent).is_err(),
        "Parent should be despawned"
    );
    assert!(
        app.world().get_entity(child).is_err(),
        "Child should be auto-despawned via ChildOf"
    );
}

#[test]
fn test_multiple_children_cleanup() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(GameplayTagsPlugin::with_data_path(
            "assets/gameplay_tags.json".to_string(),
        ))
        .add_plugins(GasPlugin);

    app.update();

    // Spawn owner with multiple attributes
    let owner = app
        .world_mut()
        .run_system_once(|mut commands: Commands| {
            let owner = commands.spawn_empty().id();
            TestAttributes::create_attributes(&mut commands, owner);
            owner
        })
        .expect("Failed to spawn owner");

    app.update();

    // Count all attribute entities
    let total_attributes = app
        .world_mut()
        .run_system_once(|attributes: Query<&AttributeName>| attributes.iter().count())
        .expect("Failed to count attributes");
    assert!(total_attributes >= 2, "Should have at least 2 attributes");

    // Despawn owner
    app.world_mut().despawn(owner);
    app.update();

    // Verify all attributes are cleaned up
    let remaining_attributes = app
        .world_mut()
        .run_system_once(|attributes: Query<&AttributeName>| {
            attributes
                .iter()
                .filter(|name| name.as_str() == "Health" || name.as_str() == "Mana")
                .count()
        })
        .expect("Failed to count remaining attributes");
    assert_eq!(
        remaining_attributes, 0,
        "All attributes should be cleaned up"
    );
}

#[test]
fn test_effects_cleaned_up_with_childof() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(GameplayTagsPlugin::with_data_path(
            "assets/gameplay_tags.json".to_string(),
        ))
        .add_plugins(GasPlugin);

    app.update();

    // Register effect
    app.world_mut()
        .run_system_once(|mut registry: ResMut<GameplayEffectRegistry>| {
            let effect = GameplayEffectDefinition::new("test.effect")
                .with_duration_policy(DurationPolicy::Infinite)
                .add_modifier(ModifierInfo {
                    attribute_name: "Health".into(),
                    operation: ModifierOperation::AddCurrent,
                    magnitude: MagnitudeCalculation::scalar(10.0),
                    channel: EvaluationChannel::Channel0,
                });
            registry.register(effect);
        })
        .expect("Failed to register effect");

    // Spawn owner with attributes
    let owner = app
        .world_mut()
        .run_system_once(|mut commands: Commands| {
            let owner = commands.spawn_empty().id();
            TestAttributes::create_attributes(&mut commands, owner);
            owner
        })
        .expect("Failed to spawn owner");

    app.update();

    // Apply effect
    app.world_mut()
        .run_system_once(move |mut commands: Commands| {
            commands.trigger(ApplyGameplayEffectEvent::new("test.effect", owner));
        })
        .expect("Failed to apply effect");

    app.update();

    // Verify effect exists
    let effect_count = app
        .world_mut()
        .run_system_once(|effects: Query<&ActiveGameplayEffect>| effects.iter().count())
        .expect("Failed to count effects");
    assert_eq!(effect_count, 1, "Should have 1 effect before despawn");

    // Despawn owner
    app.world_mut().despawn(owner);
    app.update();

    // Verify effects are cleaned up (via ChildOf)
    let effect_count_after = app
        .world_mut()
        .run_system_once(|effects: Query<&ActiveGameplayEffect>| effects.iter().count())
        .expect("Failed to count effects after despawn");
    assert_eq!(
        effect_count_after, 0,
        "Effects should be auto-cleaned via ChildOf relationship"
    );
}

#[test]
fn test_abilities_cleaned_up_with_childof() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(GameplayTagsPlugin::with_data_path(
            "assets/gameplay_tags.json".to_string(),
        ))
        .add_plugins(GasPlugin);

    app.update();

    // Register ability
    app.world_mut()
        .run_system_once(|mut registry: ResMut<AbilityRegistry>| {
            let ability = AbilityDefinition::new("test.ability");
            registry.register(ability);
        })
        .expect("Failed to register ability");

    // Spawn owner
    let owner = app
        .world_mut()
        .run_system_once(|mut commands: Commands| {
            commands
                .spawn((
                    OwnedTags(GameplayTagCountContainer::default()),
                    BlockedAbilityTags(GameplayTagCountContainer::default()),
                ))
                .id()
        })
        .expect("Failed to spawn owner");

    app.update();

    // Grant ability using helper function
    let _ability_spec = app
        .world_mut()
        .run_system_once(move |mut commands: Commands| {
            grant_ability(&mut commands, owner, "test.ability", 1)
        })
        .expect("Failed to grant ability");

    app.update();

    // Verify ability exists
    let ability_count = app
        .world_mut()
        .run_system_once(|abilities: Query<&AbilitySpec>| abilities.iter().count())
        .expect("Failed to count abilities");
    assert_eq!(ability_count, 1, "Should have 1 ability before despawn");

    // Despawn owner
    app.world_mut().despawn(owner);
    app.update();

    // Verify abilities are cleaned up (via ChildOf)
    let ability_count_after = app
        .world_mut()
        .run_system_once(|abilities: Query<&AbilitySpec>| abilities.iter().count())
        .expect("Failed to count abilities after despawn");
    assert_eq!(
        ability_count_after, 0,
        "Abilities should be auto-cleaned via ChildOf relationship"
    );
}
