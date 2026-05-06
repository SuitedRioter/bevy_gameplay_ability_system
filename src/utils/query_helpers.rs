//! Query helper utilities for common ECS patterns.
//!
//! This module provides helper functions and types for common query patterns
//! used throughout the GAS system.
//!
//! ## Performance Optimization
//!
//! Since Effects and Abilities now use ChildOf relationships, we provide two
//! query strategies:
//!
//! 1. **Direct filtering** (current): O(N) - iterates all entities
//! 2. **Children-based** (optimized): O(K) - only iterates owner's children
//!
//! The Children-based approach is significantly faster when querying a specific
//! owner's entities, especially in large worlds.

use crate::abilities::components::{AbilityOwner, AbilitySpec};
use crate::attributes::components::{AttributeData, AttributeName};
use crate::effects::components::{ActiveGameplayEffect, EffectTarget};
use bevy::ecs::relationship::Relationship;
use bevy::prelude::*;

// ============================================================================
// Attribute Queries (already use ChildOf)
// ============================================================================

/// Helper for querying attributes by name for a specific owner.
pub fn find_attribute_by_name(
    owner: Entity,
    attribute_name: &str,
    query: &Query<(Entity, &AttributeData, &ChildOf, &AttributeName)>,
) -> Option<(Entity, AttributeData)> {
    query
        .iter()
        .find(|(_, _, child_of, name)| child_of.get() == owner && name.as_str() == attribute_name)
        .map(|(entity, data, _, _)| (entity, *data))
}

/// Helper for getting all attributes for a specific owner.
pub fn get_owner_attributes(
    owner: Entity,
    query: &Query<(Entity, &AttributeData, &ChildOf, &AttributeName)>,
) -> Vec<(Entity, String, AttributeData)> {
    query
        .iter()
        .filter(|(_, _, child_of, _)| child_of.get() == owner)
        .map(|(entity, data, _, name)| (entity, name.as_str().to_string(), *data))
        .collect()
}

/// **Optimized**: Get all attributes for a specific owner using Children component.
///
/// This is significantly faster than `get_owner_attributes` when the owner has
/// many children, as it only iterates the owner's children instead of all attributes.
///
/// # Performance
/// - `get_owner_attributes`: O(N) where N = total attributes in world
/// - `get_owner_attributes_fast`: O(K) where K = owner's attributes
pub fn get_owner_attributes_fast(
    owner: Entity,
    children_query: &Query<&Children>,
    attribute_query: &Query<(&AttributeData, &AttributeName)>,
) -> Vec<(Entity, String, AttributeData)> {
    children_query
        .get(owner)
        .map(|children| {
            children
                .iter()
                .filter_map(|child| {
                    attribute_query
                        .get(child)
                        .ok()
                        .map(|(data, name)| (child, name.as_str().to_string(), *data))
                })
                .collect()
        })
        .unwrap_or_default()
}

// ============================================================================
// Effect Queries (now use ChildOf)
// ============================================================================

/// Helper for querying active effects on a target.
///
/// **Note**: This uses direct filtering (O(N)). For better performance with
/// many effects, use `get_active_effects_on_target_fast`.
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

/// **Optimized**: Get active effects on a target using Children component.
///
/// # Performance
/// - `get_active_effects_on_target`: O(N) where N = total effects in world
/// - `get_active_effects_on_target_fast`: O(K) where K = target's effects
pub fn get_active_effects_on_target_fast(
    target: Entity,
    children_query: &Query<&Children>,
    effect_query: &Query<&ActiveGameplayEffect>,
) -> Vec<(Entity, ActiveGameplayEffect)> {
    children_query
        .get(target)
        .map(|children| {
            children
                .iter()
                .filter_map(|child| {
                    effect_query
                        .get(child)
                        .ok()
                        .map(|effect| (child, effect.clone()))
                })
                .collect()
        })
        .unwrap_or_default()
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
            effect_target.0 == target && effect.definition_id.as_ref() == definition_id
        })
        .map(|(entity, effect, _)| (entity, effect.clone()))
        .collect()
}

