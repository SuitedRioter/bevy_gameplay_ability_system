//! Example demonstrating ImmunityGameplayEffectComponent.
//!
//! This example shows how to:
//! 1. Create immunity-granting effects using ImmunityComponent
//! 2. Block incoming effects based on queries
//! 3. Handle immunity events

use bevy::prelude::*;
use bevy_gameplay_ability_system::prelude::*;
use bevy_gameplay_tag::{GameplayTag, GameplayTagsManager, GameplayTagsPlugin};
use std::sync::Arc;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GameplayTagsPlugin::with_data_path(
            "assets/gameplay_tags.json".to_string(),
        ))
        .add_plugins(GasPlugin)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (apply_immunity, apply_poison, check_immunity_events),
        )
        .run();
}

#[derive(Component)]
struct Player;

fn setup(mut commands: Commands) {
    // Spawn a player entity
    commands.spawn((
        Player,
        Name::new("Player"),
        OwnedTags::default(),
        BlockedAbilityTags::default(),
    ));

    info!("Setup complete. Press I to apply immunity, P to apply poison.");
}

fn apply_immunity(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut registry: ResMut<GameplayEffectRegistry>,
    players: Query<Entity, With<Player>>,
    tags_manager: Res<GameplayTagsManager>,
) {
    if !keyboard.just_pressed(KeyCode::KeyI) {
        return;
    }

    // Create immunity effect that blocks all poison effects
    let poison_immunity = GameplayEffectDefinition::new("poison_immunity")
        .with_duration_policy(DurationPolicy::Infinite)
        .add_component(Arc::new(ImmunityComponent::new(vec![
            GameplayEffectQuery::new()
                .with_owning_tags_any(vec!["Effect.Debuff.Poison"], &tags_manager),
        ])));

    registry.register(poison_immunity);

    for player in players.iter() {
        info!("Applying poison immunity to player {:?}", player);

        commands.trigger(ApplyGameplayEffectEvent {
            effect_id: "poison_immunity".into(),
            target: player,
            source: None,
            level: 1,
            set_by_caller_magnitudes: None,
        });
    }
}

fn apply_poison(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut registry: ResMut<GameplayEffectRegistry>,
    players: Query<Entity, With<Player>>,
    tags_manager: Res<GameplayTagsManager>,
) {
    if !keyboard.just_pressed(KeyCode::KeyP) {
        return;
    }

    // Create poison effect
    if !registry.contains("poison_damage") {
        let mut poison = GameplayEffectDefinition::new("poison_damage")
            .with_duration_policy(DurationPolicy::HasDuration)
            .with_duration(5.0)
            .add_modifier(ModifierInfo::new(
                "Health",
                ModifierOperation::Add,
                MagnitudeCalculation::ScalableFloat {
                    base_value: -10.0,
                    level_multiplier: 1.0,
                },
            ));

        // Add poison tag
        poison
            .granted_tags
            .add_tag(GameplayTag::new("Effect.Debuff.Poison"), &tags_manager);

        registry.register(poison);
    }

    for player in players.iter() {
        info!("Attempting to apply poison to player {:?}", player);

        commands.trigger(ApplyGameplayEffectEvent {
            effect_id: "poison_damage".into(),
            target: player,
            source: None,
            level: 1,
            set_by_caller_magnitudes: None,
        });
    }
}

fn check_immunity_events(mut events: EventReader<GameplayEffectBlockedByImmunityEvent>) {
    for event in events.read() {
        info!(
            "Effect '{}' was blocked by immunity on target {:?}!",
            event.effect_id, event.target
        );
    }
}
