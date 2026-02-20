//! Ability activation example.
//!
//! This example demonstrates how to:
//! - Define gameplay abilities
//! - Grant abilities to entities
//! - Activate abilities with tag requirements
//! - Apply ability costs and cooldowns
//! - Cancel abilities based on tags

use bevy::prelude::*;
use bevy_gameplay_ability_system::prelude::*;
use bevy_gameplay_tag::gameplay_tag::GameplayTag;
use bevy_gameplay_tag::gameplay_tag_count_container::GameplayTagCountContainer;
use bevy_gameplay_tag::{GameplayTagsManager, GameplayTagsPlugin};

fn main() {
    App::new()
        .add_plugins(GameplayTagsPlugin::with_data_path(
            "assets/gameplay_tags.json".to_string(),
        ))
        .add_plugins(MinimalPlugins)
        .add_plugins(AttributePlugin)
        .add_plugins(EffectPlugin)
        .add_plugins(AbilityPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (try_activate_abilities, print_ability_states))
        .run();
}

/// Define a custom attribute set for a character.
struct CharacterAttributes;

impl AttributeSetDefinition for CharacterAttributes {
    fn attribute_names() -> &'static [&'static str] {
        &["Health", "Mana", "Stamina"]
    }

    fn attribute_metadata(name: &str) -> Option<AttributeMetadata> {
        match name {
            "Health" => Some(AttributeMetadata {
                name: "Health",
                min_value: Some(0.0),
                max_value: Some(100.0),
            }),
            "Mana" => Some(AttributeMetadata {
                name: "Mana",
                min_value: Some(0.0),
                max_value: Some(100.0),
            }),
            "Stamina" => Some(AttributeMetadata {
                name: "Stamina",
                min_value: Some(0.0),
                max_value: Some(100.0),
            }),
            _ => None,
        }
    }

    fn default_value(name: &str) -> f32 {
        match name {
            "Health" => 100.0,
            "Mana" => 100.0,
            "Stamina" => 100.0,
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

/// Setup system that creates abilities and a character.
fn setup(mut commands: Commands, tags_manager: Res<GameplayTagsManager>) {
    info!("=== Ability Activation Example ===");

    // Create the character entity with tags
    let character = commands.spawn(GameplayTagCountContainer::default()).id();

    // Add tags to the character
    commands
        .entity(character)
        .insert(GameplayTagCountContainer::default());

    CharacterAttributes::create_attributes(&mut commands, character);

    info!("Character created with ID: {:?}", character);

    // Initialize the ability registry
    let mut ability_registry = AbilityRegistry::default();

    // Define a fireball ability (costs mana, has cooldown)
    let fireball = AbilityDefinition::new("ability.fireball")
        .with_instancing_policy(InstancingPolicy::InstancedPerExecution)
        .add_activation_required_tag(GameplayTag::new("State.Alive"), &tags_manager)
        .add_activation_blocked_tag(GameplayTag::new("State.Stunned"), &tags_manager)
        .add_activation_owned_tag(GameplayTag::new("Ability.Casting"), &tags_manager)
        .with_cost_effect("effect.cost.mana".to_string())
        .with_cooldown_effect("effect.cooldown.fireball".to_string());

    // Define a melee attack ability (costs stamina)
    let melee_attack = AbilityDefinition::new("ability.melee")
        .with_instancing_policy(InstancingPolicy::NonInstanced)
        .add_activation_required_tag(GameplayTag::new("State.Alive"), &tags_manager)
        .add_activation_blocked_tag(GameplayTag::new("State.Disarmed"), &tags_manager)
        .with_cost_effect("effect.cost.stamina".to_string());

    // Define a defensive ability (no cost, but requires not attacking)
    let block = AbilityDefinition::new("ability.block")
        .with_instancing_policy(InstancingPolicy::InstancedPerActor)
        .add_activation_required_tag(GameplayTag::new("State.Alive"), &tags_manager)
        .add_activation_blocked_tag(GameplayTag::new("Ability.Attacking"), &tags_manager)
        .add_activation_owned_tag(GameplayTag::new("Ability.Blocking"), &tags_manager)
        .add_cancel_on_tag_added(GameplayTag::new("Ability.Attacking"), &tags_manager);

    // Register abilities
    ability_registry.register(fireball);
    ability_registry.register(melee_attack);
    ability_registry.register(block);

    commands.insert_resource(ability_registry);

    // Initialize effect registry for costs and cooldowns
    let mut effect_registry = GameplayEffectRegistry::default();

    // Mana cost effect
    let mana_cost = GameplayEffectDefinition::new("effect.cost.mana")
        .with_duration_policy(DurationPolicy::Instant)
        .add_modifier(ModifierInfo {
            attribute_name: "Mana".to_string(),
            operation: ModifierOperation::AddBase,
            magnitude: MagnitudeCalculation::ScalableFloat { base_value: -25.0 },
        });

    // Stamina cost effect
    let stamina_cost = GameplayEffectDefinition::new("effect.cost.stamina")
        .with_duration_policy(DurationPolicy::Instant)
        .add_modifier(ModifierInfo {
            attribute_name: "Stamina".to_string(),
            operation: ModifierOperation::AddBase,
            magnitude: MagnitudeCalculation::ScalableFloat { base_value: -15.0 },
        });

    // Cooldown effect
    let cooldown = GameplayEffectDefinition::new("effect.cooldown.fireball")
        .with_duration_policy(DurationPolicy::HasDuration)
        .with_duration(3.0)
        .grant_tag(GameplayTag::new("Cooldown.Fireball"), &tags_manager);

    effect_registry.register(mana_cost);
    effect_registry.register(stamina_cost);
    effect_registry.register(cooldown);

    commands.insert_resource(effect_registry);

    // Grant abilities to the character
    let fireball_spec = commands
        .spawn((
            AbilitySpec {
                definition_id: "ability.fireball".to_string(),
                level: 1,
                input_id: Some(1),
                is_active: false,
            },
            AbilityOwner(character),
            AbilityState::Ready,
        ))
        .id();

    let melee_spec = commands
        .spawn((
            AbilitySpec {
                definition_id: "ability.melee".to_string(),
                level: 1,
                input_id: Some(2),
                is_active: false,
            },
            AbilityOwner(character),
            AbilityState::Ready,
        ))
        .id();

    let block_spec = commands
        .spawn((
            AbilitySpec {
                definition_id: "ability.block".to_string(),
                level: 1,
                input_id: Some(3),
                is_active: false,
            },
            AbilityOwner(character),
            AbilityState::Ready,
        ))
        .id();

    info!("Granted 3 abilities:");
    info!("  - Fireball (ID: {:?})", fireball_spec);
    info!("  - Melee Attack (ID: {:?})", melee_spec);
    info!("  - Block (ID: {:?})", block_spec);
}

/// System that tries to activate abilities at different times.
fn try_activate_abilities(
    time: Res<Time>,
    mut commands: Commands,
    ability_specs: Query<(Entity, &AbilitySpec, &AbilityOwner)>,
) {
    let elapsed = time.elapsed_secs();

    // Try to activate fireball at t=2s
    if (elapsed - 2.0).abs() < time.delta_secs() {
        info!("\n[t=2s] Trying to activate Fireball...");
        for (spec_entity, spec, owner) in ability_specs.iter() {
            if spec.definition_id == "ability.fireball" {
                commands.trigger(TryActivateAbilityEvent {
                    ability_spec: spec_entity,
                    owner: owner.0,
                });
            }
        }
    }

    // Try to activate melee at t=4s
    if (elapsed - 4.0).abs() < time.delta_secs() {
        info!("\n[t=4s] Trying to activate Melee Attack...");
        for (spec_entity, spec, owner) in ability_specs.iter() {
            if spec.definition_id == "ability.melee" {
                commands.trigger(TryActivateAbilityEvent {
                    ability_spec: spec_entity,
                    owner: owner.0,
                });
            }
        }
    }

    // Try to activate block at t=6s
    if (elapsed - 6.0).abs() < time.delta_secs() {
        info!("\n[t=6s] Trying to activate Block...");
        for (spec_entity, spec, owner) in ability_specs.iter() {
            if spec.definition_id == "ability.block" {
                commands.trigger(TryActivateAbilityEvent {
                    ability_spec: spec_entity,
                    owner: owner.0,
                });
            }
        }
    }
}

/// System that prints ability states periodically.
fn print_ability_states(
    time: Res<Time>,
    ability_specs: Query<(&AbilitySpec, &AbilityState)>,
    attributes: Query<(&AttributeData, &AttributeName)>,
) {
    // Print every 2 seconds
    if time.elapsed_secs() % 2.0 < time.delta_secs() {
        info!("\n--- Current State ---");

        info!("Abilities:");
        for (spec, state) in ability_specs.iter() {
            info!("  {}: {:?}", spec.definition_id, state);
        }

        info!("Resources:");
        for (attr, name) in attributes.iter() {
            if name.0 == "Mana" || name.0 == "Stamina" {
                info!("  {}: {:.1}", name.0, attr.current_value);
            }
        }
    }
}
