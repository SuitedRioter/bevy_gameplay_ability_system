//! Example demonstrating ability granting through gameplay effects.
//!
//! This example shows how effects can grant temporary abilities to targets.

use bevy::prelude::*;
use bevy_gameplay_ability_system::{GasPlugin, abilities::*, attributes::*, effects::*};
use bevy_gameplay_tag::GameplayTagsPlugin;

fn main() {
    App::new()
        .add_plugins((
            MinimalPlugins,
            GameplayTagsPlugin::with_data_path("assets/gameplay_tags.json".to_string()),
            GasPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, check_results)
        .run();
}

// Simple attribute set
struct SimpleAttributeSet;

impl AttributeSetDefinition for SimpleAttributeSet {
    fn attribute_names() -> &'static [&'static str] {
        &["Health"]
    }

    fn attribute_metadata(name: &str) -> Option<AttributeMetadata> {
        match name {
            "Health" => Some(
                AttributeMetadata::new("Health")
                    .with_min(0.0)
                    .with_max(100.0),
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

fn setup(
    mut commands: Commands,
    mut effect_registry: ResMut<GameplayEffectRegistry>,
    mut ability_registry: ResMut<AbilityRegistry>,
) {
    info!("Setting up ability granting example");

    // Create a player entity
    let player = commands.spawn_empty().id();
    SimpleAttributeSet::create_attributes(&mut commands, player);

    // Register a dash ability (will be granted by buff effect)
    let dash_ability = AbilityDefinition::new("dash")
        .with_instancing_policy(InstancingPolicy::InstancedPerExecution);
    ability_registry.register(dash_ability);

    // Register a double jump ability (will be granted by buff effect)
    let double_jump_ability = AbilityDefinition::new("double_jump")
        .with_instancing_policy(InstancingPolicy::InstancedPerExecution);
    ability_registry.register(double_jump_ability);

    // Register a buff effect that grants two abilities
    let buff_effect = GameplayEffectDefinition::new("movement_buff")
        .with_duration_policy(DurationPolicy::HasDuration)
        .with_duration(5.0)
        .grant_ability(
            GrantedAbilityConfig::new("dash")
                .with_removal_policy(AbilityRemovalPolicy::CancelAbilityImmediately),
        )
        .grant_ability_simple("double_jump"); // Uses default removal policy

    effect_registry.register(buff_effect);

    // Apply the buff to the player
    commands.trigger(ApplyGameplayEffectEvent {
        effect_id: "movement_buff".into(),
        target: player,
        instigator: None,
        level: 1,
    });

    info!("Applied movement buff to player");
    info!("Player should now have 'dash' and 'double_jump' abilities");
}

fn check_results(time: Res<Time>, abilities: Query<(&AbilitySpec, &AbilityOwner)>) {
    static mut PRINTED: bool = false;
    if time.elapsed_secs() > 0.2 && unsafe { !PRINTED } {
        unsafe { PRINTED = true };

        let ability_count = abilities.iter().count();
        info!("Player has {} granted abilities", ability_count);

        for (spec, _owner) in abilities.iter() {
            info!("  - Ability: {} (level {})", spec.definition_id, spec.level);
        }

        if ability_count == 2 {
            info!("✓ Ability granting successful!");
        } else {
            warn!("✗ Expected 2 abilities, found {}", ability_count);
        }
    }
}
