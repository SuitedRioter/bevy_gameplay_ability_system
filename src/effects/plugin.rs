//! Effect system plugin.
//!
//! This plugin registers all effect-related systems and events.

use super::ability_granting::{
    cleanup_remove_on_end_abilities_system, grant_abilities_from_effects_system,
    on_gameplay_effect_removed_remove_granted_abilities,
};
use super::application_requirement::ApplicationRequirementRegistry;
use super::custom_calculation::CustomCalculationRegistry;
use super::definition::GameplayEffectRegistry;
use super::systems::*;
use crate::core::system_sets::{EffectSystemSet, GasSystemSet};
use bevy::prelude::*;

/// Plugin that adds gameplay effect system functionality.
pub struct EffectPlugin;

impl Plugin for EffectPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register resources
            .init_resource::<GameplayEffectRegistry>()
            .init_resource::<CustomCalculationRegistry>()
            .init_resource::<ApplicationRequirementRegistry>()
            // Register observer for effect application
            .add_observer(on_apply_gameplay_effect)
            .add_observer(on_gameplay_effect_removed_remove_granted_abilities)
            // Register kept systems with proper system sets
            .add_systems(
                Update,
                create_effect_modifiers_system.in_set(EffectSystemSet::CreateModifiers),
            )
            .add_systems(
                Update,
                update_effect_durations_system.in_set(EffectSystemSet::UpdateDurations),
            )
            .add_systems(
                Update,
                execute_periodic_effects_system.in_set(EffectSystemSet::ExecutePeriodic),
            )
            .add_systems(
                Update,
                remove_expired_effects_system.in_set(EffectSystemSet::RemoveExpired),
            )
            .add_systems(
                Update,
                remove_instant_effects_system.in_set(EffectSystemSet::RemoveInstant),
            )
            .add_systems(
                Update,
                aggregate_attribute_modifiers_system.in_set(EffectSystemSet::Aggregate),
            )
            .add_systems(
                Update,
                grant_abilities_from_effects_system.in_set(EffectSystemSet::CreateModifiers),
            )
            .add_systems(
                Update,
                cleanup_remove_on_end_abilities_system.in_set(GasSystemSet::Cleanup),
            );
    }
}
