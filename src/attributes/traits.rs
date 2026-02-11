//! Attribute set definition traits.
//!
//! This module provides traits for defining custom attribute sets.

use super::components::{
    AttributeData, AttributeMetadata, AttributeMetadataComponent, AttributeName, AttributeOwner,
};
use bevy::prelude::*;

/// Trait for defining an attribute set.
///
/// Implement this trait to create custom attribute sets for your game.
/// Each attribute set defines a collection of related attributes.
///
/// # Example
/// ```
/// # use bevy::prelude::*;
/// # use bevy_gameplay_ability_system::attributes::{AttributeSetDefinition, AttributeMetadata};
/// struct CharacterAttributes;
///
/// impl AttributeSetDefinition for CharacterAttributes {
///     fn attribute_names() -> &'static [&'static str] {
///         &["Health", "Mana", "Stamina"]
///     }
///
///     fn attribute_metadata(name: &str) -> Option<AttributeMetadata> {
///         match name {
///             "Health" => Some(AttributeMetadata::new("Health").with_min(0.0).with_max(100.0)),
///             "Mana" => Some(AttributeMetadata::new("Mana").with_min(0.0).with_max(100.0)),
///             "Stamina" => Some(AttributeMetadata::new("Stamina").with_min(0.0).with_max(100.0)),
///             _ => None,
///         }
///     }
///
///     fn default_value(name: &str) -> f32 {
///         match name {
///             "Health" => 100.0,
///             "Mana" => 100.0,
///             "Stamina" => 100.0,
///             _ => 0.0,
///         }
///     }
/// }
/// ```
pub trait AttributeSetDefinition: Send + Sync + 'static {
    /// Returns the names of all attributes in this set.
    fn attribute_names() -> &'static [&'static str];

    /// Returns the metadata for a specific attribute.
    fn attribute_metadata(name: &str) -> Option<AttributeMetadata>;

    /// Returns the default value for a specific attribute.
    fn default_value(name: &str) -> f32;

    /// Creates all attributes for this set and attaches them to the owner entity.
    ///
    /// This is a convenience method that creates attribute entities with the
    /// appropriate components.
    fn create_attributes(commands: &mut Commands, owner: Entity) -> Vec<Entity> {
        let mut attribute_entities = Vec::new();

        for &name in Self::attribute_names() {
            let default_value = Self::default_value(name);
            let metadata = Self::attribute_metadata(name);

            let mut entity_commands = commands.spawn((
                AttributeData::new(default_value),
                AttributeName::new(name),
                AttributeOwner(owner),
            ));

            if let Some(metadata) = metadata {
                entity_commands.insert(AttributeMetadataComponent(metadata));
            }

            attribute_entities.push(entity_commands.id());
        }

        attribute_entities
    }
}

/// Helper function to find an attribute entity by name for a given owner.
///
/// This is a utility function for querying attributes.
pub fn find_attribute(
    owner: Entity,
    attribute_name: &str,
    query: &Query<(Entity, &AttributeName, &AttributeOwner)>,
) -> Option<Entity> {
    query
        .iter()
        .find(|(_, name, attr_owner)| attr_owner.0 == owner && name.as_str() == attribute_name)
        .map(|(entity, _, _)| entity)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestAttributes;

    impl AttributeSetDefinition for TestAttributes {
        fn attribute_names() -> &'static [&'static str] {
            &["Health", "Mana"]
        }

        fn attribute_metadata(name: &str) -> Option<AttributeMetadata> {
            match name {
                "Health" => Some(
                    AttributeMetadata::new("Health")
                        .with_min(0.0)
                        .with_max(100.0),
                ),
                "Mana" => Some(AttributeMetadata::new("Mana").with_min(0.0)),
                _ => None,
            }
        }

        fn default_value(name: &str) -> f32 {
            match name {
                "Health" => 100.0,
                "Mana" => 50.0,
                _ => 0.0,
            }
        }
    }

    #[test]
    fn test_attribute_set_definition() {
        let mut app = App::new();
        let mut commands = app.world_mut().commands();

        let owner = commands.spawn_empty().id();
        let attributes = TestAttributes::create_attributes(&mut commands, owner);

        assert_eq!(attributes.len(), 2);
    }

    #[test]
    fn test_attribute_names() {
        assert_eq!(TestAttributes::attribute_names(), &["Health", "Mana"]);
    }

    #[test]
    fn test_default_values() {
        assert_eq!(TestAttributes::default_value("Health"), 100.0);
        assert_eq!(TestAttributes::default_value("Mana"), 50.0);
        assert_eq!(TestAttributes::default_value("Unknown"), 0.0);
    }
}
