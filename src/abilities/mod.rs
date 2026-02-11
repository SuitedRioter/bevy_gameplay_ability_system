//! Ability system module.
//!
//! This module provides the gameplay ability system, which allows entities to have
//! abilities that can be activated, committed (with costs and cooldowns), and canceled.

pub mod components;
pub mod definition;
pub mod plugin;
pub mod systems;

pub use components::*;
pub use definition::*;
pub use plugin::AbilityPlugin;
pub use systems::*;
