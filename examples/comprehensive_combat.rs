//! Comprehensive combat example demonstrating all GAS features working together.
//!
//! This example showcases:
//! - AttributeBased magnitude calculations
//! - CustomCalculation for complex damage formulas
//! - Effect immunity system
//! - Ability triggers (passive abilities)
//! - Granted abilities from effects (equipment)
//! - Stacking effects
//! - Periodic effects (DoT/HoT)
//! - Cooldowns and costs

use bevy::ecs::relationship::Relationship;
use bevy::prelude::*;
use bevy_gameplay_ability_system::{GasPlugin, abilities::*, attributes::*, core::*, effects::*};
use bevy_gameplay_tag::{GameplayTagsManager, GameplayTagsPlugin, gameplay_tag::GameplayTag};
use string_cache::DefaultAtom as Atom;

fn main() {
    App::new()
        .add_plugins((
            MinimalPlugins,
            GameplayTagsPlugin::with_data_path("assets/gameplay_tags.json".to_string()),
            GasPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Startup, register_effects)
        .add_systems(Startup, register_abilities)
        .add_systems(Update, (simulate_combat, check_results))
        .run();
}

// Combat attribute set
struct CombatAttributes;

impl AttributeSetDefinition for CombatAttributes {
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
            "MaxHealth" => Some(
                AttributeMetadata::new("MaxHealth")
                    .with_min(1.0)
                    .with_max(1000.0),
            ),
            "Mana" => Some(AttributeMetadata::new("Mana").with_min(0.0).with_max(500.0)),
            "MaxMana" => Some(
                AttributeMetadata::new("MaxMana")
                    .with_min(1.0)
                    .with_max(500.0),
            ),
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

// Custom calculation for critical damage
struct CriticalDamageCalculator;

impl CustomMagnitudeCalculation for CriticalDamageCalculator {
    fn calculate(&self, ctx: &CalculationContext) -> f32 {
        let base_damage = ctx
            .get_source_attribute(&"AttackPower".into())
            .unwrap_or(10.0);
        let target_defense = ctx.get_target_attribute(&"Defense".into()).unwrap_or(0.0);
        let _crit_chance = ctx
            .get_source_attribute(&"CritChance".into())
            .unwrap_or(0.0);

        // Simple damage formula: (AttackPower - Defense/2) * CritMultiplier
        let damage = (base_damage - target_defense / 2.0).max(1.0);

        // 10% base crit chance + attribute crit chance
        // Simplified: use a deterministic pattern instead of random
        let is_crit = (base_damage as u32 % 5) == 0; // Every 5th attack crits
        if is_crit {
            info!("💥 Critical hit!");
            damage * 2.0
        } else {
            damage
        }
    }

    fn required_source_attributes(&self) -> &[&'static str] {
        &["AttackPower", "CritChance"]
    }

    fn required_target_attributes(&self) -> &[&'static str] {
        &["Defense"]
    }
}

#[derive(Component)]
struct CombatTimer(Timer);

fn setup(mut commands: Commands, mut custom_calc_registry: ResMut<CustomCalculationRegistry>) {
    info!("=== Setting up Comprehensive Combat Example ===\n");

    // Register custom calculation
    custom_calc_registry.register("CriticalDamage", Box::new(CriticalDamageCalculator));

    // === Create Player ===
    let player = commands.spawn_empty().id();
    CombatAttributes::create_attributes(&mut commands, player);
    commands
        .entity(player)
        .insert((OwnedTags::default(), Name::new("Player")));

    // === Create Enemy ===
    let enemy = commands.spawn_empty().id();
    CombatAttributes::create_attributes(&mut commands, enemy);
    commands
        .entity(enemy)
        .insert((OwnedTags::default(), Name::new("Enemy")));

    // === Apply Equipment Buff (grants temporary ability) ===
    commands
        .trigger(ApplyGameplayEffectEvent::new("equipment_sword_of_power", player).with_level(1));

    info!("\n=== Combat Starting ===");
    info!("Player: 500 HP, 50 Attack, 20 Defense");
    info!("Enemy: 500 HP, 50 Attack, 20 Defense");
    info!("Player equipped: Sword of Power (grants Whirlwind ability)\n");

    // Start combat timer
    commands.spawn(CombatTimer(Timer::from_seconds(0.5, TimerMode::Repeating)));
}

fn register_effects(
    mut registry: ResMut<GameplayEffectRegistry>,
    tags_manager: Res<GameplayTagsManager>,
) {
    // 1. Basic damage effect (uses custom critical calculation)
    let basic_damage = GameplayEffectDefinition::new("basic_damage")
        .with_duration_policy(DurationPolicy::Instant)
        .with_immunity_tag(GameplayTag::new("Effect.Damage.Physical"), &tags_manager)
        .add_modifier(ModifierInfo::new(
            "Health",
            ModifierOperation::AddCurrent,
            MagnitudeCalculation::custom("CriticalDamage"),
        ));
    registry.register(basic_damage);

    // 2. Power strike (AttributeBased: 150% of AttackPower)
    let power_strike = GameplayEffectDefinition::new("power_strike_damage")
        .with_duration_policy(DurationPolicy::Instant)
        .with_immunity_tag(GameplayTag::new("Effect.Damage.Physical"), &tags_manager)
        .add_modifier(ModifierInfo::new(
            "Health",
            ModifierOperation::AddCurrent,
            MagnitudeCalculation::from_source_attribute("AttackPower", -1.5),
        ));
    registry.register(power_strike);

    // 3. Heal effect (20% of MaxHealth)
    let heal = GameplayEffectDefinition::new("heal_effect")
        .with_duration_policy(DurationPolicy::Instant)
        .add_modifier(ModifierInfo::new(
            "Health",
            ModifierOperation::AddCurrent,
            MagnitudeCalculation::from_target_attribute("MaxHealth", 0.2)
                .with_calculation_type(AttributeCalculationType::AttributeBaseValue),
        ));
    registry.register(heal);

    // 4. Poison DoT (periodic damage)
    let poison = GameplayEffectDefinition::new("poison_dot")
        .with_duration_policy(DurationPolicy::HasDuration)
        .with_duration(5.0)
        .with_period(1.0)
        .with_immunity_tag(GameplayTag::new("Effect.Damage.Poison"), &tags_manager)
        .add_modifier(ModifierInfo::new(
            "Health",
            ModifierOperation::AddCurrent,
            MagnitudeCalculation::scalar(-10.0),
        ))
        .grant_tag(GameplayTag::new("State.Poisoned"), &tags_manager);
    registry.register(poison);

    // 5. Defense buff (stacking)
    let defense_buff = GameplayEffectDefinition::new("defense_buff")
        .with_duration_policy(DurationPolicy::HasDuration)
        .with_duration(10.0)
        .with_stacking_policy(StackingPolicy::StackCount { max_stacks: 3 })
        .add_modifier(ModifierInfo::new(
            "Defense",
            ModifierOperation::AddCurrent,
            MagnitudeCalculation::scalar(10.0),
        ))
        .grant_tag(GameplayTag::new("State.Buffed"), &tags_manager);
    registry.register(defense_buff);

    // 6. Equipment effect (grants Whirlwind ability)
    let equipment_sword = GameplayEffectDefinition::new("equipment_sword_of_power")
        .with_duration_policy(DurationPolicy::Infinite)
        .grant_ability(GrantedAbilityConfig {
            ability_id: "ability_whirlwind".into(),
            removal_policy: AbilityRemovalPolicy::CancelAbilityImmediately,
        })
        .add_modifier(ModifierInfo::new(
            "AttackPower",
            ModifierOperation::AddCurrent,
            MagnitudeCalculation::scalar(20.0),
        ));
    registry.register(equipment_sword);

    // 7. Mana cost effect
    let mana_cost = GameplayEffectDefinition::new("mana_cost_30")
        .with_duration_policy(DurationPolicy::Instant)
        .add_modifier(ModifierInfo::new(
            "Mana",
            ModifierOperation::AddCurrent,
            MagnitudeCalculation::scalar(-30.0),
        ));
    registry.register(mana_cost);

    // 8. Cooldown effect
    let cooldown = GameplayEffectDefinition::new("cooldown_5s")
        .with_duration_policy(DurationPolicy::HasDuration)
        .with_duration(5.0)
        .grant_tag(GameplayTag::new("Cooldown.PowerStrike"), &tags_manager);
    registry.register(cooldown);

    // 9. Whirlwind AOE damage
    let whirlwind_damage = GameplayEffectDefinition::new("whirlwind_damage")
        .with_duration_policy(DurationPolicy::Instant)
        .add_modifier(ModifierInfo::new(
            "Health",
            ModifierOperation::AddCurrent,
            MagnitudeCalculation::from_source_attribute("AttackPower", -2.0),
        ));
    registry.register(whirlwind_damage);

    // 10. Counter-attack damage
    let counter_damage = GameplayEffectDefinition::new("counter_attack_damage")
        .with_duration_policy(DurationPolicy::Instant)
        .add_modifier(ModifierInfo::new(
            "Health",
            ModifierOperation::AddCurrent,
            MagnitudeCalculation::from_source_attribute("AttackPower", -0.5),
        ));
    registry.register(counter_damage);
}

fn register_abilities(
    mut registry: ResMut<AbilityRegistry>,
    tags_manager: Res<GameplayTagsManager>,
) {
    // 1. Basic Attack
    let basic_attack = AbilityDefinition::new("ability_basic_attack")
        .with_instancing_policy(InstancingPolicy::InstancedPerExecution)
        .add_activation_owned_tag(GameplayTag::new("Ability.Attacking"), &tags_manager);
    registry.register(basic_attack);

    // 2. Power Strike (costs mana, has cooldown)
    let power_strike = AbilityDefinition::new("ability_power_strike")
        .with_instancing_policy(InstancingPolicy::InstancedPerExecution)
        .with_cost_effect(Atom::from("mana_cost_30"))
        .with_cooldown_effect(Atom::from("cooldown_5s"))
        .add_activation_owned_tag(GameplayTag::new("Ability.PowerStrike"), &tags_manager)
        .add_activation_blocked_tag(GameplayTag::new("Cooldown.PowerStrike"), &tags_manager);
    registry.register(power_strike);

    // 3. Heal
    let heal = AbilityDefinition::new("ability_heal")
        .with_instancing_policy(InstancingPolicy::InstancedPerExecution)
        .with_cost_effect(Atom::from("mana_cost_30"))
        .add_activation_owned_tag(GameplayTag::new("Ability.Healing"), &tags_manager);
    registry.register(heal);

    // 4. Whirlwind (granted by equipment)
    let whirlwind = AbilityDefinition::new("ability_whirlwind")
        .with_instancing_policy(InstancingPolicy::InstancedPerExecution)
        .with_cost_effect(Atom::from("mana_cost_30"))
        .add_activation_owned_tag(GameplayTag::new("Ability.Whirlwind"), &tags_manager);
    registry.register(whirlwind);

    // 5. Counter-attack (passive, triggered by damage)
    let counter = AbilityDefinition::new("ability_counter_attack")
        .with_instancing_policy(InstancingPolicy::InstancedPerExecution)
        .add_activation_owned_tag(GameplayTag::new("Ability.CounterAttack"), &tags_manager);
    registry.register(counter);
}

#[derive(Resource, Default)]
struct CombatState {
    turn: u32,
}

fn simulate_combat(
    mut commands: Commands,
    time: Res<Time>,
    mut timers: Query<(Entity, &mut CombatTimer)>,
    mut state: Local<CombatState>,
    players: Query<(Entity, &Name)>,
    attributes: Query<(&AttributeData, &AttributeName, &ChildOf)>,
) {
    let Ok((timer_entity, mut timer)) = timers.single_mut() else {
        return;
    };

    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }

    // Find player and enemy by name
    let mut player = None;
    let mut enemy = None;
    for (entity, name) in players.iter() {
        if name.as_str() == "Player" {
            player = Some(entity);
        } else if name.as_str() == "Enemy" {
            enemy = Some(entity);
        }
    }

    let Some(player) = player else { return };
    let Some(enemy) = enemy else { return };

    // Check if combat should end
    let player_health = get_attribute_value(&attributes, player, "Health");
    let enemy_health = get_attribute_value(&attributes, enemy, "Health");

    if player_health <= 0.0 || enemy_health <= 0.0 {
        return;
    }

    state.turn += 1;
    info!("\n--- Turn {} ---", state.turn);

    // Player's turn
    if state.turn == 1 {
        info!("Player uses Basic Attack on Enemy");
        commands.trigger(
            ApplyGameplayEffectEvent::new("basic_damage", enemy)
                .with_level(1)
                .with_instigator(player),
        );
        // Trigger damage event for counter-attack
        commands.trigger(GameplayEvent {
            event_tag: GameplayTag::new("Event.Damage.Received"),
            instigator: Some(player),
            target: Some(enemy),
            magnitude: Some(0.0),
            target_data: None,
        });
    } else if state.turn == 2 {
        info!("Player uses Power Strike on Enemy");
        // Note: This example doesn't properly set up abilities, just applies effects directly
        commands.trigger(
            ApplyGameplayEffectEvent::new("power_strike_damage", enemy)
                .with_level(1)
                .with_instigator(player),
        );
    } else if state.turn == 3 {
        info!("Player applies Poison to Enemy");
        commands.trigger(
            ApplyGameplayEffectEvent::new("poison_dot", enemy)
                .with_level(1)
                .with_instigator(player),
        );
    } else if state.turn == 4 {
        info!("Player stacks Defense Buff (1st stack)");
        commands.trigger(
            ApplyGameplayEffectEvent::new("defense_buff", player)
                .with_level(1)
                .with_instigator(player),
        );
    } else if state.turn == 5 {
        info!("Player stacks Defense Buff (2nd stack)");
        commands.trigger(
            ApplyGameplayEffectEvent::new("defense_buff", player)
                .with_level(1)
                .with_instigator(player),
        );
    } else if state.turn == 6 {
        info!("Player uses Heal");
        commands.trigger(
            ApplyGameplayEffectEvent::new("heal_effect", player)
                .with_level(1)
                .with_instigator(player),
        );
    } else if state.turn >= 7 {
        // End combat
        info!("\n=== Combat Ended ===");
        commands.entity(timer_entity).despawn();
    }
}

fn get_attribute_value(
    attributes: &Query<(&AttributeData, &AttributeName, &ChildOf)>,
    owner: Entity,
    name: &str,
) -> f32 {
    for (data, attr_name, child_of) in attributes.iter() {
        if child_of.get() == owner && attr_name.as_str() == name {
            return data.current_value;
        }
    }
    0.0
}

fn check_results(
    time: Res<Time>,
    attributes: Query<(&AttributeData, &AttributeName, &ChildOf)>,
    names: Query<&Name>,
) {
    static mut LAST_PRINT: f32 = 0.0;
    let elapsed = time.elapsed_secs();

    if elapsed - unsafe { LAST_PRINT } > 0.5 {
        unsafe { LAST_PRINT = elapsed };

        info!("\n--- Status Update ---");
        for (data, attr_name, child_of) in attributes.iter() {
            if (attr_name.as_str() == "Health"
                || attr_name.as_str() == "Mana"
                || attr_name.as_str() == "Defense")
                && let Ok(name) = names.get(child_of.get())
            {
                info!(
                    "{} {}: {:.1}",
                    name.as_str(),
                    attr_name.as_str(),
                    data.current_value
                );
            }
        }
    }
}
