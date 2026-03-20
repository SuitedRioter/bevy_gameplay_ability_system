//! Attribute system plugin.
//!
//! This plugin registers the attribute lifecycle hooks resource.

use super::hooks::AttributeLifecycleHooks;
use bevy::prelude::*;

/// Plugin that adds attribute system functionality.
pub struct AttributePlugin;

impl Plugin for AttributePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AttributeLifecycleHooks>();
    }
}
