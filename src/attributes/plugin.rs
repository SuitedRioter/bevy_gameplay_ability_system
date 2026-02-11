//! Attribute system plugin.
//!
//! This plugin registers all attribute-related systems and events.

use super::systems::{clamp_attributes_system, trigger_attribute_change_events_system};
use bevy::prelude::*;

/// Plugin that adds attribute system functionality.
///
/// This plugin registers:
/// - Attribute change events
/// - Attribute clamping system
/// - Attribute change event triggering system
///
/// # Example
/// ```
/// # use bevy::prelude::*;
/// # use bevy_gameplay_ability_system::attributes::AttributePlugin;
/// App::new()
///     .add_plugins(AttributePlugin);
/// ```
pub struct AttributePlugin;

impl Plugin for AttributePlugin {
    fn build(&self, app: &mut App) {
        app
            // Register systems
            .add_systems(
                PostUpdate,
                clamp_attributes_system.before(trigger_attribute_change_events_system),
            )
            .add_systems(PostUpdate, trigger_attribute_change_events_system);
    }
}
