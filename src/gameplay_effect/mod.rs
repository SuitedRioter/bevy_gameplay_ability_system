use bevy::app::{App, Plugin, Startup};
use bevy::log::info;

pub struct GameplayEffectPlugin;

impl Plugin for GameplayEffectPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

fn setup() {
    info!("GameplayEffectPlugin")
}
