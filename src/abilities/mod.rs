//! Ability system module.
//!
//! This module provides the gameplay ability system, which allows entities to have
//! abilities that can be activated, committed (with costs and cooldowns), and canceled.

pub mod components;
pub mod definition;
pub mod events;
pub mod plugin;
pub mod systems;
pub mod traits;
pub mod trigger_systems;
pub mod triggers;

pub use components::*;
pub use definition::*;
pub use events::*;
pub use plugin::AbilityPlugin;
pub use systems::*;
pub use traits::*;
pub use trigger_systems::*;
pub use triggers::*;
