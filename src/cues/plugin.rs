//! GameplayCue plugin.
//!
//! This module provides the plugin for the gameplay cue system.

use super::manager::GameplayCueManager;
use super::systems::*;
use bevy::prelude::*;

/// Plugin for the gameplay cue system.
///
/// This plugin registers the GameplayCueManager resource and all cue systems.
///
/// # Example
///
/// ```
/// use bevy::prelude::*;
/// use bevy_gameplay_ability_system::cues::CuePlugin;
///
/// App::new()
///     .add_plugins(CuePlugin)
///     .run();
/// ```
pub struct CuePlugin;

impl Plugin for CuePlugin {
    fn build(&self, app: &mut App) {
        // Register resources
        app.init_resource::<GameplayCueManager>();

        // TODO: Register events with Bevy 0.18 observer pattern
        // For now, systems will handle cue execution directly

        // Register systems
        app.add_systems(
            Update,
            (
                handle_gameplay_cue_system,
                route_gameplay_cue_system,
                execute_static_cues_system,
                manage_cue_actors_system,
                cleanup_finished_cues_system,
                update_while_active_cues_system,
            )
                .chain(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_builds() {
        let mut app = App::new();
        app.add_plugins(CuePlugin);

        // Verify resource is registered
        assert!(app.world().get_resource::<GameplayCueManager>().is_some());
    }
}
