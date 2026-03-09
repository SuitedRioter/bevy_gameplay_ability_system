//! Attribute system module.
//!
//! This module provides a flexible attribute system for game entities.
//! Attributes are stored as separate entities, allowing for efficient
//! querying and modification through the ECS.
//!
//! # Architecture
//!
//! - Each attribute is a separate entity with `AttributeData` component
//! - Attributes are linked to their owner via Bevy's `Parent` component (ChildOf relationship)
//! - Attributes can have metadata defining constraints (min/max values)
//! - Lifecycle hooks allow custom logic in Pre/Post attribute changes
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
//!
//!     // Implement lifecycle hooks
//!     fn pre_attribute_change(context: &mut AttributeModifyContext) {
//!         // Clamp to metadata constraints
//!         context.new_value = context.new_value.max(0.0).min(100.0);
//!     }
//! }
//!
//! fn setup(mut commands: Commands, world: &mut World) {
//!     CharacterAttributes::register_hooks(world);
//!     let player = commands.spawn_empty().id();
//!     CharacterAttributes::create_attributes(&mut commands, player);
//! }
//! ```

pub mod components;
pub mod hooks;
pub mod plugin;
pub mod traits;

pub use components::*;
pub use hooks::*;
pub use plugin::*;
pub use traits::*;
