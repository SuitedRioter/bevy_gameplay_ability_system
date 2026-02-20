//! Gameplay effects example.
//!
//! This example demonstrates how to:
//! - Define gameplay effects
//! - Apply instant effects (modify BaseValue)
//! - Apply duration effects (modify CurrentValue temporarily)
//! - Apply infinite effects
//! - Use effect modifiers with different operations
//! - Handle effect stacking

use bevy::prelude::*;
use bevy_gameplay_ability_system::prelude::*;
use bevy_gameplay_tag::{GameplayTagsManager, gameplay_tag::GameplayTag};

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(AttributePlugin)
        .add_plugins(EffectPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (apply_effects_over_time, print_attribute_state))
        .run();
}

/// Define a custom attribute set for a character.
struct CharacterAttributes;

impl AttributeSetDefinition for CharacterAttributes {
    fn attribute_names() -> &'static [&'static str] {
        &["Health", "MaxHealth", "Damage", "AttackSpeed"]
    }

    fn attribute_metadata(name: &str) -> Option<AttributeMetadata> {
        match name {
            "Health" => Some(AttributeMetadata {
                name: "Health",
                min_value: Some(0.0),
                max_value: None,
            }),
            "MaxHealth" => Some(AttributeMetadata {
                name: "MaxHealth",
                min_value: Some(1.0),
                max_value: None,
            }),
            "Damage" => Some(AttributeMetadata {
                name: "Damage",
                min_value: Some(0.0),
                max_value: None,
            }),
            "AttackSpeed" => Some(AttributeMetadata {
                name: "AttackSpeed",
                min_value: Some(0.1),
                max_value: None,
            }),
            _ => None,
        }
    }

    fn default_value(name: &str) -> f32 {
        match name {
            "Health" => 100.0,
            "MaxHealth" => 100.0,
            "Damage" => 10.0,
            "AttackSpeed" => 1.0,
            _ => 0.0,
        }
    }

    fn create_attributes(commands: &mut Commands, owner: Entity) -> Vec<Entity> {
        let mut attribute_entities = Vec::new();

        for name in Self::attribute_names() {
            let metadata = Self::attribute_metadata(name).unwrap();
            let default_value = Self::default_value(name);

            let attr_entity = commands
                .spawn((
                    AttributeData {
                        base_value: default_value,
                        current_value: default_value,
                    },
                    AttributeName(name.to_string()),
                    AttributeOwner(owner),
                    AttributeMetadataComponent(metadata),
                ))
                .id();

            attribute_entities.push(attr_entity);
        }

        attribute_entities
    }
}

/// Setup system that creates effects and a character.
fn setup(mut commands: Commands, tags_manager: Res<GameplayTagsManager>) {
    info!("=== Gameplay Effects Example ===");

    // Create the character entity
    let character = commands.spawn_empty().id();
    CharacterAttributes::create_attributes(&mut commands, character);

    info!("Character created with ID: {:?}", character);

    // Initialize the effect registry
    let mut registry = GameplayEffectRegistry::default();

    // Define an instant damage effect
    let damage_effect = GameplayEffectDefinition::new("effect.damage.instant")
        .with_duration_policy(DurationPolicy::Instant)
        .add_modifier(ModifierInfo {
            attribute_name: "Health".to_string(),
            operation: ModifierOperation::AddBase,
            magnitude: MagnitudeCalculation::ScalableFloat { base_value: -20.0 },
        });

    // Define a duration-based buff (increases damage for 5 seconds)
    let damage_buff = GameplayEffectDefinition::new("effect.buff.damage")
        .with_duration_policy(DurationPolicy::HasDuration)
        .with_duration(5.0)
        .add_modifier(ModifierInfo {
            attribute_name: "Damage".to_string(),
            operation: ModifierOperation::AddCurrent,
            magnitude: MagnitudeCalculation::ScalableFloat { base_value: 15.0 },
        })
        .grant_tag(GameplayTag::new("State.Buffed"), &tags_manager);

    // Define an infinite effect (permanent stat increase)
    let permanent_health_boost = GameplayEffectDefinition::new("effect.permanent.health")
        .with_duration_policy(DurationPolicy::Infinite)
        .add_modifier(ModifierInfo {
            attribute_name: "MaxHealth".to_string(),
            operation: ModifierOperation::AddBase,
            magnitude: MagnitudeCalculation::ScalableFloat { base_value: 50.0 },
        });

    // Define a multiplicative effect (attack speed buff)
    let attack_speed_buff = GameplayEffectDefinition::new("effect.buff.attackspeed")
        .with_duration_policy(DurationPolicy::HasDuration)
        .with_duration(3.0)
        .add_modifier(ModifierInfo {
            attribute_name: "AttackSpeed".to_string(),
            operation: ModifierOperation::MultiplyAdditive,
            magnitude: MagnitudeCalculation::ScalableFloat {
                base_value: 0.5, // 50% increase
            },
        });

    // Register effects
    registry.register(damage_effect);
    registry.register(damage_buff);
    registry.register(permanent_health_boost);
    registry.register(attack_speed_buff);

    commands.insert_resource(registry);

    info!("Registered 4 gameplay effects:");
    info!("  - effect.damage.instant (Instant damage)");
    info!("  - effect.buff.damage (Duration buff)");
    info!("  - effect.permanent.health (Infinite boost)");
    info!("  - effect.buff.attackspeed (Multiplicative buff)");
}

