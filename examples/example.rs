use bevy::DefaultPlugins;
use bevy::app::App;
use bevy::prelude::*;
use bevy_gameplay_ability_system::attributes::core::AttributeSet;
use bevy_gameplay_ability_system::gameplay_ability_system_plugin::GameplayAbilitySystemPlugin;
use bevy_gameplay_ability_system::{GameplayAttribute, define_attribute, define_attribute_manual};
use bevy_gameplay_tag::gameplay_tag_container::GameplayTagContainer;
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
    commands
        .spawn((Name::new("Player"), GameplayTagContainer::new()))
        .with_children(|parent| {
            parent
                .spawn((Name::new("Attributes"), AttributeSet))
                .with_children(|attr_parent| {
                    attr_parent.spawn((Name::new("MaxHealth"), MaxHealth::with_value(500.0)));
                });
        });
    // camera
    commands.spawn((
        Name::new("Camera"),
        Camera3d::default(),
        Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

define_attribute_manual!(Health, default = 100.0);
define_attribute!(MaxHealth, min = 0.0, max = 1000.0, default = 100.0);

impl GameplayAttribute for Health {}