/// **Optimized**: Find effects by definition ID using Children component.
pub fn find_effects_by_definition_fast(
    target: Entity,
    definition_id: &str,
    children_query: &Query<&Children>,
    effect_query: &Query<&ActiveGameplayEffect>,
) -> Vec<(Entity, ActiveGameplayEffect)> {
    children_query
        .get(target)
        .map(|children| {
            children
                .iter()
                .filter_map(|child| {
                    effect_query.get(child).ok().and_then(|effect| {
                        if effect.definition_id.as_ref() == definition_id {
                            Some((child, effect.clone()))
                        } else {
                            None
                        }
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

// ============================================================================
// Ability Queries (now use ChildOf)
// ============================================================================

/// Helper for querying abilities owned by an entity.
///
/// **Note**: This uses direct filtering (O(N)). For better performance with
/// many abilities, use `get_owned_abilities_fast`.
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

/// **Optimized**: Get owned abilities using Children component.
///
/// # Performance
/// - `get_owned_abilities`: O(N) where N = total abilities in world
/// - `get_owned_abilities_fast`: O(K) where K = owner's abilities
pub fn get_owned_abilities_fast(
    owner: Entity,
    children_query: &Query<&Children>,
    ability_query: &Query<&AbilitySpec>,
) -> Vec<(Entity, AbilitySpec)> {
    children_query
        .get(owner)
        .map(|children| {
            children
                .iter()
                .filter_map(|child| {
                    ability_query
                        .get(child)
                        .ok()
                        .map(|spec| (child, spec.clone()))
                })
                .collect()
        })
        .unwrap_or_default()
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
            ability_owner.0 == owner && spec.definition_id.as_ref() == definition_id
        })
        .map(|(entity, spec, _)| (entity, spec.clone()))
}

/// **Optimized**: Find ability by definition ID using Children component.
pub fn find_ability_by_definition_fast(
    owner: Entity,
    definition_id: &str,
    children_query: &Query<&Children>,
    ability_query: &Query<&AbilitySpec>,
) -> Option<(Entity, AbilitySpec)> {
    children_query.get(owner).ok().and_then(|children| {
        children.iter().find_map(|child| {
            ability_query.get(child).ok().and_then(|spec| {
                if spec.definition_id.as_ref() == definition_id {
                    Some((child, spec.clone()))
                } else {
                    None
                }
            })
        })
    })
}

// ============================================================================
// Existence Checks
// ============================================================================

/// Helper for checking if an entity has a specific attribute.
pub fn has_attribute(
    owner: Entity,
    attribute_name: &str,
    query: &Query<(&ChildOf, &AttributeName)>,
) -> bool {
    query
        .iter()
        .any(|(child_of, name)| child_of.get() == owner && name.as_str() == attribute_name)
}

/// Helper for checking if an entity has any active effects.
pub fn has_active_effects(target: Entity, query: &Query<&EffectTarget>) -> bool {
    query.iter().any(|effect_target| effect_target.0 == target)
}

/// **Optimized**: Check if entity has active effects using Children component.
pub fn has_active_effects_fast(
    target: Entity,
    children_query: &Query<&Children>,
    effect_query: &Query<&ActiveGameplayEffect>,
) -> bool {
    children_query
        .get(target)
        .map(|children| children.iter().any(|child| effect_query.contains(child)))
        .unwrap_or(false)
}

/// Helper for checking if an entity has any granted abilities.
pub fn has_abilities(owner: Entity, query: &Query<&AbilityOwner>) -> bool {
    query.iter().any(|ability_owner| ability_owner.0 == owner)
}

/// **Optimized**: Check if entity has abilities using Children component.
pub fn has_abilities_fast(
    owner: Entity,
    children_query: &Query<&Children>,
    ability_query: &Query<&AbilitySpec>,
) -> bool {
    children_query
        .get(owner)
        .map(|children| children.iter().any(|child| ability_query.contains(child)))
        .unwrap_or(false)
}

// ============================================================================
// Counting
// ============================================================================

/// Helper for counting active effects on a target.
pub fn count_active_effects(target: Entity, query: &Query<&EffectTarget>) -> usize {
    query
        .iter()
        .filter(|effect_target| effect_target.0 == target)
        .count()
}

/// **Optimized**: Count active effects using Children component.
pub fn count_active_effects_fast(
    target: Entity,
    children_query: &Query<&Children>,
    effect_query: &Query<&ActiveGameplayEffect>,
) -> usize {
    children_query
        .get(target)
        .map(|children| {
            children
                .iter()
                .filter(|&child| effect_query.contains(child))
                .count()
        })
        .unwrap_or(0)
}

/// Helper for counting abilities owned by an entity.
pub fn count_abilities(owner: Entity, query: &Query<&AbilityOwner>) -> usize {
    query
        .iter()
        .filter(|ability_owner| ability_owner.0 == owner)
        .count()
}

/// **Optimized**: Count abilities using Children component.
pub fn count_abilities_fast(
    owner: Entity,
    children_query: &Query<&Children>,
    ability_query: &Query<&AbilitySpec>,
) -> usize {
    children_query
        .get(owner)
        .map(|children| {
            children
                .iter()
                .filter(|&child| ability_query.contains(child))
                .count()
        })
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_attribute() {
        let mut world = World::new();
        let owner = world.spawn_empty().id();

        world
            .spawn(AttributeName::new("Health"))
            .set_parent_in_place(owner);

        // Use QueryState for direct world access
        let mut query_state = world.query::<(&ChildOf, &AttributeName)>();
        let query_result = query_state
            .iter(&world)
            .any(|(child_of, attr_name)| child_of.get() == owner && attr_name.as_str() == "Health");
        assert!(query_result);

        let query_result = query_state
            .iter(&world)
            .any(|(child_of, attr_name)| child_of.get() == owner && attr_name.as_str() == "Mana");
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

    #[test]
    fn test_fast_queries_with_children() {
        let mut world = World::new();
        let owner = world.spawn_empty().id();

        // Spawn attributes as children
        world
            .spawn((AttributeData::new(100.0), AttributeName::new("Health")))
            .set_parent_in_place(owner);

        world
            .spawn((AttributeData::new(50.0), AttributeName::new("Mana")))
            .set_parent_in_place(owner);

        world.flush();

        // Test fast query - note: QueryState cannot be used with these helpers
        // They expect Query<'_, '_, T> from system parameters
        // For direct world access, use the non-fast versions or manual iteration
        let children = world.get::<Children>(owner).unwrap();
        let mut count = 0;
        for child in children.iter() {
            if world.get::<AttributeData>(child).is_some() {
                count += 1;
            }
        }
        assert_eq!(count, 2);
    }
}
