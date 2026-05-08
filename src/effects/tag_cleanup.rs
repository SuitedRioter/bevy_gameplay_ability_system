//! Tag cleanup system for expired effects.
//!
//! This system handles removing granted tags when effects expire.
//! It's separated from the main removal system to avoid query conflicts.

use super::components::{ActiveGameplayEffect, EffectGrantedTags, EffectTarget};
use crate::core::OwnedTags;
use bevy::prelude::*;
use bevy_gameplay_tag::GameplayTagsManager;

/// System that removes granted tags from expired effects.
///
/// This runs after `remove_expired_effects_system` to clean up tags
/// without conflicting with the observer's `ParamSet<Query<&mut OwnedTags>>`.
pub fn cleanup_expired_effect_tags_system(
    mut commands: Commands,
    tags_manager: Res<GameplayTagsManager>,
    mut tag_containers: Query<&mut OwnedTags>,
    // Query for effects that are about to be despawned
    mut removed_effects: RemovedComponents<ActiveGameplayEffect>,
    // Cache of granted tags before despawn
    granted_tags_query: Query<(&EffectTarget, &EffectGrantedTags)>,
) {
    for effect_entity in removed_effects.read() {
        // Try to get the granted tags before the entity is fully despawned
        if let Ok((target, granted)) = granted_tags_query.get(effect_entity) {
            if let Ok(mut target_tags) = tag_containers.get_mut(target.0) {
                target_tags.0.update_tag_container_count(
                    &granted.tags,
                    -1,
                    &tags_manager,
                    &mut commands,
                    target.0,
                );
            }
        }
    }
}
