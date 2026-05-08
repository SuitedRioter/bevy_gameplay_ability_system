//! Example demonstrating WhileActive gameplay cues.
//!
//! This example shows how to:
//! 1. Register static cue handlers
//! 2. Trigger WhileActive cues
//! 3. Update cues every frame
//! 4. Remove cues when effects end

use bevy::prelude::*;
use bevy_gameplay_ability_system::core::{BlockedAbilityTags, OwnedTags};
use bevy_gameplay_ability_system::prelude::*;
use bevy_gameplay_tag::{GameplayTag, GameplayTagsPlugin};
use std::sync::Arc;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GameplayTagsPlugin::with_data_path(
            "assets/gameplay_tags.json".to_string(),
        ))
        .add_plugins(GasPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (apply_shield_buff, remove_shield_buff))
        .run();
}

/// Example static cue handler for a shield buff.
struct ShieldBuffCue;

impl GameplayCueNotifyStatic for ShieldBuffCue {
    fn on_execute(&self, target: Entity, params: &GameplayCueParameters, _commands: &mut Commands) {
        info!(
            "Shield buff executed on {:?} with magnitude {}",
            target, params.raw_magnitude
        );
    }

    fn on_active(&self, target: Entity, params: &GameplayCueParameters, _commands: &mut Commands) {
        info!(
            "Shield buff activated on {:?} with magnitude {}",
            target, params.raw_magnitude
        );
    }

    fn while_active(
        &self,
        target: Entity,
        params: &GameplayCueParameters,
        _commands: &mut Commands,
    ) {
        // This is called every frame while the cue is active
        // In a real game, this would update VFX/SFX
        debug!(
            "Shield buff active on {:?} (magnitude: {})",
            target, params.raw_magnitude
        );
    }

    fn on_remove(&self, target: Entity, _params: &GameplayCueParameters, _commands: &mut Commands) {
        info!("Shield buff removed from {:?}", target);
    }
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct ShieldActive {
    start_time: f32,
}

fn setup(
    mut commands: Commands,
    mut static_handlers: ResMut<StaticCueHandlers>,
) {
    // Register the shield buff cue handler
    let shield_tag = GameplayTag::new("GameplayCue.Buff.Shield");
    static_handlers.register(shield_tag, Arc::new(ShieldBuffCue));

    info!("Registered ShieldBuffCue handler");

    // Spawn a player entity
    commands.spawn((
        Player,
        Name::new("Player"),
        OwnedTags::default(),
        BlockedAbilityTags::default(),
    ));

    info!("Setup complete. Press SPACE to apply shield buff, R to remove it.");
}

fn apply_shield_buff(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    players: Query<Entity, (With<Player>, Without<ShieldActive>)>,
) {
    if !keyboard.just_pressed(KeyCode::Space) {
        return;
    }

    for player in players.iter() {
        info!("Applying shield buff to player {:?}", player);

        // Create cue parameters
        let params = GameplayCueParameters::new()
            .with_magnitude(100.0, 1.0)
            .with_target(player);

        // Add ActiveWhileActiveCues component to track the cue
        let mut active_cues = ActiveWhileActiveCues::default();
        active_cues.add(GameplayTag::new("GameplayCue.Buff.Shield"), params);

        commands.entity(player).insert((
            active_cues,
            ShieldActive {
                start_time: time.elapsed_secs(),
            },
        ));

        info!("Shield buff applied! It will update every frame.");
    }
}

fn remove_shield_buff(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    players: Query<(Entity, &ShieldActive), With<Player>>,
) {
    if !keyboard.just_pressed(KeyCode::KeyR) {
        return;
    }

    for (player, shield) in players.iter() {
        let duration = time.elapsed_secs() - shield.start_time;
        info!(
            "Removing shield buff from player {:?} (duration: {:.2}s)",
            player, duration
        );

        // Remove the ActiveWhileActiveCues component
        // This will stop the while_active() calls
        commands
            .entity(player)
            .remove::<ActiveWhileActiveCues>()
            .remove::<ShieldActive>();

        info!("Shield buff removed!");
    }
}
