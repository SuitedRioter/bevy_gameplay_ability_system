use bevy::DefaultPlugins;
use bevy::app::App;
use bevy_gameplay_ability_system::gameplay_ability_system_plugin::GameplayAbilitySystemPlugin;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(GameplayAbilitySystemPlugin)
        .run();
}
