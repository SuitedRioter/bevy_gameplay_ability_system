//! Ability granting system.
//!
//! This module handles granting and removing abilities when effects are applied/removed.

use super::components::*;
use super::definition::*;
use crate::abilities::{AbilityOwner, AbilityRegistry, AbilitySpec};
use bevy::prelude::*;

/// Component that tracks abilities granted by an effect.
#[derive(Component, Debug, Clone)]
pub struct GrantedAbilities {
    /// List of ability entities granted by this effect.
    pub granted_ability_entities: Vec<Entity>,
}

/// System that grants abilities when effects are applied.
///
/// This system runs after effects are applied and grants any abilities
/// specified in the effect definition.
pub fn grant_abilities_from_effects_system(
    mut commands: Commands,
    registry: Res<GameplayEffectRegistry>,
    ability_registry: Res<AbilityRegistry>,
    // Query for newly applied effects that don't have GrantedAbilities yet
    new_effects: Query<
        (Entity, &ActiveGameplayEffect, &EffectTarget),
        (Without<GrantedAbilities>, Added<ActiveGameplayEffect>),
    >,
) {
    for (effect_entity, active_effect, effect_target) in new_effects.iter() {
        let Some(definition) = registry.get(&active_effect.definition_id) else {
            continue;
        };

        if definition.granted_abilities.is_empty() {
            continue;
        }

        let mut granted_entities = Vec::new();

        // Grant each ability to the target
        for granted_config in &definition.granted_abilities {
            let Some(ability_def) = ability_registry.get(&granted_config.ability_id) else {
                warn!(
                    "Ability definition not found: {}",
                    granted_config.ability_id
                );
                continue;
            };

            // Spawn the ability spec
            let ability_entity = commands
                .spawn((
                    AbilitySpec {
                        definition_id: granted_config.ability_id.clone(),
                        level: active_effect.level,
                        input_id: None,
                    },
                    AbilityOwner(effect_target.0),
                    crate::abilities::AbilityActiveState {
                        is_active: false,
                        active_count: 0,
                    },
                    GrantedByEffect {
                        effect_entity,
                        removal_policy: granted_config.removal_policy,
                    },
                ))
                .id();

            granted_entities.push(ability_entity);

            info!(
                "Granted ability '{}' to entity {:?} from effect '{}'",
                granted_config.ability_id, effect_target.0, active_effect.definition_id
            );
        }

        // Add GrantedAbilities component to track what we granted
        commands.entity(effect_entity).insert(GrantedAbilities {
            granted_ability_entities: granted_entities,
        });
    }
}

/// Component that marks an ability as granted by an effect.
#[derive(Component, Debug, Clone, Copy)]
pub struct GrantedByEffect {
    /// The effect entity that granted this ability.
    pub effect_entity: Entity,
    /// How to handle this ability when the effect is removed.
    pub removal_policy: AbilityRemovalPolicy,
}

/// System that removes granted abilities when effects are removed.
///
/// This system handles the removal policy for granted abilities.
pub fn remove_granted_abilities_system(
    mut commands: Commands,
    mut removed_effects: RemovedComponents<ActiveGameplayEffect>,
    granted_abilities_query: Query<&GrantedAbilities>,
    ability_query: Query<(
        &GrantedByEffect,
        &crate::abilities::AbilityActiveState,
        &crate::abilities::AbilityOwner,
    )>,
) {
    for effect_entity in removed_effects.read() {
        // Get the list of granted abilities before the effect is fully despawned
        let Ok(granted_abilities) = granted_abilities_query.get(effect_entity) else {
            continue;
        };

        for &ability_entity in &granted_abilities.granted_ability_entities {
            let Ok((granted_by, active_state, ability_owner)) = ability_query.get(ability_entity)
            else {
                continue;
            };

            match granted_by.removal_policy {
                AbilityRemovalPolicy::CancelAbilityImmediately => {
                    // Cancel the ability if active, then remove it
                    if active_state.is_active {
                        commands.trigger(crate::abilities::systems::CancelAbilityEvent {
                            instance: None, // Cancel all instances
                            ability_spec: ability_entity,
                            owner: ability_owner.0, // Use the actual owner from AbilityOwner component
                        });
                    }
                    commands.entity(ability_entity).despawn();
                    info!(
                        "Removed granted ability {:?} (CancelAbilityImmediately)",
                        ability_entity
                    );
                }
                AbilityRemovalPolicy::RemoveAbilityOnEnd => {
                    // If active, mark for removal when it ends
                    // If not active, remove immediately
                    if active_state.is_active {
                        commands.entity(ability_entity).insert(RemoveAbilityOnEnd);
                        info!(
                            "Marked granted ability {:?} for removal on end",
                            ability_entity
                        );
                    } else {
                        commands.entity(ability_entity).despawn();
                        info!(
                            "Removed granted ability {:?} (RemoveAbilityOnEnd, not active)",
                            ability_entity
                        );
                    }
                }
                AbilityRemovalPolicy::DoNothing => {
                    // Leave the ability, but remove the GrantedByEffect marker
                    commands.entity(ability_entity).remove::<GrantedByEffect>();
                    info!(
                        "Kept granted ability {:?} (DoNothing policy)",
                        ability_entity
                    );
                }
            }
        }
    }
}

/// Marker component for abilities that should be removed when they end.
#[derive(Component, Debug)]
pub struct RemoveAbilityOnEnd;

/// System that removes abilities marked with RemoveAbilityOnEnd when they become inactive.
pub fn cleanup_remove_on_end_abilities_system(
    mut commands: Commands,
    abilities: Query<
        (Entity, &crate::abilities::AbilityActiveState),
        (
            With<RemoveAbilityOnEnd>,
            Changed<crate::abilities::AbilityActiveState>,
        ),
    >,
) {
    for (ability_entity, active_state) in abilities.iter() {
        if !active_state.is_active {
            commands.entity(ability_entity).despawn();
            info!(
                "Removed ability {:?} after it ended (RemoveAbilityOnEnd)",
                ability_entity
            );
        }
    }
}
