//! Effect system plugin.
//!
//! This plugin registers all effect-related systems and events.

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
            // Register observer for effect application
            .add_observer(on_apply_gameplay_effect)
            // Register kept systems with proper system sets
            .add_systems(
                Update,
                create_effect_modifiers_system
                    .in_set(GasSystemSet::Effects)
                    .in_set(EffectSystemSet::CreateModifiers),
            )
            .add_systems(
                Update,
                update_effect_durations_system
                    .in_set(GasSystemSet::Effects)
                    .in_set(EffectSystemSet::UpdateDurations),
            )
            .add_systems(
                Update,
                execute_periodic_effects_system
                    .in_set(GasSystemSet::Effects)
                    .in_set(EffectSystemSet::ExecutePeriodic),
            )
            .add_systems(
                Update,
                remove_expired_effects_system
                    .in_set(GasSystemSet::Effects)
                    .in_set(EffectSystemSet::RemoveExpired),
            )
            .add_systems(
                Update,
                remove_instant_effects_system
                    .in_set(GasSystemSet::Effects)
                    .in_set(EffectSystemSet::RemoveInstant),
            )
            .add_systems(PostUpdate, aggregate_attribute_modifiers_system);
    }
}
