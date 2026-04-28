//! Ability system module.
//!
//! This module provides the gameplay ability system, which allows entities to have
//! abilities that can be activated, committed (with costs and cooldowns), and canceled.

pub mod activation_context;
pub mod activation_info;
pub mod components;
pub mod definition;
pub mod events;
pub mod plugin;
pub mod systems;
pub mod target_data;
pub mod traits;
pub mod trigger_systems;
pub mod triggers;

pub use activation_context::*;
pub use activation_info::*;
pub use components::*;
pub use definition::*;
pub use events::*;
pub use plugin::AbilityPlugin;
pub use systems::*;
pub use target_data::*;
pub use traits::*;
pub use trigger_systems::*;
pub use triggers::*;
