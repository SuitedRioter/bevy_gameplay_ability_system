//! Helper functions for granting abilities with proper ChildOf relationships.

use bevy::prelude::*;
use string_cache::DefaultAtom as Atom;

use super::components::{AbilityOwner, AbilitySpec};

/// Helper function to grant an ability to an owner with proper ChildOf relationship.
///
/// This ensures the ability is automatically cleaned up when the owner is despawned.
///
/// # Example
/// ```ignore
/// let player = commands.spawn_empty().id();
/// let ability_spec = grant_ability(&mut commands, player, "fireball", 1);
/// ```
pub fn grant_ability(
    commands: &mut Commands,
    owner: Entity,
    ability_id: impl Into<Atom>,
    level: i32,
) -> Entity {
    commands
        .spawn((
            AbilitySpec::new(ability_id, level),
            AbilityOwner(owner),
            ChildOf(owner), // Automatic cleanup when owner is despawned
        ))
        .id()
}

/// Extension trait for Commands to grant abilities more ergonomically.
pub trait GrantAbilityExt {
    /// Grant an ability to an owner entity.
    ///
    /// # Example
    /// ```ignore
    /// commands.grant_ability(player, "fireball", 1);
    /// ```
    fn grant_ability(&mut self, owner: Entity, ability_id: impl Into<Atom>, level: i32) -> Entity;
}

impl GrantAbilityExt for Commands<'_, '_> {
    fn grant_ability(&mut self, owner: Entity, ability_id: impl Into<Atom>, level: i32) -> Entity {
        grant_ability(self, owner, ability_id, level)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;
    use bevy::ecs::relationship::Relationship;

    #[test]
    fn test_grant_ability_with_childof() {
        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins);

        let owner = app.world_mut().spawn_empty().id();
        let ability_spec = app.world_mut().resource_scope(|world, _: Mut<Time>| {
            grant_ability(&mut world.commands(), owner, "test_ability", 1)
        });

        app.update();

        // Verify ability exists
        assert!(app.world().get_entity(ability_spec).is_ok());

        // Verify ChildOf relationship
        let child_of = app.world().get::<ChildOf>(ability_spec);
        assert!(child_of.is_some());
        assert_eq!(child_of.unwrap().get(), owner);

        // Despawn owner
        app.world_mut().despawn(owner);
        app.update();

        // Verify ability is auto-cleaned
        assert!(app.world().get_entity(ability_spec).is_err());
    }
}
