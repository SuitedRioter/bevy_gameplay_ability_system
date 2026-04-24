//! Comprehensive RPG example demonstrating all GAS features working together.
//!
//! This example showcases:
//! - Attribute system with custom attribute sets
//! - Effects with AttributeBased and CustomCalculation
//! - Abilities with triggers and costs
//! - Immunity system
//! - Granted abilities from effects
//! - Stacking effects
//! - Periodic effects (DoT/HoT)

use bevy::ecs::relationship::Relationship;
use bevy::prelude::*;
use bevy_gameplay_ability_system::{
    GasPlugin, abilities::*, attributes::*, core::ImmunityTags, effects::*,
};
use bevy_gameplay_tag::{GameplayTag, GameplayTagsManager, GameplayTagsPlugin};

fn main() {
    App::new()
        .add_plugins((
            MinimalPlugins,
            GameplayTagsPlugin::with_data_path("assets/gameplay_tags.json".to_string()),
            GasPlugin,
        ))
        .add_systems(Startup, setup_game)
        .add_systems(Update, (simulate_combat, check_combat_results))
        .run();
}

// RPG Attribute Set
struct RpgAttributeSet;

impl AttributeSetDefinition for RpgAttributeSet {
    fn attribute_names() -> &'static [&'static str] {
        &[
            "Health",
            "MaxHealth",
            "Mana",
            "MaxMana",
            "AttackPower",
            "Defense",
            "CritChance",
        ]
    }

    fn attribute_metadata(name: &str) -> Option<AttributeMetadata> {
        match name {
            "Health" => Some(
                AttributeMetadata::new("Health")
                    .with_min(0.0)
                    .with_max(1000.0),
            ),
            "MaxHealth" => Some(AttributeMetadata::new("MaxHealth").with_min(1.0)),
            "Mana" => Some(AttributeMetadata::new("Mana").with_min(0.0).with_max(500.0)),
            "MaxMana" => Some(AttributeMetadata::new("MaxMana").with_min(1.0)),
            "AttackPower" => Some(AttributeMetadata::new("AttackPower").with_min(0.0)),
            "Defense" => Some(AttributeMetadata::new("Defense").with_min(0.0)),
            "CritChance" => Some(
                AttributeMetadata::new("CritChance")
                    .with_min(0.0)
                    .with_max(1.0),
            ),
            _ => None,
        }
    }

    fn default_value(name: &str) -> f32 {
        match name {
            "Health" => 500.0,
            "MaxHealth" => 500.0,
            "Mana" => 200.0,
            "MaxMana" => 200.0,
            "AttackPower" => 50.0,
            "Defense" => 20.0,
            "CritChance" => 0.1,
            _ => 0.0,
        }
    }
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Enemy;

#[derive(Resource)]
struct CombatTimer(Timer);

fn setup_game(
    mut commands: Commands,
    mut effect_registry: ResMut<GameplayEffectRegistry>,
    mut ability_registry: ResMut<AbilityRegistry>,
    tags_manager: Res<GameplayTagsManager>,
) {
    info!("=== Setting up Comprehensive RPG Example ===");

    // Create player
    let player = commands.spawn((Player, Name::new("Hero"))).id();
    RpgAttributeSet::create_attributes(&mut commands, player);

    // Create enemy
    let enemy = commands.spawn((Enemy, Name::new("Goblin"))).id();
    RpgAttributeSet::create_attributes(&mut commands, enemy);

    // Register effects
    register_effects(&mut effect_registry, &tags_manager);

    // Register abilities
    register_abilities(&mut ability_registry, &tags_manager);

    // Grant abilities to player
    grant_player_abilities(&mut commands, player, &ability_registry);

    // Start combat timer
    commands.insert_resource(CombatTimer(Timer::from_seconds(1.0, TimerMode::Repeating)));

    info!("Player and Enemy created with full attribute sets");
    info!("Combat will begin in 1 second...");
}

