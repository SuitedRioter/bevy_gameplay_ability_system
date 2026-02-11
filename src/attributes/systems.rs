//! Attribute system implementation.
//!
//! This module contains the systems that manage attributes and their modifiers.

use super::components::{AttributeData, AttributeMetadataComponent, AttributeName, AttributeOwner};
use bevy::prelude::*;

/// Event triggered when an attribute value changes.
#[derive(Event, Debug, Clone)]
pub struct AttributeChangedEvent {
    /// The entity that owns the attribute.
    pub owner: Entity,
    /// The attribute entity.
    pub attribute: Entity,
    /// The name of the attribute.
    pub attribute_name: String,
    /// The old value.
    pub old_value: f32,
    /// The new value.
    pub new_value: f32,
}

/// System that clamps attribute values to their defined constraints.
///
/// This runs after modifiers are applied to ensure attributes stay within
/// their min/max bounds.
pub fn clamp_attributes_system(
    mut attributes: Query<
        (&mut AttributeData, Option<&AttributeMetadataComponent>),
        Changed<AttributeData>,
    >,
) {
    for (mut attr, metadata) in attributes.iter_mut() {
        if let Some(metadata) = metadata {
            let clamped = metadata.0.clamp(attr.current_value);
            if clamped != attr.current_value {
                attr.current_value = clamped;
            }

            let clamped_base = metadata.0.clamp(attr.base_value);
            if clamped_base != attr.base_value {
                attr.base_value = clamped_base;
            }
        }
    }
}

/// System that triggers attribute change events.
///
/// This detects changes to attribute values and emits events for other systems
/// to react to.
pub fn trigger_attribute_change_events_system(
    mut commands: Commands,
    attributes: Query<
        (Entity, &AttributeData, &AttributeName, &AttributeOwner),
        Changed<AttributeData>,
    >,
) {
    for (entity, attr, name, owner) in attributes.iter() {
        // Note: We can't easily get the old value here without storing it.
        // For now, we'll emit the event with the same value for old and new.
        // A more sophisticated implementation would use a separate component
        // to track previous values.
        commands.trigger(AttributeChangedEvent {
            owner: owner.0,
            attribute: entity,
            attribute_name: name.as_str().to_string(),
            old_value: attr.current_value,
            new_value: attr.current_value,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attributes::components::{AttributeMetadata, AttributeMetadataComponent};

    #[test]
    fn test_clamp_attributes_system() {
        let mut app = App::new();
        app.add_systems(Update, clamp_attributes_system);

        let metadata = AttributeMetadata::new("Health")
            .with_min(0.0)
            .with_max(100.0);

        let entity = app
            .world_mut()
            .spawn((
                AttributeData {
                    base_value: 150.0,
                    current_value: 150.0,
                },
                AttributeMetadataComponent(metadata),
            ))
            .id();

        app.update();

        let attr = app.world().entity(entity).get::<AttributeData>().unwrap();
        assert_eq!(attr.current_value, 100.0);
        assert_eq!(attr.base_value, 100.0);
    }

    // //     #[test]
    // //     fn test_attribute_change_events() {
    // //         let mut app = App::new();
    // //         app.init_resource::<bevy::ecs::event::Events<AttributeChangedEvent>>();
    // //         app.add_systems(Update, trigger_attribute_change_events_system);
    // //
    // //         let owner = app.world_mut().spawn_empty().id();
    // //         let attr_entity = app
    // //             .world_mut()
    // //             .spawn((
    // //                 AttributeData::new(100.0),
    // //                 AttributeName::new("Health"),
    // //                 AttributeOwner(owner),
    // //             ))
    // //             .id();
    // //
    // //         app.update();
    // //
    // //         // Modify the attribute
    // //         app.world_mut()
    // //             .entity_mut(attr_entity)
    // //             .get_mut::<AttributeData>()
    // //             .unwrap()
    // //             .current_value = 80.0;
    // //
    // //         app.update();
    // //
    // //         let mut event_reader = app.world_mut().resource_mut::<bevy::ecs::event::Events<AttributeChangedEvent>>();
    // //         let events: Vec<_> = event_reader.drain().collect();
    // //
    // //         assert_eq!(events.len(), 1);
    // //         assert_eq!(events[0].owner, owner);
    // //         assert_eq!(events[0].attribute, attr_entity);
    // //         assert_eq!(events[0].attribute_name, "Health");
    // //     }
}
