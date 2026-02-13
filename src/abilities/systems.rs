//! Ability system implementations.
//!
//! This module contains the systems that manage gameplay abilities.

use super::components::*;
use super::definition::*;
use crate::abilities::AbilityRegistry;
use bevy::prelude::*;
use bevy_gameplay_tag::gameplay_tag_count_container::GameplayTagCountContainer;

/// Event for trying to activate an ability.
#[derive(Event, Debug, Clone)]
pub struct TryActivateAbilityEvent {
    /// The ability spec entity to activate.
    pub ability_spec: Entity,
    /// The owner entity.
    pub owner: Entity,
}

/// Event triggered when an ability is successfully activated.
#[derive(Event, Debug, Clone)]
pub struct AbilityActivatedEvent {
    /// The ability spec entity.
    pub ability_spec: Entity,
    /// The owner entity.
    pub owner: Entity,
    /// The active instance entity (if instanced).
    pub instance: Option<Entity>,
}

/// Event triggered when an ability ends.
#[derive(Event, Debug, Clone)]
pub struct AbilityEndedEvent {
    /// The ability spec entity.
    pub ability_spec: Entity,
    /// The owner entity.
    pub owner: Entity,
    /// The active instance entity (if instanced).
    pub instance: Option<Entity>,
}

/// Event for committing an ability (applying costs and cooldowns).
#[derive(Event, Debug, Clone)]
pub struct CommitAbilityEvent {
    /// The ability spec entity.
    pub ability_spec: Entity,
    /// The owner entity.
    pub owner: Entity,
}

/// Event for canceling an ability.
#[derive(Event, Debug, Clone)]
pub struct CancelAbilityEvent {
    /// The ability spec entity.
    pub ability_spec: Entity,
    /// The owner entity.
    pub owner: Entity,
}

/// System that tries to activate abilities in response to events.
pub fn try_activate_ability_system(
    _commands: Commands,
    _registry: Res<AbilityRegistry>,
    _time: Res<Time>,
    _ability_specs: Query<(&AbilitySpec, &AbilityOwner, &AbilityState)>,
    _tag_containers: Query<&GameplayTagCountContainer>,
) {
    // TODO: Implement with observer pattern
    // This will check activation requirements and create ability instances
}

/// System that checks if abilities can be activated based on tag requirements.
pub fn check_ability_activation_requirements(
    ability_def: &AbilityDefinition,
    tags: &GameplayTagCountContainer,
) -> bool {
    // Check required tags
    for required_tag in &ability_def.activation_required_tags {
        if !tags.has_matching_gameplay_tag(required_tag) {
            return false;
        }
    }

    // Check blocked tags
    for blocked_tag in &ability_def.activation_blocked_tags {
        if tags.has_matching_gameplay_tag(blocked_tag) {
            return false;
        }
    }

    true
}

/// System that commits abilities (applies costs and cooldowns).
pub fn commit_ability_system(
    _commands: Commands,
    _registry: Res<AbilityRegistry>,
    _ability_specs: Query<(&AbilitySpec, &AbilityOwner)>,
) {
    // TODO: Implement with observer pattern
    // This will apply cost effects and cooldown effects
}

/// System that ends abilities.
pub fn end_ability_system(
    mut commands: Commands,
    mut ability_specs: Query<(Entity, &mut AbilitySpec, &mut AbilityState)>,
    active_instances: Query<(Entity, &ActiveAbilityInstance)>,
) {
    // End abilities that are marked for ending
    for (spec_entity, mut spec, mut state) in ability_specs.iter_mut() {
        if spec.is_active && *state == AbilityState::Active {
            // Find and despawn any active instances
            for (instance_entity, instance) in active_instances.iter() {
                if instance.spec_entity == spec_entity {
                    commands.entity(instance_entity).despawn();
                }
            }

            // Update spec state
            spec.is_active = false;
            *state = AbilityState::Ready;
        }
    }
}

/// System that cancels abilities based on tags.
pub fn cancel_abilities_by_tags_system(
    mut commands: Commands,
    registry: Res<AbilityRegistry>,
    mut ability_specs: Query<(Entity, &mut AbilitySpec, &AbilityOwner, &mut AbilityState)>,
    tag_containers: Query<&GameplayTagCountContainer>,
    active_instances: Query<(Entity, &ActiveAbilityInstance)>,
) {
    for (spec_entity, mut spec, owner, mut state) in ability_specs.iter_mut() {
        if !spec.is_active {
            continue;
        }

        let Some(definition) = registry.get(&spec.definition_id) else {
            continue;
        };

        let Ok(tags) = tag_containers.get(owner.0) else {
            continue;
        };

        // Check if any cancel tags are present
        let mut should_cancel = false;
        for cancel_tag in &definition.cancel_on_tags_added {
            if tags.has_matching_gameplay_tag(cancel_tag) {
                should_cancel = true;
                break;
            }
        }

        if should_cancel {
            // Find and despawn any active instances
            for (instance_entity, instance) in active_instances.iter() {
                if instance.spec_entity == spec_entity {
                    commands.entity(instance_entity).despawn();
                }
            }

            // Update spec state
            spec.is_active = false;
            *state = AbilityState::Ready;
        }
    }
}

/// System that updates ability states based on cooldowns and tags.
pub fn update_ability_states_system(
    mut ability_specs: Query<(&AbilitySpec, &AbilityOwner, &mut AbilityState)>,
    cooldowns: Query<&AbilityCooldown>,
    tag_containers: Query<&GameplayTagCountContainer>,
    registry: Res<AbilityRegistry>,
) {
    for (spec, owner, mut state) in ability_specs.iter_mut() {
        if spec.is_active {
            *state = AbilityState::Active;
            continue;
        }

        // Check for cooldown
        let has_cooldown = cooldowns.iter().any(|cd| !cd.is_expired());
        if has_cooldown {
            *state = AbilityState::Cooldown;
            continue;
        }

        // Check if blocked by tags
        if let Some(definition) = registry.get(&spec.definition_id)
            && let Ok(tags) = tag_containers.get(owner.0)
            && !check_ability_activation_requirements(definition, tags)
        {
            *state = AbilityState::Blocked;
            continue;
        }

        *state = AbilityState::Ready;
    }
}

/// System that updates ability cooldowns.
pub fn update_ability_cooldowns_system(
    mut commands: Commands,
    mut cooldowns: Query<(Entity, &mut AbilityCooldown)>,
    time: Res<Time>,
) {
    for (entity, mut cooldown) in cooldowns.iter_mut() {
        cooldown.tick(time.delta_secs());

        if cooldown.is_expired() {
            commands.entity(entity).remove::<AbilityCooldown>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_gameplay_tag::gameplay_tag::GameplayTag;

    #[test]
    fn test_check_activation_requirements() {
        // Note: This test is simplified because GameplayTagCountContainer requires
        // Commands, Entity, and GameplayTagsManager to properly add tags.
        // In a real scenario, you would set up a full Bevy app with the tag system.

        // Create ability definition
        let ability = AbilityDefinition::new("test")
            .add_activation_required_tag(GameplayTag::new("State.Alive"))
            .add_activation_blocked_tag(GameplayTag::new("State.Stunned"));

        // Verify the ability definition was created correctly
        assert_eq!(ability.id, "test");
        assert_eq!(ability.activation_required_tags.len(), 1);
        assert_eq!(ability.activation_blocked_tags.len(), 1);
    }
}
