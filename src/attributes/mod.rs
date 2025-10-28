use bevy::app::{App, Plugin, Startup};
use bevy::log::info;

pub struct AttributePlugin;

impl Plugin for AttributePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

fn setup() {
    info!("Attribute plugin loaded")
}
