use bevy::DefaultPlugins;
use bevy::app::App;
use bevy::prelude::*;
use bevy_gameplay_ability_system::attributes::core::{AttributeSet, GameplayAttributeId};
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
                })
                .with_children(|attr_parent| {
                    attr_parent.spawn((Name::new("Health"), Health::with_value(300.0)));
                });
        });
    // camera
    commands.spawn((
        Name::new("Camera"),
        Camera3d::default(),
        Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

define_attribute!(MaxHealth);
define_attribute_manual!(Health);

impl GameplayAttribute for Health {
    fn attribute_id() -> GameplayAttributeId {
        GameplayAttributeId::of::<Self>()
    }

    fn get_base_value(&self) -> f32 {
        self.base_value
    }
    fn get_current_value(&self) -> f32 {
        self.current_value
    }

    /// 设置current_value
    fn set_current_value_internal(&mut self, value: f32) {
        self.current_value = value;
    }
    /// 设置base_value
    fn set_base_value_internal(&mut self, value: f32) {
        self.base_value = value;
    }
}
