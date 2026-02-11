//! Effect system plugin.
//!
//! This plugin registers all effect-related systems and events.

use super::definition::GameplayEffectRegistry;
use super::systems::*;
use bevy::prelude::*;

/// Plugin that adds gameplay effect system functionality.
///
/// This plugin registers:
/// - Effect registry resource
/// - Effect application events
/// - Effect systems for applying, updating, and removing effects
/// - Modifier aggregation system
///
/// # Example
/// ```
/// # use bevy::prelude::*;
/// # use bevy_gameplay_ability_system::effects::EffectPlugin;
/// App::new()
///     .add_plugins(EffectPlugin);
/// ```
pub struct EffectPlugin;

impl Plugin for EffectPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register resources
            .init_resource::<GameplayEffectRegistry>()
            // Register systems
            .add_systems(Update, apply_gameplay_effect_system)
            .add_systems(
                Update,
                create_effect_modifiers_system.after(apply_gameplay_effect_system),
            )
            .add_systems(
                Update,
                update_effect_durations_system.after(create_effect_modifiers_system),
            )
            .add_systems(
                Update,
                execute_periodic_effects_system.after(update_effect_durations_system),
            )
            .add_systems(
                Update,
                remove_expired_effects_system.after(execute_periodic_effects_system),
            )
            .add_systems(
                Update,
                remove_instant_effects_system.after(remove_expired_effects_system),
            )
            .add_systems(PostUpdate, aggregate_attribute_modifiers_system);
    }
}
