//! Complete RPG Example
//!
//! This example demonstrates a complete RPG combat scenario using all features
//! of the Gameplay Ability System:
//! - Custom attribute sets (Health, Mana, Stamina, etc.)
//! - Multiple gameplay effects (damage, healing, buffs, debuffs)
//! - Combat abilities with costs and cooldowns
//! - Tag-based requirements and blocking
//! - GameplayCues for visual feedback
//!
//! This example simulates a turn-based combat between a player and an enemy.

use bevy::prelude::*;
use bevy_gameplay_ability_system::prelude::*;
use bevy_gameplay_tag::gameplay_tag::GameplayTag;
use bevy_gameplay_tag::gameplay_tag_count_container::GameplayTagCountContainer;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GasPlugin)
        .add_systems(Startup, setup_game)
        .add_systems(
            Update,
            (simulate_combat, display_combat_log, handle_death).chain(),
        )
        .run();
}

// ============================================================================
// ATTRIBUTE DEFINITIONS
// ============================================================================

/// Character attribute set with RPG stats.
struct CharacterAttributes;

impl AttributeSetDefinition for CharacterAttributes {
    fn attribute_names() -> &'static [&'static str] {
        &[
            "Health",
            "MaxHealth",
            "Mana",
            "MaxMana",
            "Stamina",
            "MaxStamina",
            "Attack",
            "Defense",
            "MagicPower",
            "CritChance",
            "CritDamage",
        ]
    }

    fn attribute_metadata(name: &str) -> Option<AttributeMetadata> {
        match name {
            "Health" => Some(AttributeMetadata {
                name: "Health",
                min_value: Some(0.0),
                max_value: Some(100.0),
            }),
            "MaxHealth" => Some(AttributeMetadata {
                name: "MaxHealth",
                min_value: Some(1.0),
                max_value: Some(999.0),
            }),
            "Mana" => Some(AttributeMetadata {
                name: "Mana",
                min_value: Some(0.0),
                max_value: Some(50.0),
            }),
            "MaxMana" => Some(AttributeMetadata {
                name: "MaxMana",
                min_value: Some(1.0),
                max_value: Some(999.0),
            }),
            "Stamina" => Some(AttributeMetadata {
                name: "Stamina",
                min_value: Some(0.0),
                max_value: Some(100.0),
            }),
            "MaxStamina" => Some(AttributeMetadata {
                name: "MaxStamina",
                min_value: Some(1.0),
                max_value: Some(999.0),
            }),
            "Attack" => Some(AttributeMetadata {
                name: "Attack",
                min_value: Some(0.0),
                max_value: None,
            }),
            "Defense" => Some(AttributeMetadata {
                name: "Defense",
                min_value: Some(0.0),
                max_value: None,
            }),
            "MagicPower" => Some(AttributeMetadata {
                name: "MagicPower",
                min_value: Some(0.0),
                max_value: None,
            }),
            "CritChance" => Some(AttributeMetadata {
                name: "CritChance",
                min_value: Some(0.0),
                max_value: Some(1.0),
            }),
            "CritDamage" => Some(AttributeMetadata {
                name: "CritDamage",
                min_value: Some(1.0),
                max_value: Some(10.0),
            }),
            _ => None,
        }
    }

    fn default_value(name: &str) -> f32 {
        match name {
            "Health" => 100.0,
            "MaxHealth" => 100.0,
            "Mana" => 50.0,
            "MaxMana" => 50.0,
            "Stamina" => 100.0,
            "MaxStamina" => 100.0,
            "Attack" => 10.0,
            "Defense" => 5.0,
            "MagicPower" => 8.0,
            "CritChance" => 0.15,
            "CritDamage" => 1.5,
            _ => 0.0,
        }
    }

    fn create_attributes(commands: &mut Commands, owner: Entity) -> Vec<Entity> {
        let mut attribute_entities = Vec::new();

        for name in Self::attribute_names() {
            let metadata = Self::attribute_metadata(name).unwrap();
            let value = Self::default_value(name);

            let attr_entity = commands
                .spawn((
                    AttributeData {
                        base_value: value,
                        current_value: value,
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

// ============================================================================
// COMPONENTS
// ============================================================================

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Enemy;

#[derive(Component)]
struct CombatStats {
    turn_count: u32,
    damage_dealt: f32,
    damage_taken: f32,
    abilities_used: u32,
}

#[derive(Component)]
struct CombatAI {
    next_action_timer: f32,
    action_delay: f32,
}

#[derive(Resource)]
struct CombatLog {
    messages: Vec<String>,
}

#[derive(Resource)]
struct EffectRegistry {
    definitions: Vec<GameplayEffectDefinition>,
}

#[derive(Resource)]
struct AbilityRegistry {
    definitions: Vec<AbilityDefinition>,
}

// ============================================================================
// SETUP
// ============================================================================

fn setup_game(mut commands: Commands) {
    // Initialize resources
    commands.insert_resource(CombatLog {
        messages: Vec::new(),
    });

    // Create effect definitions
    let effect_registry = create_effect_registry();
    commands.insert_resource(effect_registry);

    // Create ability definitions
    let ability_registry = create_ability_registry();
    commands.insert_resource(ability_registry);

    // Spawn player
    let player = commands
        .spawn((
            Player,
            Name::new("Hero"),
            CombatStats {
                turn_count: 0,
                damage_dealt: 0.0,
                damage_taken: 0.0,
                abilities_used: 0,
            },
            GameplayTagCountContainer::default(),
        ))
        .id();

    // Create player attributes
    CharacterAttributes::create_attributes(&mut commands, player);

    // Grant player abilities
    grant_player_abilities(&mut commands, player);

    // Spawn enemy
    let enemy = commands
        .spawn((
            Enemy,
            Name::new("Goblin"),
            CombatStats {
                turn_count: 0,
                damage_dealt: 0.0,
                damage_taken: 0.0,
                abilities_used: 0,
            },
            CombatAI {
                next_action_timer: 2.0,
                action_delay: 2.0,
            },
            GameplayTagCountContainer::default(),
        ))
        .id();

    // Create enemy attributes (weaker stats)
    create_enemy_attributes(&mut commands, enemy);

    // Grant enemy abilities
    grant_enemy_abilities(&mut commands, enemy);

    info!("=== RPG Combat Example Started ===");
    info!("Player: Hero vs Enemy: Goblin");
    info!("Press SPACE to use abilities");
}

fn create_enemy_attributes(commands: &mut Commands, owner: Entity) {
    let attributes = [
        ("Health", 80.0, Some(0.0), Some(80.0)),
        ("MaxHealth", 80.0, Some(1.0), Some(999.0)),
        ("Mana", 30.0, Some(0.0), Some(30.0)),
        ("MaxMana", 30.0, Some(1.0), Some(999.0)),
        ("Stamina", 80.0, Some(0.0), Some(80.0)),
        ("MaxStamina", 80.0, Some(1.0), Some(999.0)),
        ("Attack", 8.0, Some(0.0), None),
        ("Defense", 3.0, Some(0.0), None),
        ("MagicPower", 5.0, Some(0.0), None),
        ("CritChance", 0.10, Some(0.0), Some(1.0)),
        ("CritDamage", 1.3, Some(1.0), Some(10.0)),
    ];

    for (name, value, min, max) in attributes {
        let metadata = AttributeMetadata {
            name,
            min_value: min,
            max_value: max,
        };

        commands.spawn((
            AttributeData {
                base_value: value,
                current_value: value,
            },
            AttributeName(name.to_string()),
            AttributeOwner(owner),
            AttributeMetadataComponent(metadata),
        ));
    }
}

// ============================================================================
// EFFECT DEFINITIONS
// ============================================================================

fn create_effect_registry() -> EffectRegistry {
    let mut definitions = Vec::new();

    // Damage effects
    definitions.push(
        GameplayEffectDefinition::new("effect.damage.physical")
            .with_duration_policy(DurationPolicy::Instant)
            .add_modifier(ModifierInfo {
                attribute_name: "Health".to_string(),
                operation: ModifierOperation::AddBase,
                magnitude: MagnitudeCalculation::ScalableFloat { base_value: -15.0 },
            }),
    );

    definitions.push(
        GameplayEffectDefinition::new("effect.damage.magic")
            .with_duration_policy(DurationPolicy::Instant)
            .add_modifier(ModifierInfo {
                attribute_name: "Health".to_string(),
                operation: ModifierOperation::AddBase,
                magnitude: MagnitudeCalculation::ScalableFloat { base_value: -20.0 },
            }),
    );

    // Healing effects
    definitions.push(
        GameplayEffectDefinition::new("effect.heal.instant")
            .with_duration_policy(DurationPolicy::Instant)
            .add_modifier(ModifierInfo {
                attribute_name: "Health".to_string(),
                operation: ModifierOperation::AddBase,
                magnitude: MagnitudeCalculation::ScalableFloat { base_value: 25.0 },
            }),
    );

    definitions.push(
        GameplayEffectDefinition::new("effect.heal.overtime")
            .with_duration_policy(DurationPolicy::HasDuration)
            .with_duration(6.0)
            .with_period(2.0)
            .add_modifier(ModifierInfo {
                attribute_name: "Health".to_string(),
                operation: ModifierOperation::AddBase,
                magnitude: MagnitudeCalculation::ScalableFloat { base_value: 5.0 },
            })
            .grant_tag(GameplayTag::new("Effect.HealOverTime")),
    );

    // Buff effects
    definitions.push(
        GameplayEffectDefinition::new("effect.buff.attack")
            .with_duration_policy(DurationPolicy::HasDuration)
            .with_duration(10.0)
            .add_modifier(ModifierInfo {
                attribute_name: "Attack".to_string(),
                operation: ModifierOperation::MultiplyAdditive,
                magnitude: MagnitudeCalculation::ScalableFloat { base_value: 0.5 },
            })
            .grant_tag(GameplayTag::new("Effect.Buff.Attack")),
    );

    definitions.push(
        GameplayEffectDefinition::new("effect.buff.defense")
            .with_duration_policy(DurationPolicy::HasDuration)
            .with_duration(8.0)
            .add_modifier(ModifierInfo {
                attribute_name: "Defense".to_string(),
                operation: ModifierOperation::MultiplyAdditive,
                magnitude: MagnitudeCalculation::ScalableFloat { base_value: 0.3 },
            })
            .grant_tag(GameplayTag::new("Effect.Buff.Defense")),
    );

    // Debuff effects
    definitions.push(
        GameplayEffectDefinition::new("effect.debuff.stun")
            .with_duration_policy(DurationPolicy::HasDuration)
            .with_duration(3.0)
            .grant_tag(GameplayTag::new("State.Stunned")),
    );

    definitions.push(
        GameplayEffectDefinition::new("effect.debuff.poison")
            .with_duration_policy(DurationPolicy::HasDuration)
            .with_duration(10.0)
            .with_period(2.0)
            .add_modifier(ModifierInfo {
                attribute_name: "Health".to_string(),
                operation: ModifierOperation::AddBase,
                magnitude: MagnitudeCalculation::ScalableFloat { base_value: -3.0 },
            })
            .grant_tag(GameplayTag::new("Effect.Debuff.Poison")),
    );

    // Cost effects
    definitions.push(
        GameplayEffectDefinition::new("effect.cost.mana.small")
            .with_duration_policy(DurationPolicy::Instant)
            .add_modifier(ModifierInfo {
                attribute_name: "Mana".to_string(),
                operation: ModifierOperation::AddBase,
                magnitude: MagnitudeCalculation::ScalableFloat { base_value: -10.0 },
            }),
    );

    definitions.push(
        GameplayEffectDefinition::new("effect.cost.mana.medium")
            .with_duration_policy(DurationPolicy::Instant)
            .add_modifier(ModifierInfo {
                attribute_name: "Mana".to_string(),
                operation: ModifierOperation::AddBase,
                magnitude: MagnitudeCalculation::ScalableFloat { base_value: -20.0 },
            }),
    );

    definitions.push(
        GameplayEffectDefinition::new("effect.cost.stamina")
            .with_duration_policy(DurationPolicy::Instant)
            .add_modifier(ModifierInfo {
                attribute_name: "Stamina".to_string(),
                operation: ModifierOperation::AddBase,
                magnitude: MagnitudeCalculation::ScalableFloat { base_value: -15.0 },
            }),
    );

    // Cooldown effects
    definitions.push(
        GameplayEffectDefinition::new("effect.cooldown.attack")
            .with_duration_policy(DurationPolicy::HasDuration)
            .with_duration(1.5)
            .grant_tag(GameplayTag::new("Cooldown.Attack")),
    );

    definitions.push(
        GameplayEffectDefinition::new("effect.cooldown.spell")
            .with_duration_policy(DurationPolicy::HasDuration)
            .with_duration(3.0)
            .grant_tag(GameplayTag::new("Cooldown.Spell")),
    );

    definitions.push(
        GameplayEffectDefinition::new("effect.cooldown.heal")
            .with_duration_policy(DurationPolicy::HasDuration)
            .with_duration(5.0)
            .grant_tag(GameplayTag::new("Cooldown.Heal")),
    );

    EffectRegistry { definitions }
}

// ============================================================================
// ABILITY DEFINITIONS
// ============================================================================

fn create_ability_registry() -> AbilityRegistry {
    let mut definitions = Vec::new();

    // Basic attack
    definitions.push(
        AbilityDefinition::new("ability.attack.basic")
            .with_instancing_policy(InstancingPolicy::InstancedPerExecution)
            .add_activation_required_tag(GameplayTag::new("State.Alive"))
            .add_activation_blocked_tag(GameplayTag::new("State.Stunned"))
            .add_activation_blocked_tag(GameplayTag::new("Cooldown.Attack"))
            .add_cost_effect("effect.cost.stamina".to_string())
            .with_cooldown_effect("effect.cooldown.attack".to_string()),
    );

    // Fireball spell
    definitions.push(
        AbilityDefinition::new("ability.spell.fireball")
            .with_instancing_policy(InstancingPolicy::InstancedPerExecution)
            .add_activation_required_tag(GameplayTag::new("State.Alive"))
            .add_activation_blocked_tag(GameplayTag::new("State.Stunned"))
            .add_activation_blocked_tag(GameplayTag::new("Cooldown.Spell"))
            .add_cost_effect("effect.cost.mana.medium".to_string())
            .with_cooldown_effect("effect.cooldown.spell".to_string()),
    );

    // Healing spell
    definitions.push(
        AbilityDefinition::new("ability.spell.heal")
            .with_instancing_policy(InstancingPolicy::InstancedPerExecution)
            .add_activation_required_tag(GameplayTag::new("State.Alive"))
            .add_activation_blocked_tag(GameplayTag::new("State.Stunned"))
            .add_activation_blocked_tag(GameplayTag::new("Cooldown.Heal"))
            .add_cost_effect("effect.cost.mana.small".to_string())
            .with_cooldown_effect("effect.cooldown.heal".to_string()),
    );

    // Power strike
    definitions.push(
        AbilityDefinition::new("ability.attack.power")
            .with_instancing_policy(InstancingPolicy::InstancedPerExecution)
            .add_activation_required_tag(GameplayTag::new("State.Alive"))
            .add_activation_blocked_tag(GameplayTag::new("State.Stunned"))
            .add_activation_blocked_tag(GameplayTag::new("Cooldown.Attack"))
            .add_cost_effect("effect.cost.stamina".to_string())
            .with_cooldown_effect("effect.cooldown.attack".to_string()),
    );

    AbilityRegistry { definitions }
}

// ============================================================================
// ABILITY GRANTING
// ============================================================================

fn grant_player_abilities(commands: &mut Commands, owner: Entity) {
    let abilities = [
        "ability.attack.basic",
        "ability.spell.fireball",
        "ability.spell.heal",
        "ability.attack.power",
    ];

    for ability_id in abilities {
        commands.spawn((
            AbilitySpec {
                definition_id: ability_id.to_string(),
                level: 1,
                input_id: None,
                is_active: false,
            },
            AbilityOwner(owner),
        ));
    }
}

fn grant_enemy_abilities(commands: &mut Commands, owner: Entity) {
    let abilities = ["ability.attack.basic", "ability.attack.power"];

    for ability_id in abilities {
        commands.spawn((
            AbilitySpec {
                definition_id: ability_id.to_string(),
                level: 1,
                input_id: None,
                is_active: false,
            },
            AbilityOwner(owner),
        ));
    }
}

// ============================================================================
// COMBAT SIMULATION
// ============================================================================

fn simulate_combat(
    time: Res<Time>,
    mut commands: Commands,
    mut combat_log: ResMut<CombatLog>,
    effect_registry: Res<EffectRegistry>,
    ability_registry: Res<AbilityRegistry>,
    player_query: Query<Entity, With<Player>>,
    enemy_query: Query<Entity, With<Enemy>>,
    mut ai_query: Query<&mut CombatAI, With<Enemy>>,
    attributes_query: Query<(&AttributeData, &AttributeName, &AttributeOwner)>,
) {
    // Enemy AI
    for mut ai in ai_query.iter_mut() {
        ai.next_action_timer -= time.delta_secs();

        if ai.next_action_timer <= 0.0 {
            ai.next_action_timer = ai.action_delay;

            if let Ok(enemy) = enemy_query.single() {
                if let Ok(player) = player_query.single() {
                    // Check if enemy has enough stamina
                    let has_stamina = attributes_query.iter().any(|(data, name, owner)| {
                        owner.0 == enemy && name.0 == "Stamina" && data.current_value >= 15.0
                    });

                    if has_stamina {
                        // Enemy attacks player
                        let damage_effect = effect_registry
                            .definitions
                            .iter()
                            .find(|def| def.id == "effect.damage.physical")
                            .unwrap()
                            .clone();

                        apply_effect_to_target(&mut commands, player, damage_effect);

                        combat_log
                            .messages
                            .push(format!("Goblin attacks Hero for 15 damage!"));
                    }
                }
            }
        }
    }
}

fn apply_effect_to_target(
    commands: &mut Commands,
    target: Entity,
    effect_def: GameplayEffectDefinition,
) {
    let effect_entity = commands
        .spawn((
            ActiveGameplayEffect {
                definition_id: effect_def.id.clone(),
                level: 1,
                start_time: 0.0,
                stack_count: 1,
            },
            EffectTarget(target),
        ))
        .id();

    if effect_def.duration_policy == DurationPolicy::HasDuration {
        commands.entity(effect_entity).insert(EffectDuration {
            remaining: effect_def.duration_magnitude,
            total: effect_def.duration_magnitude,
        });
    }

    if effect_def.period > 0.0 {
        commands.entity(effect_entity).insert(PeriodicEffect {
            period: effect_def.period,
            time_until_next: effect_def.period,
        });
    }
}

// ============================================================================
// COMBAT LOG DISPLAY
// ============================================================================

fn display_combat_log(mut combat_log: ResMut<CombatLog>) {
    if !combat_log.messages.is_empty() {
        for message in combat_log.messages.drain(..) {
            info!("{}", message);
        }
    }
}

// ============================================================================
// DEATH HANDLING
// ============================================================================

fn handle_death(
    mut commands: Commands,
    mut combat_log: ResMut<CombatLog>,
    player_query: Query<Entity, With<Player>>,
    enemy_query: Query<Entity, With<Enemy>>,
    attributes_query: Query<(&AttributeData, &AttributeName, &AttributeOwner)>,
) {
    // Check player death
    if let Ok(player) = player_query.single() {
        let health = attributes_query
            .iter()
            .find(|(_, name, owner)| owner.0 == player && name.0 == "Health")
            .map(|(data, _, _)| data.current_value)
            .unwrap_or(0.0);

        if health <= 0.0 {
            combat_log
                .messages
                .push("=== GAME OVER - Hero has been defeated! ===".to_string());
            info!("=== GAME OVER - Hero has been defeated! ===");
            commands.entity(player).despawn();
        }
    }

    // Check enemy death
    if let Ok(enemy) = enemy_query.single() {
        let health = attributes_query
            .iter()
            .find(|(_, name, owner)| owner.0 == enemy && name.0 == "Health")
            .map(|(data, _, _)| data.current_value)
            .unwrap_or(0.0);

        if health <= 0.0 {
            combat_log
                .messages
                .push("=== VICTORY - Goblin has been defeated! ===".to_string());
            info!("=== VICTORY - Goblin has been defeated! ===");
            commands.entity(enemy).despawn();
        }
    }
}
