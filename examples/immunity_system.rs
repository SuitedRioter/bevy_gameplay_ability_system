//! Example demonstrating the immunity system.
//!
//! Shows how to grant immunity to specific effect types using immunity tags.

use bevy::prelude::*;
use bevy_gameplay_ability_system::{GasPlugin, attributes::*, core::ImmunityTags, effects::*};
use bevy_gameplay_tag::{GameplayTag, GameplayTagsManager, GameplayTagsPlugin};

fn main() {
    App::new()
        .add_plugins((
            MinimalPlugins,
            GameplayTagsPlugin::with_data_path("assets/gameplay_tags.json".to_string()),
            GasPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, check_results)
        .run();
}

struct CombatAttributeSet;

impl AttributeSetDefinition for CombatAttributeSet {
    fn attribute_names() -> &'static [&'static str] {
        &["Health"]
    }

    fn attribute_metadata(name: &str) -> Option<AttributeMetadata> {
        match name {
            "Health" => Some(
                AttributeMetadata::new("Health")
                    .with_min(0.0)
                    .with_max(100.0),
            ),
            _ => None,
        }
    }

    fn default_value(name: &str) -> f32 {
        match name {
            "Health" => 100.0,
            _ => 0.0,
        }
    }
}

fn setup(
    mut commands: Commands,
    mut registry: ResMut<GameplayEffectRegistry>,
    tags_manager: Res<GameplayTagsManager>,
) {
    info!("Setting up immunity system example");

    // Create two targets
    let vulnerable_target = commands.spawn_empty().id();
    CombatAttributeSet::create_attributes(&mut commands, vulnerable_target);

    let immune_target = commands.spawn_empty().id();
    CombatAttributeSet::create_attributes(&mut commands, immune_target);

    // Grant immunity to fire damage using GameplayTag
    let fire_immunity_tag = GameplayTag::new("Effect.Damage.Fire");
    let mut immunity_container = bevy_gameplay_tag::GameplayTagCountContainer::default();
    immunity_container
        .explicit_tags
        .add_tag(fire_immunity_tag.clone(), &tags_manager);
    commands
        .entity(immune_target)
        .insert(ImmunityTags(immunity_container));

    // Register a fire damage effect
    let fire_damage = GameplayEffectDefinition::new("fire_damage")
        .with_duration_policy(DurationPolicy::Instant)
        .with_immunity_tag(fire_immunity_tag, &tags_manager)
        .add_modifier(ModifierInfo::new(
            "Health",
            ModifierOperation::AddCurrent,
            MagnitudeCalculation::scalar(-20.0),
        ));

    registry.register(fire_damage);

    // Register a physical damage effect (no immunity tag)
    let physical_damage = GameplayEffectDefinition::new("physical_damage")
        .with_duration_policy(DurationPolicy::Instant)
        .add_modifier(ModifierInfo::new(
            "Health",
            ModifierOperation::AddCurrent,
            MagnitudeCalculation::scalar(-15.0),
        ));

    registry.register(physical_damage);

    // Apply fire damage to both targets
    commands.trigger(ApplyGameplayEffectEvent::new("fire_damage", vulnerable_target).with_level(1));

    commands.trigger(ApplyGameplayEffectEvent::new("fire_damage", immune_target).with_level(1));

    // Apply physical damage to immune target (should work)
    commands.trigger(ApplyGameplayEffectEvent::new("physical_damage", immune_target).with_level(1));

    info!("Vulnerable target should take fire damage: 100 -> 80");
    info!("Immune target should block fire damage but take physical: 100 -> 85");
}

fn check_results(time: Res<Time>, attributes: Query<(&AttributeData, &AttributeName, &ChildOf)>) {
    static mut PRINTED: bool = false;
    if time.elapsed_secs() > 0.2 && unsafe { !PRINTED } {
        unsafe { PRINTED = true };

        for (data, name, _) in attributes.iter() {
            if name.as_str() == "Health" {
                info!("Final Health: {}", data.current_value);
            }
        }

        info!("Example complete!");
    }
}