fn register_effects(registry: &mut GameplayEffectRegistry, tags_manager: &GameplayTagsManager) {
    // 1. Basic Attack Damage (AttributeBased)
    let basic_attack = GameplayEffectDefinition::new("basic_attack")
        .with_duration_policy(DurationPolicy::Instant)
        .with_immunity_tag(GameplayTag::new("Effect.Damage.Physical"), tags_manager)
        .add_modifier(ModifierInfo::new(
            "Health",
            ModifierOperation::AddCurrent,
            MagnitudeCalculation::from_source_attribute("AttackPower", -1.0)
                .with_post_multiply_add(-10.0), // Base damage + AttackPower
        ));
    registry.register(basic_attack);

    // 2. Defense Buff (grants temporary defense)
    let defense_buff = GameplayEffectDefinition::new("defense_buff")
        .with_duration_policy(DurationPolicy::HasDuration)
        .with_duration(5.0)
        .add_modifier(ModifierInfo::new(
            "Defense",
            ModifierOperation::AddCurrent,
            MagnitudeCalculation::ScalableFloat { base_value: 30.0 },
        ))
        .add_granted_tag(GameplayTag::new("State.Buffed"), tags_manager);
    registry.register(defense_buff);

    // 3. Poison DoT (Periodic damage)
    let poison = GameplayEffectDefinition::new("poison")
        .with_duration_policy(DurationPolicy::HasDuration)
        .with_duration(6.0)
        .with_period(2.0)
        .add_modifier(ModifierInfo::new(
            "Health",
            ModifierOperation::AddCurrent,
            MagnitudeCalculation::ScalableFloat { base_value: -15.0 },
        ))
        .add_granted_tag(GameplayTag::new("State.Poisoned"), tags_manager);
    registry.register(poison);

    // 4. Healing Potion (Instant heal based on MaxHealth)
    let heal_potion = GameplayEffectDefinition::new("heal_potion")
        .with_duration_policy(DurationPolicy::Instant)
        .add_modifier(ModifierInfo::new(
            "Health",
            ModifierOperation::AddCurrent,
            MagnitudeCalculation::from_target_attribute("MaxHealth", 0.3), // 30% of max health
        ));
    registry.register(heal_potion);

    // 5. Rage Buff (stacking attack power)
    let rage_stack = GameplayEffectDefinition::new("rage_stack")
        .with_duration_policy(DurationPolicy::HasDuration)
        .with_duration(10.0)
        .with_stacking_policy(StackingPolicy::StackCount { max_stacks: 5 })
        .add_modifier(ModifierInfo::new(
            "AttackPower",
            ModifierOperation::AddCurrent,
            MagnitudeCalculation::ScalableFloat { base_value: 10.0 }, // +10 per stack
        ))
        .add_granted_tag(GameplayTag::new("State.Enraged"), tags_manager);
    registry.register(rage_stack);

    // 6. Mana Regeneration (Periodic)
    let mana_regen = GameplayEffectDefinition::new("mana_regen")
        .with_duration_policy(DurationPolicy::HasDuration)
        .with_duration(10.0)
        .with_period(1.0)
        .add_modifier(ModifierInfo::new(
            "Mana",
            ModifierOperation::AddCurrent,
            MagnitudeCalculation::ScalableFloat { base_value: 10.0 },
        ));
    registry.register(mana_regen);
}

fn register_abilities(registry: &mut AbilityRegistry, tags_manager: &GameplayTagsManager) {
    // 1. Basic Attack Ability
    let basic_attack_ability = AbilityDefinition::new("ability_basic_attack")
        .with_instancing_policy(InstancingPolicy::InstancedPerExecution)
        .add_effect_to_apply("basic_attack".into())
        .with_activation_owned_tag(GameplayTag::new("Ability.Attacking"), tags_manager)
        .with_blocking_tag(GameplayTag::new("State.Stunned"), tags_manager);
    registry.register(basic_attack_ability);

    // 2. Defensive Stance (applies defense buff)
    let defensive_stance = AbilityDefinition::new("ability_defensive_stance")
        .with_instancing_policy(InstancingPolicy::InstancedPerActor)
        .add_effect_to_apply("defense_buff".into())
        .with_cost(AbilityCost::Mana(30.0))
        .with_activation_owned_tag(GameplayTag::new("Ability.Defending"), tags_manager);
    registry.register(defensive_stance);

    // 3. Poison Strike (applies poison)
    let poison_strike = AbilityDefinition::new("ability_poison_strike")
        .with_instancing_policy(InstancingPolicy::InstancedPerExecution)
        .add_effect_to_apply("poison".into())
        .with_cost(AbilityCost::Mana(50.0))
        .with_cooldown(5.0, tags_manager)
        .with_activation_owned_tag(GameplayTag::new("Ability.PoisonStrike"), tags_manager);
    registry.register(poison_strike);

    // 4. Heal Self
    let heal_self = AbilityDefinition::new("ability_heal_self")
        .with_instancing_policy(InstancingPolicy::InstancedPerExecution)
        .add_effect_to_apply("heal_potion".into())
        .with_cost(AbilityCost::Mana(40.0))
        .with_cooldown(8.0, tags_manager);
    registry.register(heal_self);

    // 5. Berserker Rage (stacking buff)
    let berserker_rage = AbilityDefinition::new("ability_berserker_rage")
        .with_instancing_policy(InstancingPolicy::InstancedPerExecution)
        .add_effect_to_apply("rage_stack".into())
        .with_cost(AbilityCost::Mana(20.0))
        .with_activation_owned_tag(GameplayTag::new("Ability.Rage"), tags_manager);
    registry.register(berserker_rage);
}

