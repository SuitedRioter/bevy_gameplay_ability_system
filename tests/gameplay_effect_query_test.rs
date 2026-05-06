//! Integration tests for GameplayEffectQuery system.
//!
//! Tests the query system used by ImmunityComponent, RemoveOtherEffectsComponent,
//! and other advanced features for matching gameplay effects.

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use bevy_gameplay_ability_system::{
    GasPlugin,
    core::components::OwnedTags,
    effects::{ActiveGameplayEffect, EffectTarget, GameplayEffectQuery},
};
use bevy_gameplay_tag::{
    GameplayTag, GameplayTagContainer, GameplayTagsManager, GameplayTagsPlugin,
};

/// Helper to create a test app with all necessary plugins.
fn setup_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(GameplayTagsPlugin::with_data_path(
            "assets/gameplay_tags.json".to_string(),
        ))
        .add_plugins(GasPlugin);
    app.update(); // Initialize tags
    app
}

/// Helper to spawn a test entity with tags.
fn spawn_entity_with_tags(world: &mut World, tag_names: Vec<&str>) -> Entity {
    let entity = world
        .spawn((OwnedTags::default(), Name::new("TestEntity")))
        .id();

    // Add tags using a system to get proper access to Commands
    let tag_names_owned: Vec<String> = tag_names.iter().map(|s| s.to_string()).collect();
    world.run_system_once(
        move |mut query: Query<&mut OwnedTags>,
              tags_manager: Res<GameplayTagsManager>,
              mut commands: Commands| {
            if let Ok(mut tags) = query.get_mut(entity) {
                for tag_name in &tag_names_owned {
                    let mut container = GameplayTagContainer::default();
                    container.add_tag(GameplayTag::new(tag_name), &tags_manager);
                    tags.0.update_tag_container_count(
                        &container,
                        1,
                        &tags_manager,
                        &mut commands,
                        entity,
                    );
                }
            }
        },
    );

    entity
}

/// Helper to spawn an active effect with specific properties.
fn spawn_active_effect(
    world: &mut World,
    definition_id: &str,
    source: Entity,
    target: Entity,
    granted_tag_names: Vec<&str>,
) -> Entity {
    let manager = world.resource::<GameplayTagsManager>();
    let mut granted_tags = GameplayTagContainer::default();
    for tag_name in granted_tag_names {
        let tag = GameplayTag::new(tag_name);
        granted_tags.add_tag(tag, manager);
    }

    world
        .spawn((
            ActiveGameplayEffect {
                definition_id: definition_id.into(),
                source,
                target,
                level: 1,
                start_time: 0.0,
                granted_tags,
                stack_count: 1,
            },
            EffectTarget(target),
        ))
        .id()
}

#[test]
fn test_query_by_definition_id() {
    let mut app = setup_test_app();
    let world = app.world_mut();

    let source = world.spawn_empty().id();
    let target = world.spawn_empty().id();

    let poison_effect = spawn_active_effect(world, "poison_dot", source, target, vec![]);
    let fire_effect = spawn_active_effect(world, "fire_dot", source, target, vec![]);

    // Query for poison effects
    let query = GameplayEffectQuery::new().with_definition_id("poison_dot");
    let matching = query.find_matching_effects(target, world);

    assert_eq!(matching.len(), 1);
    assert!(matching.contains(&poison_effect));
    assert!(!matching.contains(&fire_effect));
}

#[test]
fn test_query_by_owning_tags_any() {
    let mut app = setup_test_app();
    let world = app.world_mut();

    let source = world.spawn_empty().id();
    let target = world.spawn_empty().id();

    let poison_effect = spawn_active_effect(
        world,
        "poison",
        source,
        target,
        vec!["Effect.Debuff.Poison"],
    );
    let burn_effect =
        spawn_active_effect(world, "burn", source, target, vec!["Effect.Debuff.Burn"]);
    let heal_effect = spawn_active_effect(world, "heal", source, target, vec!["Effect.Buff.Heal"]);

    // Query for any debuff
    let manager = world.resource::<GameplayTagsManager>();
    let query = GameplayEffectQuery::new()
        .with_owning_tags_any(vec!["Effect.Debuff.Poison", "Effect.Debuff.Burn"], manager);
    let matching = query.find_matching_effects(target, world);

    assert_eq!(matching.len(), 2);
    assert!(matching.contains(&poison_effect));
    assert!(matching.contains(&burn_effect));
    assert!(!matching.contains(&heal_effect));
}

