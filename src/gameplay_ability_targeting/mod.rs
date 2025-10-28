use bevy::app::{App, Plugin, Startup};
use bevy::log::info;

pub struct GameplayAbilityTargetingPlugin;

impl Plugin for GameplayAbilityTargetingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

fn setup() {
    info!("GameplayAbilityTargetingPlugin")
}
