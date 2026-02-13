//! Bevy Gameplay Ability System
//!
//! A comprehensive gameplay ability system for Bevy, inspired by Unreal Engine's
//! GameplayAbilitySystem (GAS). This library provides a flexible and powerful
//! framework for implementing RPG-style abilities, attributes, and effects.
//!
//! # Features
//!
//! - **Attribute System**: Define custom attribute sets with constraints
//! - **Gameplay Effects**: Modify attributes with instant, duration, or infinite effects
//! - **Gameplay Abilities**: Implement abilities with costs, cooldowns, and activation requirements
//! - **Gameplay Cues**: Visual and audio feedback system
//! - **Tag-based System**: Uses `bevy_gameplay_tag` for flexible tag matching
//! - **Pure ECS Architecture**: Fully leverages Bevy's ECS for performance
//!
//! # Quick Start
//!
//! ```rust
//! use bevy::prelude::*;
//! use bevy_gameplay_ability_system::prelude::*;
//!
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     .add_plugins(GasPlugin)
//!     .run();
//! ```
//!
//! # Architecture
//!
//! The system is built on four core modules:
//!
//! 1. **Attributes**: Base values and current values with modifier aggregation
//! 2. **Effects**: Temporary or permanent modifications to attributes
//! 3. **Abilities**: Player-activated actions with costs and cooldowns
//! 4. **Cues**: Visual and audio feedback for gameplay events
//!
//! Each module is designed to work independently but integrates seamlessly
//! with the others.

pub mod abilities;
pub mod attributes;
pub mod core;
pub mod cues;
pub mod effects;
pub mod utils;

/// Prelude module for convenient imports.
pub mod prelude {
    pub use crate::attributes::components::*;
    pub use crate::attributes::plugin::AttributePlugin;
    pub use crate::attributes::systems::AttributeChangedEvent;
    pub use crate::attributes::traits::*;

    pub use crate::effects::components::*;
    pub use crate::effects::definition::*;
    pub use crate::effects::plugin::EffectPlugin;
    pub use crate::effects::systems::{
        ApplyGameplayEffectEvent, GameplayEffectAppliedEvent, GameplayEffectRemovedEvent,
    };

    pub use crate::abilities::components::*;
    pub use crate::abilities::definition::*;
    pub use crate::abilities::plugin::AbilityPlugin;
    pub use crate::abilities::systems::{
        AbilityActivatedEvent, AbilityEndedEvent, CancelAbilityEvent, CommitAbilityEvent,
        TryActivateAbilityEvent,
    };

    pub use crate::cues::manager::*;
    pub use crate::cues::notify::*;
    pub use crate::cues::plugin::CuePlugin;
    pub use crate::cues::systems::TriggerGameplayCueEvent;

    pub use crate::core::events::*;
    pub use crate::core::handles::*;
    pub use crate::core::system_sets::*;

    pub use crate::utils::*;

    pub use crate::GasPlugin;
}

use bevy::prelude::*;

/// Main plugin for the Gameplay Ability System.
///
/// This plugin combines all sub-plugins and provides a single entry point
/// for adding the GAS to your Bevy app.
///
/// # Example
///
/// ```
/// use bevy::prelude::*;
/// use bevy_gameplay_ability_system::GasPlugin;
///
/// App::new()
///     .add_plugins(DefaultPlugins)
///     .add_plugins(GasPlugin)
///     .run();
/// ```
pub struct GasPlugin;

impl Plugin for GasPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(attributes::AttributePlugin)
            .add_plugins(effects::EffectPlugin)
            .add_plugins(abilities::AbilityPlugin)
            .add_plugins(cues::CuePlugin);
    }
}