#[test]
fn test_query_by_owning_tags_all() {
    let mut app = setup_test_app();
    let world = app.world_mut();

    let source = world.spawn_empty().id();
    let target = world.spawn_empty().id();

    // Effect with multiple tags
    let manager = world.resource::<GameplayTagsManager>();
    let mut multi_tags = GameplayTagContainer::default();
    multi_tags.add_tag(GameplayTag::new("Effect.Debuff.Poison"), manager);
    multi_tags.add_tag(GameplayTag::new("Effect.Debuff.Burn"), manager);

    let multi_effect = world
        .spawn((
            ActiveGameplayEffect {
                definition_id: "toxic_burn".into(),
                source,
                target,
                level: 1,
                start_time: 0.0,
                granted_tags: multi_tags,
                stack_count: 1,
            },
            EffectTarget(target),
        ))
        .id();

    let poison_only = spawn_active_effect(
        world,
        "poison",
        source,
        target,
        vec!["Effect.Debuff.Poison"],
    );

    // Query for effects with BOTH tags
    let manager = world.resource::<GameplayTagsManager>();
    let query = GameplayEffectQuery::new()
        .with_owning_tags_all(vec!["Effect.Debuff.Poison", "Effect.Debuff.Burn"], manager);
    let matching = query.find_matching_effects(target, world);

    assert_eq!(matching.len(), 1);
    assert!(matching.contains(&multi_effect));
    assert!(!matching.contains(&poison_only));
}

#[test]
fn test_query_by_owning_tags_none() {
    let mut app = setup_test_app();
    let world = app.world_mut();

    let source = world.spawn_empty().id();
    let target = world.spawn_empty().id();

    let poison_effect = spawn_active_effect(
        world,
        "poison",
        source,
        target,
        vec!["Effect.Debuff.Poison"],
    );
    let heal_effect = spawn_active_effect(world, "heal", source, target, vec!["Effect.Buff.Heal"]);

    // Query for effects that are NOT debuffs
    let manager = world.resource::<GameplayTagsManager>();
    let query = GameplayEffectQuery::new()
        .with_owning_tags_none(vec!["Effect.Debuff.Poison", "Effect.Debuff.Burn"], manager);
    let matching = query.find_matching_effects(target, world);

    assert_eq!(matching.len(), 1);
    assert!(!matching.contains(&poison_effect));
    assert!(matching.contains(&heal_effect));
}

#[test]
fn test_query_by_source_tags_any() {
    let mut app = setup_test_app();
    let world = app.world_mut();

    let enemy_source = spawn_entity_with_tags(world, vec!["Actor.Enemy"]);
    let player_source = spawn_entity_with_tags(world, vec!["Actor.Player"]);
    let target = world.spawn_empty().id();

    let enemy_effect = spawn_active_effect(world, "damage", enemy_source, target, vec![]);
    let player_effect = spawn_active_effect(world, "heal", player_source, target, vec![]);

    // Query for effects from enemies
    let manager = world.resource::<GameplayTagsManager>();
    let query = GameplayEffectQuery::new().with_source_tags_any(vec!["Actor.Enemy"], manager);
    let matching = query.find_matching_effects(target, world);

    assert_eq!(matching.len(), 1);
    assert!(matching.contains(&enemy_effect));
    assert!(!matching.contains(&player_effect));
}

#[test]
fn test_query_by_source_tags_all() {
    let mut app = setup_test_app();
    let world = app.world_mut();

    let elite_enemy = spawn_entity_with_tags(world, vec!["Actor.Enemy", "Actor.Elite"]);
    let normal_enemy = spawn_entity_with_tags(world, vec!["Actor.Enemy"]);
    let target = world.spawn_empty().id();

    let elite_effect = spawn_active_effect(world, "damage", elite_enemy, target, vec![]);
    let normal_effect = spawn_active_effect(world, "damage", normal_enemy, target, vec![]);

    // Query for effects from elite enemies
    let manager = world.resource::<GameplayTagsManager>();
    let query = GameplayEffectQuery::new()
        .with_source_tags_all(vec!["Actor.Enemy", "Actor.Elite"], manager);
    let matching = query.find_matching_effects(target, world);

    assert_eq!(matching.len(), 1);
    assert!(matching.contains(&elite_effect));
    assert!(!matching.contains(&normal_effect));
}

