//! Example demonstrating ability activation flow.
//!
//! Shows: TryActivate → Commit (costs/cooldowns) → End

use bevy::prelude::*;
use bevy_gameplay_ability_system::prelude::*;
use bevy_gameplay_tag::GameplayTagsPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GameplayTagsPlugin::with_data_path(
            "assets/gameplay_tags.json".to_string(),
        ))
        .add_plugins(GasPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, activate_ability)
        .run();
}

fn setup(mut commands: Commands, mut ability_registry: ResMut<AbilityRegistry>) {
    // Register ability definition
    ability_registry
        .register(AbilityDefinition::new("Fireball").with_cooldown_effect("Fireball.Cooldown"));

    // Create player with granted ability
    let player = commands.spawn_empty().id();
    let ability = commands
        .spawn((AbilitySpec::new("Fireball", 1), AbilityOwner(player)))
        .id();

    info!(
        "Player created with Fireball ability (entity: {:?})",
        ability
    );
    info!("Press SPACE to activate, E to end");
}

fn activate_ability(
    mut commands: Commands,
    abilities: Query<(Entity, &AbilitySpec, &AbilityOwner)>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        for (ability_entity, spec, owner) in &abilities {
            if !spec.is_active {
                info!("→ TryActivate: {}", spec.definition_id);
                commands.trigger(TryActivateAbilityEvent {
                    ability_spec: ability_entity,
                    owner: owner.0,
                });
            }
        }
    }

    if keyboard.just_pressed(KeyCode::KeyE) {
        for (ability_entity, spec, owner) in &abilities {
            if spec.is_active {
                info!("→ EndAbility: {}", spec.definition_id);
                commands.trigger(EndAbilityEvent {
                    ability_spec: ability_entity,
                    owner: owner.0,
                });
            }
        }
    }
}