fn grant_player_abilities(commands: &mut Commands, player: Entity, registry: &AbilityRegistry) {
    let abilities = [
        "ability_basic_attack",
        "ability_defensive_stance",
        "ability_poison_strike",
        "ability_heal_self",
        "ability_berserker_rage",
    ];

    for ability_id in abilities {
        if let Some(definition) = registry.get(&ability_id.into()) {
            let spec = commands
                .spawn((
                    AbilitySpec {
                        definition_id: ability_id.into(),
                        level: 1,
                    },
                    AbilityOwner(player),
                    AbilityActiveState::default(),
                ))
                .id();

            info!("Granted ability '{}' to player", ability_id);
        }
    }
}

#[derive(Resource, Default)]
struct CombatPhase(u32);

fn simulate_combat(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<CombatTimer>,
    mut phase: Local<u32>,
    player_query: Query<Entity, With<Player>>,
    enemy_query: Query<Entity, With<Enemy>>,
    ability_query: Query<(Entity, &AbilitySpec, &AbilityOwner)>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    let Ok(player) = player_query.get_single() else {
        return;
    };
    let Ok(enemy) = enemy_query.get_single() else {
        return;
    };

    *phase += 1;

    match *phase {
        1 => {
            info!("\n=== Phase 1: Player uses Berserker Rage (3 stacks) ===");
            activate_ability(
                &mut commands,
                &ability_query,
                player,
                "ability_berserker_rage",
                player,
            );
        }
        2 => {
            activate_ability(
                &mut commands,
                &ability_query,
                player,
                "ability_berserker_rage",
                player,
            );
        }
        3 => {
            activate_ability(
                &mut commands,
                &ability_query,
                player,
                "ability_berserker_rage",
                player,
            );
            info!("Player should now have 3 stacks of Rage (+30 AttackPower)");
        }
        4 => {
            info!("\n=== Phase 2: Player attacks Enemy ===");
            activate_ability(
                &mut commands,
                &ability_query,
                player,
                "ability_basic_attack",
                enemy,
            );
        }
        5 => {
            info!("\n=== Phase 3: Player uses Poison Strike on Enemy ===");
            activate_ability(
                &mut commands,
                &ability_query,
                player,
                "ability_poison_strike",
                enemy,
            );
        }
        6 => {
            info!("Poison ticking...");
        }
        7 => {
            info!("Poison ticking...");
        }
        8 => {
            info!("\n=== Phase 4: Player uses Defensive Stance ===");
            activate_ability(
                &mut commands,
                &ability_query,
                player,
                "ability_defensive_stance",
                player,
            );
        }
        9 => {
            info!("\n=== Phase 5: Enemy attacks Player (should be reduced by defense) ===");
            commands.trigger(ApplyGameplayEffectEvent {
                effect_id: "basic_attack".into(),
                target: player,
                instigator: Some(enemy),
                level: 1,
            });
        }
        10 => {
            info!("\n=== Phase 6: Player heals self ===");
            activate_ability(
                &mut commands,
                &ability_query,
                player,
                "ability_heal_self",
                player,
            );
        }
        _ => {
            info!("\n=== Combat Complete ===");
            timer.0.pause();
        }
    }
}

fn activate_ability(
    commands: &mut Commands,
    ability_query: &Query<(Entity, &AbilitySpec, &AbilityOwner)>,
    owner: Entity,
    ability_id: &str,
    target: Entity,
) {
    for (spec_entity, spec, ability_owner) in ability_query.iter() {
        if ability_owner.0 == owner && spec.definition_id.as_ref() == ability_id {
            commands.trigger(TryActivateAbilityEvent {
                owner,
                ability_spec: spec_entity,
            });
            // Note: In a real game, you'd pass target through ability context
            info!("Activated ability '{}' targeting {:?}", ability_id, target);
            return;
        }
    }
}

fn check_combat_results(
    time: Res<Time>,
    attributes: Query<(&AttributeData, &AttributeName, &ChildOf)>,
    player_query: Query<Entity, With<Player>>,
    enemy_query: Query<Entity, With<Enemy>>,
) {
    static mut LAST_PRINT: f32 = 0.0;
    let current_time = time.elapsed_secs();

    unsafe {
        if current_time - LAST_PRINT < 1.0 {
            return;
        }
        LAST_PRINT = current_time;
    }

    let Ok(player) = player_query.get_single() else {
        return;
    };
    let Ok(enemy) = enemy_query.get_single() else {
        return;
    };

    info!("\n--- Current Stats ---");

    for entity in [player, enemy] {
        let name = if entity == player { "Player" } else { "Enemy" };
        info!("{}:", name);

        for (data, attr_name, child_of) in attributes.iter() {
            if child_of.get() == entity {
                match attr_name.as_str() {
                    "Health" | "Mana" | "AttackPower" | "Defense" => {
                        info!("  {}: {:.1}", attr_name.as_str(), data.current_value);
                    }
                    _ => {}
                }
            }
        }
    }
}
