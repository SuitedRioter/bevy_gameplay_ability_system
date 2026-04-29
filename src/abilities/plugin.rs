//! Ability system plugin.
//!
//! This plugin registers all ability-related systems and events.

use super::definition::AbilityRegistry;
use super::systems::*;
use super::tasks;
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
            // Task systems
            .add_systems(
                Update,
                (
                    tasks::tick_wait_delay_tasks_system,
                    tasks::cleanup_finished_tasks_system,
                )
                    .chain()
                    .in_set(GasSystemSet::Abilities),
            )
            // Task observers
            .add_observer(tasks::handle_gameplay_event_for_tasks_system)
            // Trigger systems
            .add_systems(
                Update,
                (
                    handle_owned_tag_added_triggers_system,
                    handle_owned_tag_present_triggers_system,
                    tasks::check_wait_attribute_change_tasks_system,
                    tasks::execute_apply_effect_to_target_data_tasks_system,
                )
                    .in_set(GasSystemSet::Abilities),
            )
            .add_observer(tasks::on_effect_applied_for_tasks)
            .add_observer(tasks::on_effect_removed_for_tasks)
            .add_observer(tasks::on_ability_instance_removed);
    }
}
