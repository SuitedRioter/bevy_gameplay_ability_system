//! Gameplay Ability Target Data
//!
//! Represents targeting information for abilities, including actors, locations, and hit results.
//! This is the ECS equivalent of UE's `FGameplayAbilityTargetData` and `FGameplayAbilityTargetDataHandle`.

use bevy::prelude::*;

/// Targeting information for an ability activation.
///
/// This is a value object that can be passed through events and stored on ability instances.
/// It represents "what/where" the ability is targeting.
#[derive(Debug, Clone, Component)]
pub struct GameplayAbilityTargetData {
    /// Primary target actors
    pub actors: Vec<Entity>,
    /// Origin transform (where the ability is cast from)
    pub origin: Option<Transform>,
    /// End point transform (where the ability is aimed at)
    pub end_point: Option<Transform>,
}

impl Default for GameplayAbilityTargetData {
    fn default() -> Self {
        Self::empty()
    }
}

impl GameplayAbilityTargetData {
    /// Create empty target data
    pub fn empty() -> Self {
        Self {
            actors: Vec::new(),
            origin: None,
            end_point: None,
        }
    }

    /// Create target data from a single actor
    pub fn from_actor(actor: Entity) -> Self {
        Self {
            actors: vec![actor],
            origin: None,
            end_point: None,
        }
    }

    /// Create target data from multiple actors
    pub fn from_actors(actors: Vec<Entity>) -> Self {
        Self {
            actors,
            origin: None,
            end_point: None,
        }
    }

    /// Create target data from a location
    pub fn from_location(location: Vec3) -> Self {
        Self {
            actors: Vec::new(),
            origin: None,
            end_point: Some(Transform::from_translation(location)),
        }
    }

    /// Create target data from a transform
    pub fn from_transform(transform: Transform) -> Self {
        Self {
            actors: Vec::new(),
            origin: None,
            end_point: Some(transform),
        }
    }

    /// Set the origin transform
    pub fn with_origin(mut self, origin: Transform) -> Self {
        self.origin = Some(origin);
        self
    }

    /// Set the end point transform
    pub fn with_end_point(mut self, end_point: Transform) -> Self {
        self.end_point = Some(end_point);
        self
    }

    /// Add an actor to the target list
    pub fn add_actor(&mut self, actor: Entity) {
        self.actors.push(actor);
    }

    /// Check if this target data has any actors
    pub fn has_actors(&self) -> bool {
        !self.actors.is_empty()
    }

    /// Check if this target data has an origin
    pub fn has_origin(&self) -> bool {
        self.origin.is_some()
    }

    /// Check if this target data has an end point
    pub fn has_end_point(&self) -> bool {
        self.end_point.is_some()
    }

    /// Get the first actor, if any
    pub fn first_actor(&self) -> Option<Entity> {
        self.actors.first().copied()
    }

    /// Get the end point location, if any
    pub fn end_point_location(&self) -> Option<Vec3> {
        self.end_point.map(|t| t.translation)
    }

    /// Get the primary target entity (first actor in the list)
    pub fn primary_target(&self) -> Option<Entity> {
        self.first_actor()
    }

    /// Get all target entities
    pub fn all_targets(&self) -> &[Entity] {
        &self.actors
    }
}
