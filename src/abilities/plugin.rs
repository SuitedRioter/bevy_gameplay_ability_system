//! Ability system plugin.
//!
//! This plugin registers all ability-related systems and events.

use super::definition::AbilityRegistry;
use super::events::GameplayEvent;
use super::systems::*;
use super::trigger_systems::*;
use crate::core::system_sets::GasSystemSet;
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
            .add_observer(on_instance_removed)
            .add_observer(handle_gameplay_event_triggers_system)
            // Activation systems: spawn instances, then call activate.
            .add_systems(
                Update,
                (
                    spawn_pending_ability_instances_system,
                    call_activate_ability_system,
                )
                    .chain()
                    .in_set(GasSystemSet::Abilities),
            )
            // Trigger systems
            .add_systems(
                Update,
                (
                    handle_owned_tag_added_triggers_system,
                    handle_owned_tag_present_triggers_system,
                )
                    .in_set(GasSystemSet::Abilities),
            );
    }
}
