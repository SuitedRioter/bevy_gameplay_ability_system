//! Gameplay effect system module.
//!
//! This module provides a comprehensive gameplay effect system inspired by
//! Unreal Engine's GameplayAbilitySystem. Effects can modify attributes,
//! grant tags, and have various duration and stacking policies.
//!
//! # Architecture
//!
//! - Each active effect is a separate entity with `ActiveGameplayEffect` component
//! - Effects create modifier entities that modify attributes
//! - Modifiers are aggregated and applied to attributes each frame
//! - Effects can be instant, duration-based, or infinite
//! - Effects can be periodic (execute at regular intervals)
//! - Effects support various stacking policies
//!
//! # Example
//!
//! ```
//! use bevy::prelude::*;
//! use bevy_gameplay_ability_system::effects::*;
//!
//! fn setup(mut registry: ResMut<GameplayEffectRegistry>) {
//!     // Create a healing effect
//!     let heal_effect = GameplayEffectDefinition::new("heal")
//!         .with_duration(5.0)
//!         .with_period(1.0)
//!         .add_modifier(ModifierInfo::new(
//!             "Health",
//!             ModifierOperation::AddCurrent,
//!             MagnitudeCalculation::scalar(10.0),
//!         ));
//!
//!     registry.register(heal_effect);
//! }
//!
//! fn apply_heal(
//!     mut events: EventWriter<ApplyGameplayEffectEvent>,
//!     player: Query<Entity, With<Player>>,
//! ) {
//!     if let Ok(player_entity) = player.get_single() {
//!         events.send(ApplyGameplayEffectEvent {
//!             effect_id: "heal".to_string(),
//!             target: player_entity,
//!             instigator: None,
//!             level: 1,
//!         });
//!     }
//! }
//! ```

pub mod components;
pub mod definition;
pub mod plugin;
pub mod systems;

pub use components::*;
pub use definition::*;
pub use plugin::*;
pub use systems::*;
