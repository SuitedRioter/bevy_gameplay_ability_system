//! Attribute system module.
//!
//! This module provides a flexible attribute system for game entities.
//! Attributes are stored as separate entities, allowing for efficient
//! querying and modification through the ECS.
//!
//! # Architecture
//!
//! - Each attribute is a separate entity with `AttributeData` component
//! - Attributes are linked to their owner via `AttributeOwner` component
//! - Attributes can have metadata defining constraints (min/max values)
//! - Changes to attributes trigger events for other systems to react to
//!
//! # Example
//!
//! ```
//! use bevy::prelude::*;
//! use bevy_gameplay_ability_system::attributes::*;
//!
//! // Define a custom attribute set
//! struct CharacterAttributes;
//!
//! impl AttributeSetDefinition for CharacterAttributes {
//!     fn attribute_names() -> &'static [&'static str] {
//!         &["Health", "Mana"]
//!     }
//!
//!     fn attribute_metadata(name: &str) -> Option<AttributeMetadata> {
//!         match name {
//!             "Health" => Some(AttributeMetadata::new("Health").with_min(0.0).with_max(100.0)),
//!             "Mana" => Some(AttributeMetadata::new("Mana").with_min(0.0).with_max(100.0)),
//!             _ => None,
//!         }
//!     }
//!
//!     fn default_value(name: &str) -> f32 {
//!         match name {
//!             "Health" => 100.0,
//!             "Mana" => 100.0,
//!             _ => 0.0,
//!         }
//!     }
//! }
//!
//! fn setup(mut commands: Commands) {
//!     // Create an entity with attributes
//!     let player = commands.spawn_empty().id();
//!     CharacterAttributes::create_attributes(&mut commands, player);
//! }
//! ```

pub mod components;
pub mod plugin;
pub mod systems;
pub mod traits;

pub use components::*;
pub use plugin::*;
pub use systems::*;
pub use traits::*;
