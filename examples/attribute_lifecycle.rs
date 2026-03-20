//! Example demonstrating attribute lifecycle hooks.
//!
//! Shows how to use Pre/Post hooks in AttributeSet.

use bevy::prelude::*;
use bevy_gameplay_ability_system::attributes::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins(AttributePlugin);

    // Register hooks once at startup
    CharacterAttributes::register_hooks(app.world_mut());

    app.add_systems(Startup, setup).run();
}

// Define AttributeSet with lifecycle hooks
struct CharacterAttributes;

impl AttributeSetDefinition for CharacterAttributes {
    fn attribute_names() -> &'static [&'static str] {
        &["Health", "Mana"]
    }

    fn attribute_metadata(name: &str) -> Option<AttributeMetadata> {
        match name {
            "Health" => Some(
                AttributeMetadata::new("Health")
                    .with_min(0.0)
                    .with_max(100.0),
            ),
            "Mana" => Some(AttributeMetadata::new("Mana").with_min(0.0).with_max(100.0)),
            _ => None,
        }
    }

    fn default_value(name: &str) -> f32 {
        match name {
            "Health" | "Mana" => 100.0,
            _ => 0.0,
        }
    }

    // Pre hook - can modify new_value
    fn pre_attribute_change(context: &mut AttributeModifyContext) {
        info!(
            "PRE: {} changing {} -> {}",
            context.attribute_name.as_ref(),
            context.old_value,
            context.new_value
        );

        // Example: Minimum damage of 1
        if context.new_value < context.old_value {
            let damage = context.old_value - context.new_value;
            if damage < 1.0 {
                context.new_value = context.old_value - 1.0;
            }
        }
    }

    // Post hook - react to changes
    fn post_attribute_change(context: &AttributeModifyContext) {
        info!(
            "POST: {} changed {} -> {}",
            context.attribute_name.as_ref(),
            context.old_value,
            context.new_value
        );

        if context.attribute_name.as_ref() == "Health" && context.new_value <= 0.0 {
            warn!("Character died!");
        }
    }
}

fn setup(mut commands: Commands) {
    let player = commands.spawn_empty().id();
    CharacterAttributes::create_attributes(&mut commands, player);

    info!("Player created with Health and Mana");
}
