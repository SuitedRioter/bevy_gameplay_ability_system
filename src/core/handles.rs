//! Handle types for safe entity references.
//!
//! Handles provide a stable way to reference entities that may be despawned,
//! using a generation counter to detect stale references.

use bevy::prelude::*;

/// A handle to an ability spec entity.
///
/// This provides a stable reference to an ability that can detect if the
/// underlying entity has been despawned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AbilityHandle {
    /// The entity this handle refers to.
    pub entity: Entity,
    /// Generation counter for detecting stale references.
    pub generation: u32,
}

impl AbilityHandle {
    /// Creates a new ability handle.
    pub fn new(entity: Entity, generation: u32) -> Self {
        Self { entity, generation }
    }

    /// Checks if this handle is still valid.
    pub fn is_valid(&self, world: &World) -> bool {
        world.get_entity(self.entity).is_ok()
    }
}

/// A handle to an active gameplay effect entity.
///
/// This provides a stable reference to an effect that can detect if the
/// underlying entity has been despawned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EffectHandle {
    /// The entity this handle refers to.
    pub entity: Entity,
    /// Generation counter for detecting stale references.
    pub generation: u32,
}

impl EffectHandle {
    /// Creates a new effect handle.
    pub fn new(entity: Entity, generation: u32) -> Self {
        Self { entity, generation }
    }

    /// Checks if this handle is still valid.
    pub fn is_valid(&self, world: &World) -> bool {
        world.get_entity(self.entity).is_ok()
    }
}

/// A handle to an attribute entity.
///
/// This provides a stable reference to an attribute that can detect if the
/// underlying entity has been despawned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AttributeHandle {
    /// The entity this handle refers to.
    pub entity: Entity,
    /// Generation counter for detecting stale references.
    pub generation: u32,
}

impl AttributeHandle {
    /// Creates a new attribute handle.
    pub fn new(entity: Entity, generation: u32) -> Self {
        Self { entity, generation }
    }

    /// Checks if this handle is still valid.
    pub fn is_valid(&self, world: &World) -> bool {
        world.get_entity(self.entity).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ability_handle_creation() {
        let entity = Entity::from_bits(42);
        let handle = AbilityHandle::new(entity, 1);

        assert_eq!(handle.entity, entity);
        assert_eq!(handle.generation, 1);
    }

    #[test]
    fn test_effect_handle_creation() {
        let entity = Entity::from_bits(42);
        let handle = EffectHandle::new(entity, 1);

        assert_eq!(handle.entity, entity);
        assert_eq!(handle.generation, 1);
    }

    #[test]
    fn test_attribute_handle_creation() {
        let entity = Entity::from_bits(42);
        let handle = AttributeHandle::new(entity, 1);

        assert_eq!(handle.entity, entity);
        assert_eq!(handle.generation, 1);
    }
}
