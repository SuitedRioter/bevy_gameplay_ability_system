use bevy::DefaultPlugins;
use bevy::app::App;
use bevy::prelude::*;
use bevy_gameplay_ability_system::attributes::macros::Health;
use bevy_gameplay_ability_system::gameplay_ability_system_plugin::GameplayAbilitySystemPlugin;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(GameplayAbilitySystemPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((Name::new("Player"), Health::new()));
    // camera
    commands.spawn((
        Name::new("Camera"),
        Camera3d::default(),
        Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
