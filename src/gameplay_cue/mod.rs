use bevy::app::{App, Plugin, Startup};
use bevy::log::info;

pub struct GameplayCuePlugin;

impl Plugin for GameplayCuePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

fn setup() {
    info!("GameplayCuePlugin")
}
