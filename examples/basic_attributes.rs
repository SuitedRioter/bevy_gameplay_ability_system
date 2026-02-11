//! Basic attributes example.
//!
//! This example demonstrates how to:
//! - Define a custom attribute set
//! - Create entities with attributes
//! - Modify attribute values
//! - Listen to attribute change events

use bevy::prelude::*;
use bevy_gameplay_ability_system::prelude::*;

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(AttributePlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (modify_attributes, print_attribute_changes))
        .run();
}

/// Define a custom attribute set for a character.
struct CharacterAttributes;

impl AttributeSetDefinition for CharacterAttributes {
    fn attribute_names() -> &'static [&'static str] {
        &[
            "Health",
            "MaxHealth",
            "Mana",
            "MaxMana",
            "Strength",
            "Defense",
        ]
    }

    fn attribute_metadata(name: &str) -> Option<AttributeMetadata> {
        match name {
            "Health" => Some(AttributeMetadata {
                name: "Health",
                min_value: Some(0.0),
                max_value: Some(100.0),
            }),
            "MaxHealth" => Some(AttributeMetadata {
                name: "MaxHealth",
                min_value: Some(1.0),
                max_value: None,
            }),
            "Mana" => Some(AttributeMetadata {
                name: "Mana",
                min_value: Some(0.0),
                max_value: Some(100.0),
            }),
            "MaxMana" => Some(AttributeMetadata {
                name: "MaxMana",
                min_value: Some(1.0),
                max_value: None,
            }),
            "Strength" => Some(AttributeMetadata {
                name: "Strength",
                min_value: Some(0.0),
                max_value: None,
            }),
            "Defense" => Some(AttributeMetadata {
                name: "Defense",
                min_value: Some(0.0),
                max_value: None,
            }),
            _ => None,
        }
    }

    fn default_value(name: &str) -> f32 {
        match name {
            "Health" => 100.0,
            "MaxHealth" => 100.0,
            "Mana" => 50.0,
            "MaxMana" => 50.0,
            "Strength" => 10.0,
            "Defense" => 5.0,
            _ => 0.0,
        }
    }

    fn create_attributes(commands: &mut Commands, owner: Entity) -> Vec<Entity> {
        let mut attribute_entities = Vec::new();

        for name in Self::attribute_names() {
            let metadata = Self::attribute_metadata(name).unwrap();
            let default_value = Self::default_value(name);

            let attr_entity = commands
                .spawn((
                    AttributeData {
                        base_value: default_value,
                        current_value: default_value,
                    },
                    AttributeName(name.to_string()),
                    AttributeOwner(owner),
                    AttributeMetadataComponent(metadata),
                ))
                .id();

            attribute_entities.push(attr_entity);
        }

        attribute_entities
    }
}

/// Setup system that creates a character with attributes.
fn setup(mut commands: Commands) {
    info!("=== Basic Attributes Example ===");
    info!("Creating a character with attributes...");

    // Create the character entity
    let character = commands.spawn_empty().id();

    // Create attributes for the character
    let _attribute_entities = CharacterAttributes::create_attributes(&mut commands, character);

    info!("Character created with ID: {:?}", character);
    info!("Attributes: Health, MaxHealth, Mana, MaxMana, Strength, Defense");
}

/// System that modifies attributes over time.
fn modify_attributes(
    time: Res<Time>,
    mut attributes: Query<(&mut AttributeData, &AttributeName, &AttributeOwner)>,
) {
    // Only modify once per second
    if time.elapsed_secs() % 2.0 < time.delta_secs() {
        for (mut attr, name, _owner) in attributes.iter_mut() {
            match name.0.as_str() {
                "Health" => {
                    // Simulate damage
                    attr.current_value = (attr.current_value - 10.0).max(0.0);
                    info!(
                        "Health damaged: {} -> {}",
                        attr.current_value + 10.0,
                        attr.current_value
                    );
                }
                "Mana" => {
                    // Simulate mana consumption
                    attr.current_value = (attr.current_value - 5.0).max(0.0);
                    info!(
                        "Mana consumed: {} -> {}",
                        attr.current_value + 5.0,
                        attr.current_value
                    );
                }
                "Strength" => {
                    // Simulate strength buff
                    attr.base_value += 1.0;
                    attr.current_value = attr.base_value;
                    info!("Strength increased: {}", attr.current_value);
                }
                _ => {}
            }
        }
    }
}

/// System that prints attribute change events.
fn print_attribute_changes(
    mut commands: Commands,
    attributes: Query<
        (Entity, &AttributeData, &AttributeName, &AttributeOwner),
        Changed<AttributeData>,
    >,
) {
    for (attr_entity, attr, name, owner) in attributes.iter() {
        // Trigger attribute changed event
        commands.trigger(AttributeChangedEvent {
            owner: owner.0,
            attribute: attr_entity,
            attribute_name: name.0.clone(),
            old_value: attr.base_value,
            new_value: attr.current_value,
        });
    }
}