#[test]
fn test_query_by_source_tags_none() {
    let mut app = setup_test_app();
    let world = app.world_mut();

    let enemy_source = spawn_entity_with_tags(world, vec!["Actor.Enemy"]);
    let neutral_source = spawn_entity_with_tags(world, vec!["Actor.Neutral"]);
    let target = world.spawn_empty().id();

    let enemy_effect = spawn_active_effect(world, "damage", enemy_source, target, vec![]);
    let neutral_effect = spawn_active_effect(world, "damage", neutral_source, target, vec![]);

    // Query for effects NOT from enemies
    let manager = world.resource::<GameplayTagsManager>();
    let query = GameplayEffectQuery::new().with_source_tags_none(vec!["Actor.Enemy"], manager);
    let matching = query.find_matching_effects(target, world);

    assert_eq!(matching.len(), 1);
    assert!(!matching.contains(&enemy_effect));
    assert!(matching.contains(&neutral_effect));
}

#[test]
fn test_query_with_custom_match() {
    let mut app = setup_test_app();
    let world = app.world_mut();

    let source = world.spawn_empty().id();
    let target = world.spawn_empty().id();

    let low_level_effect = world
        .spawn((
            ActiveGameplayEffect {
                definition_id: "damage".into(),
                source,
                target,
                level: 1,
                start_time: 0.0,
                granted_tags: GameplayTagContainer::default(),
                stack_count: 1,
            },
            EffectTarget(target),
        ))
        .id();

    let high_level_effect = world
        .spawn((
            ActiveGameplayEffect {
                definition_id: "damage".into(),
                source,
                target,
                level: 10,
                start_time: 0.0,
                granted_tags: GameplayTagContainer::default(),
                stack_count: 1,
            },
            EffectTarget(target),
        ))
        .id();

    // Query for high-level effects (level >= 5)
    let query = GameplayEffectQuery::new().with_custom_match(|effect_entity, world| {
        if let Some(active_effect) = world.get::<ActiveGameplayEffect>(effect_entity) {
            active_effect.level >= 5
        } else {
            false
        }
    });
    let matching = query.find_matching_effects(target, world);

    assert_eq!(matching.len(), 1);
    assert!(!matching.contains(&low_level_effect));
    assert!(matching.contains(&high_level_effect));
}

#[test]
fn test_query_combined_criteria() {
    let mut app = setup_test_app();
    let world = app.world_mut();

    let enemy_source = spawn_entity_with_tags(world, vec!["Actor.Enemy"]);
    let player_source = spawn_entity_with_tags(world, vec!["Actor.Player"]);
    let target = world.spawn_empty().id();

    // Enemy poison effect
    let enemy_poison = spawn_active_effect(
        world,
        "poison_dot",
        enemy_source,
        target,
        vec!["Effect.Debuff.Poison"],
    );

    // Player poison effect (shouldn't match - wrong source)
    let _player_poison = spawn_active_effect(
        world,
        "poison_dot",
        player_source,
        target,
        vec!["Effect.Debuff.Poison"],
    );

    // Enemy fire effect (shouldn't match - wrong tags)
    let _enemy_fire = spawn_active_effect(
        world,
        "fire_dot",
        enemy_source,
        target,
        vec!["Effect.Debuff.Burn"],
    );

    // Query for poison effects from enemies
    let manager = world.resource::<GameplayTagsManager>();
    let query = GameplayEffectQuery::new()
        .with_owning_tags_any(vec!["Effect.Debuff.Poison"], manager)
        .with_source_tags_any(vec!["Actor.Enemy"], manager);
    let matching = query.find_matching_effects(target, world);

    assert_eq!(matching.len(), 1);
    assert!(matching.contains(&enemy_poison));
}

#[test]
fn test_query_matches_empty_query() {
    let mut app = setup_test_app();
    let world = app.world_mut();

    let source = world.spawn_empty().id();
    let target = world.spawn_empty().id();

    let effect1 = spawn_active_effect(world, "effect1", source, target, vec![]);
    let effect2 = spawn_active_effect(world, "effect2", source, target, vec![]);

    // Empty query should match all effects
    let query = GameplayEffectQuery::new();
    let matching = query.find_matching_effects(target, world);

    assert_eq!(matching.len(), 2);
    assert!(matching.contains(&effect1));
    assert!(matching.contains(&effect2));
}

#[test]
fn test_query_no_matches() {
    let mut app = setup_test_app();
    let world = app.world_mut();

    let source = world.spawn_empty().id();
    let target = world.spawn_empty().id();

    let _effect = spawn_active_effect(
        world,
        "poison",
        source,
        target,
        vec!["Effect.Debuff.Poison"],
    );

    // Query for fire effects (none exist)
    let manager = world.resource::<GameplayTagsManager>();
    let query =
        GameplayEffectQuery::new().with_owning_tags_any(vec!["Effect.Debuff.Burn"], manager);
    let matching = query.find_matching_effects(target, world);

    assert_eq!(matching.len(), 0);
}