/// System that applies effects over time for demonstration.
fn apply_effects_over_time(
    time: Res<Time>,
    mut commands: Commands,
    registry: Res<GameplayEffectRegistry>,
    characters: Query<Entity, With<AttributeOwner>>,
) {
    let elapsed = time.elapsed_secs();

    // Apply different effects at different times
    if (elapsed - 2.0).abs() < time.delta_secs() {
        info!("\n[t=2s] Applying instant damage effect...");
        if let Some(character) = characters.iter().next() {
            if let Some(effect) = registry.get("effect.damage.instant") {
                commands.trigger(ApplyGameplayEffectEvent {
                    target: character,
                    effect_id: effect.id.clone(),
                    level: 1,
                    instigator: None,
                });
            }
        }
    }

    if (elapsed - 4.0).abs() < time.delta_secs() {
        info!("\n[t=4s] Applying permanent health boost...");
        if let Some(character) = characters.iter().next() {
            if let Some(effect) = registry.get("effect.permanent.health") {
                commands.trigger(ApplyGameplayEffectEvent {
                    target: character,
                    effect_id: effect.id.clone(),
                    level: 1,
                    instigator: None,
                });
            }
        }
    }

    if (elapsed - 6.0).abs() < time.delta_secs() {
        info!("\n[t=6s] Applying damage buff (5 second duration)...");
        if let Some(character) = characters.iter().next() {
            if let Some(effect) = registry.get("effect.buff.damage") {
                commands.trigger(ApplyGameplayEffectEvent {
                    target: character,
                    effect_id: effect.id.clone(),
                    level: 1,
                    instigator: None,
                });
            }
        }
    }

    if (elapsed - 8.0).abs() < time.delta_secs() {
        info!("\n[t=8s] Applying attack speed buff (3 second duration)...");
        if let Some(character) = characters.iter().next() {
            if let Some(effect) = registry.get("effect.buff.attackspeed") {
                commands.trigger(ApplyGameplayEffectEvent {
                    target: character,
                    effect_id: effect.id.clone(),
                    level: 1,
                    instigator: None,
                });
            }
        }
    }
}

/// System that prints attribute state periodically.
fn print_attribute_state(time: Res<Time>, attributes: Query<(&AttributeData, &AttributeName)>) {
    // Print every 2 seconds
    if time.elapsed_secs() % 2.0 < time.delta_secs() {
        info!("\n--- Current Attribute State ---");
        for (attr, name) in attributes.iter() {
            info!(
                "  {}: Base={:.1}, Current={:.1}",
                name.0, attr.base_value, attr.current_value
            );
        }
    }
}
