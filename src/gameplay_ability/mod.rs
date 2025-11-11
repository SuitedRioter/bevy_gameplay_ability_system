mod components;
mod states;
mod systems;

use bevy::app::{App, Plugin, Startup};
use bevy::log::info;

pub struct GameplayAbilityPlugin;

impl Plugin for GameplayAbilityPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

fn setup() {
    info!("GameplayAbilityPlugin")
}
