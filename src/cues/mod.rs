//! GameplayCue module.
//!
//! This module provides the gameplay cue system, which handles visual and audio
//! feedback for gameplay events.

pub mod manager;
pub mod notify;
pub mod plugin;
pub mod systems;

pub use manager::*;
pub use notify::*;
pub use plugin::CuePlugin;
pub use systems::*;
