//! Ability system plugin.
//!
//! This plugin registers all ability-related systems and events.

use super::definition::AbilityRegistry;
use super::systems::*;
use bevy::prelude::*;

/// Plugin that adds gameplay ability system functionality.
///
/// This plugin registers:
/// - Ability registry resource
/// - Ability activation events
/// - Ability systems for activation, commitment, cancellation, and state management
///
/// # Example
/// ```
/// # use bevy::prelude::*;
/// # use bevy_gameplay_ability_system::abilities::AbilityPlugin;
/// App::new()
///     .add_plugins(AbilityPlugin);
/// ```
pub struct AbilityPlugin;

impl Plugin for AbilityPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register resources
            .init_resource::<AbilityRegistry>()
            // Register systems
            .add_systems(
                Update,
                (
                    try_activate_ability_system,
                    commit_ability_system,
                    end_ability_system,
                    cancel_abilities_by_tags_system,
                    update_ability_states_system,
                    update_ability_cooldowns_system,
                ),
            );
    }
}
