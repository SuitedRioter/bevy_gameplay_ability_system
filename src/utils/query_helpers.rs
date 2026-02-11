//! Query helper utilities for common ECS patterns.
//!
//! This module provides helper functions and types for common query patterns
//! used throughout the GAS system.

use crate::abilities::components::{AbilityOwner, AbilitySpec};
use crate::attributes::components::{AttributeData, AttributeName, AttributeOwner};
use crate::effects::components::{ActiveGameplayEffect, EffectTarget};
use bevy::prelude::*;

/// Helper for querying attributes by name for a specific owner.
pub fn find_attribute_by_name(
    owner: Entity,
    attribute_name: &str,
    query: &Query<(Entity, &AttributeData, &AttributeOwner, &AttributeName)>,
) -> Option<(Entity, AttributeData)> {
    query
        .iter()
        .find(|(_, _, attr_owner, name)| attr_owner.0 == owner && name.0 == attribute_name)
        .map(|(entity, data, _, _)| (entity, data.clone()))
}

/// Helper for getting all attributes for a specific owner.
pub fn get_owner_attributes(
    owner: Entity,
    query: &Query<(Entity, &AttributeData, &AttributeOwner, &AttributeName)>,
) -> Vec<(Entity, String, AttributeData)> {
    query
        .iter()
        .filter(|(_, _, attr_owner, _)| attr_owner.0 == owner)
        .map(|(entity, data, _, name)| (entity, name.0.clone(), data.clone()))
        .collect()
}

/// Helper for querying active effects on a target.
pub fn get_active_effects_on_target(
    target: Entity,
    query: &Query<(Entity, &ActiveGameplayEffect, &EffectTarget)>,
) -> Vec<(Entity, ActiveGameplayEffect)> {
    query
        .iter()
        .filter(|(_, _, effect_target)| effect_target.0 == target)
        .map(|(entity, effect, _)| (entity, effect.clone()))
        .collect()
}

/// Helper for finding active effects by definition ID on a target.
pub fn find_effects_by_definition(
    target: Entity,
    definition_id: &str,
    query: &Query<(Entity, &ActiveGameplayEffect, &EffectTarget)>,
) -> Vec<(Entity, ActiveGameplayEffect)> {
    query
        .iter()
        .filter(|(_, effect, effect_target)| {
            effect_target.0 == target && effect.definition_id == definition_id
        })
        .map(|(entity, effect, _)| (entity, effect.clone()))
        .collect()
}

/// Helper for querying abilities owned by an entity.
pub fn get_owned_abilities(
    owner: Entity,
    query: &Query<(Entity, &AbilitySpec, &AbilityOwner)>,
) -> Vec<(Entity, AbilitySpec)> {
    query
        .iter()
        .filter(|(_, _, ability_owner)| ability_owner.0 == owner)
        .map(|(entity, spec, _)| (entity, spec.clone()))
        .collect()
}

/// Helper for finding an ability by definition ID.
pub fn find_ability_by_definition(
    owner: Entity,
    definition_id: &str,
    query: &Query<(Entity, &AbilitySpec, &AbilityOwner)>,
) -> Option<(Entity, AbilitySpec)> {
    query
        .iter()
        .find(|(_, spec, ability_owner)| {
            ability_owner.0 == owner && spec.definition_id == definition_id
        })
        .map(|(entity, spec, _)| (entity, spec.clone()))
}

/// Helper for checking if an entity has a specific attribute.
pub fn has_attribute(
    owner: Entity,
    attribute_name: &str,
    query: &Query<(&AttributeOwner, &AttributeName)>,
) -> bool {
    query
        .iter()
        .any(|(attr_owner, name)| attr_owner.0 == owner && name.0 == attribute_name)
}

/// Helper for checking if an entity has any active effects.
pub fn has_active_effects(target: Entity, query: &Query<&EffectTarget>) -> bool {
    query.iter().any(|effect_target| effect_target.0 == target)
}

/// Helper for checking if an entity has any granted abilities.
pub fn has_abilities(owner: Entity, query: &Query<&AbilityOwner>) -> bool {
    query.iter().any(|ability_owner| ability_owner.0 == owner)
}

/// Helper for counting active effects on a target.
pub fn count_active_effects(target: Entity, query: &Query<&EffectTarget>) -> usize {
    query
        .iter()
        .filter(|effect_target| effect_target.0 == target)
        .count()
}

/// Helper for counting abilities owned by an entity.
pub fn count_abilities(owner: Entity, query: &Query<&AbilityOwner>) -> usize {
    query
        .iter()
        .filter(|ability_owner| ability_owner.0 == owner)
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_attribute() {
        let mut world = World::new();
        let owner = world.spawn_empty().id();

        world.spawn((AttributeOwner(owner), AttributeName("Health".to_string())));

        // Use QueryState for direct world access
        let mut query_state = world.query::<(&AttributeOwner, &AttributeName)>();
        let query_result = query_state
            .iter(&world)
            .any(|(attr_owner, attr_name)| attr_owner.0 == owner && attr_name.0 == "Health");
        assert!(query_result);

        let query_result = query_state
            .iter(&world)
            .any(|(attr_owner, attr_name)| attr_owner.0 == owner && attr_name.0 == "Mana");
        assert!(!query_result);
    }

    #[test]
    fn test_count_active_effects() {
        let mut world = World::new();
        let target = world.spawn_empty().id();

        world.spawn(EffectTarget(target));
        world.spawn(EffectTarget(target));

        let mut query_state = world.query::<&EffectTarget>();
        let count = query_state
            .iter(&world)
            .filter(|effect_target| effect_target.0 == target)
            .count();
        assert_eq!(count, 2);
    }
}
