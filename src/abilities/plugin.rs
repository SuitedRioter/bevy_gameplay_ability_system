//! Ability system plugin.
//!
//! This plugin registers all ability-related systems and events.

use super::definition::AbilityRegistry;
use super::systems::*;
use crate::core::system_sets::{AbilitySystemSet, GasSystemSet};
use crate::effects::definition::GameplayEffectRegistry;
use bevy::prelude::*;

/// Plugin that adds gameplay ability system functionality.
pub struct AbilityPlugin;

impl Plugin for AbilityPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register resources
            .init_resource::<AbilityRegistry>()
            .init_resource::<GameplayEffectRegistry>()
            // Register observers
            .add_observer(on_try_activate_ability)
            .add_observer(on_commit_ability)
            .add_observer(on_end_ability)
            .add_observer(on_cancel_ability)
            // Register kept systems with proper system sets
            .add_systems(
                Update,
                cancel_abilities_by_tags_system
                    .in_set(GasSystemSet::Abilities)
                    .in_set(AbilitySystemSet::Cancel),
            )
            .add_systems(
                Update,
                update_ability_states_system
                    .in_set(GasSystemSet::Abilities)
                    .in_set(AbilitySystemSet::UpdateStates),
            )
            .add_systems(
                Update,
                update_ability_cooldowns_system
                    .in_set(GasSystemSet::Abilities)
                    .in_set(AbilitySystemSet::UpdateCooldowns),
            );
    }
}
