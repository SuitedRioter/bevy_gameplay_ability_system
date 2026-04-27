//! Comprehensive RPG example demonstrating multiple GAS features working together.
//!
//! This example showcases:
//! - Custom attribute sets
//! - Instant, duration, periodic, and stacking effects
//! - Abilities with custom behaviors
//! - Cost and cooldown effects
//! - Immunity-tag aware effect definitions

use bevy::ecs::relationship::Relationship;
use bevy::prelude::*;
use bevy_gameplay_ability_system::{GasPlugin, abilities::*, attributes::*, core::*, effects::*};
use bevy_gameplay_tag::{GameplayTag, GameplayTagsManager, GameplayTagsPlugin};
use std::{collections::HashMap, sync::Arc};

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

#[derive(Resource, Default)]
struct AbilityTargets(HashMap<Entity, Entity>);

struct ApplyEffectBehavior {
    effect_id: &'static str,
}

impl AbilityBehavior for ApplyEffectBehavior {
    fn activate(
        &self,
        commands: &mut Commands,
        _instance_entity: Entity,
        spec_entity: Entity,
        source: Entity,
        _target: Option<Entity>,
    ) {
        let effect_id = self.effect_id;
        commands.queue(move |world: &mut World| {
            let target = world
                .resource::<AbilityTargets>()
                .0
                .get(&spec_entity)
                .copied()
                .unwrap_or(source);
            let level = world
                .get::<AbilitySpec>(spec_entity)
                .map(|spec| spec.level)
                .unwrap_or(1);

            world.trigger(ApplyGameplayEffectEvent {
                effect_id: effect_id.into(),
                target,
                instigator: Some(source),
                level,
            });
        });
    }
}

fn setup_game(
    mut commands: Commands,
    mut effect_registry: ResMut<GameplayEffectRegistry>,
    mut ability_registry: ResMut<AbilityRegistry>,
    tags_manager: Res<GameplayTagsManager>,
) {
    info!("=== Setting up Comprehensive RPG Example ===");

    let player = commands
        .spawn((
            Player,
            Name::new("Hero"),
            OwnedTags::default(),
            BlockedAbilityTags::default(),
            ImmunityTags::default(),
        ))
        .id();
    RpgAttributeSet::create_attributes(&mut commands, player);

    let enemy = commands
        .spawn((
            Enemy,
            Name::new("Goblin"),
            OwnedTags::default(),
            BlockedAbilityTags::default(),
            ImmunityTags::default(),
        ))
        .id();
    RpgAttributeSet::create_attributes(&mut commands, enemy);

    register_effects(&mut effect_registry, &tags_manager);
    register_abilities(&mut ability_registry, &tags_manager);
    grant_player_abilities(&mut commands, player, &ability_registry);

    commands.insert_resource(AbilityTargets::default());
    commands.insert_resource(CombatTimer(Timer::from_seconds(1.0, TimerMode::Repeating)));

    info!("Player and Enemy created with full attribute sets");
    info!("Combat will begin in 1 second...");
}

fn register_effects(
    registry: &mut GameplayEffectRegistry,
    tags_manager: &Res<GameplayTagsManager>,
) {
    let basic_attack = GameplayEffectDefinition::new("basic_attack")
        .with_duration_policy(DurationPolicy::Instant)
        .add_modifier(ModifierInfo::new(
            "Health",
            ModifierOperation::AddCurrent,
            MagnitudeCalculation::from_source_attribute("AttackPower", -1.0)
                .with_post_multiply_add(-10.0),
        ));
    registry.register(basic_attack);

    let defense_buff = GameplayEffectDefinition::new("defense_buff")
        .with_duration(5.0)
        .add_modifier(ModifierInfo::new(
            "Defense",
            ModifierOperation::AddCurrent,
            MagnitudeCalculation::scalar(30.0),
        ))
        .grant_tag(GameplayTag::new("State.Buffed"), tags_manager);
    registry.register(defense_buff);

    let poison = GameplayEffectDefinition::new("poison")
        .with_duration(6.0)
        .with_period(2.0)
        .with_immunity_tag(GameplayTag::new("Effect.Debuff.Poison"), tags_manager)
        .add_modifier(ModifierInfo::new(
            "Health",
            ModifierOperation::AddCurrent,
            MagnitudeCalculation::scalar(-15.0),
        ))
        .grant_tag(GameplayTag::new("Effect.Debuff.Poison"), tags_manager);
    registry.register(poison);

    let heal_potion = GameplayEffectDefinition::new("heal_potion")
        .with_duration_policy(DurationPolicy::Instant)
        .add_modifier(ModifierInfo::new(
            "Health",
            ModifierOperation::AddCurrent,
            MagnitudeCalculation::from_target_attribute("MaxHealth", 0.3),
        ));
    registry.register(heal_potion);

    let rage_stack = GameplayEffectDefinition::new("rage_stack")
        .with_duration(10.0)
        .with_stacking_policy(StackingPolicy::StackCount { max_stacks: 5 })
        .add_modifier(ModifierInfo::new(
            "AttackPower",
            ModifierOperation::AddCurrent,
            MagnitudeCalculation::scalar(10.0),
        ))
        .grant_tag(GameplayTag::new("Effect.Buff.Attack"), tags_manager);
    registry.register(rage_stack);

    let mana_regen = GameplayEffectDefinition::new("mana_regen")
        .with_duration(10.0)
        .with_period(1.0)
        .add_modifier(ModifierInfo::new(
            "Mana",
            ModifierOperation::AddCurrent,
            MagnitudeCalculation::scalar(10.0),
        ))
        .grant_tag(GameplayTag::new("Effect.HealOverTime"), tags_manager);
    registry.register(mana_regen);

    let mana_cost_20 = GameplayEffectDefinition::new("mana_cost_20")
        .with_duration_policy(DurationPolicy::Instant)
        .add_modifier(ModifierInfo::new(
            "Mana",
            ModifierOperation::AddCurrent,
            MagnitudeCalculation::scalar(-20.0),
        ));
    registry.register(mana_cost_20);

    let mana_cost_30 = GameplayEffectDefinition::new("mana_cost_30")
        .with_duration_policy(DurationPolicy::Instant)
        .add_modifier(ModifierInfo::new(
            "Mana",
            ModifierOperation::AddCurrent,
            MagnitudeCalculation::scalar(-30.0),
        ));
    registry.register(mana_cost_30);

    let mana_cost_40 = GameplayEffectDefinition::new("mana_cost_40")
        .with_duration_policy(DurationPolicy::Instant)
        .add_modifier(ModifierInfo::new(
            "Mana",
            ModifierOperation::AddCurrent,
            MagnitudeCalculation::scalar(-40.0),
        ));
    registry.register(mana_cost_40);

    let mana_cost_50 = GameplayEffectDefinition::new("mana_cost_50")
        .with_duration_policy(DurationPolicy::Instant)
        .add_modifier(ModifierInfo::new(
            "Mana",
            ModifierOperation::AddCurrent,
            MagnitudeCalculation::scalar(-50.0),
        ));
    registry.register(mana_cost_50);

    let cooldown_spell_5s = GameplayEffectDefinition::new("cooldown_spell_5s")
        .with_duration(5.0)
        .grant_tag(GameplayTag::new("Cooldown.Spell"), tags_manager);
    registry.register(cooldown_spell_5s);

    let cooldown_heal_8s = GameplayEffectDefinition::new("cooldown_heal_8s")
        .with_duration(8.0)
        .grant_tag(GameplayTag::new("Cooldown.Heal"), tags_manager);
    registry.register(cooldown_heal_8s);
}

fn register_abilities(registry: &mut AbilityRegistry, tags_manager: &Res<GameplayTagsManager>) {
    let basic_attack_ability = AbilityDefinition::new("ability_basic_attack")
        .with_instancing_policy(InstancingPolicy::InstancedPerExecution)
        .with_behavior(Arc::new(ApplyEffectBehavior {
            effect_id: "basic_attack",
        }))
        .add_activation_owned_tag(GameplayTag::new("Ability.Attacking"), tags_manager)
        .add_source_blocked_tag(GameplayTag::new("State.Stunned"), tags_manager);
    registry.register(basic_attack_ability);

    let defensive_stance = AbilityDefinition::new("ability_defensive_stance")
        .with_instancing_policy(InstancingPolicy::InstancedPerActor)
        .with_behavior(Arc::new(ApplyEffectBehavior {
            effect_id: "defense_buff",
        }))
        .with_cost_effect("mana_cost_30")
        .add_activation_owned_tag(GameplayTag::new("Ability.Blocking"), tags_manager)
        .add_source_blocked_tag(GameplayTag::new("State.Stunned"), tags_manager);
    registry.register(defensive_stance);

    let poison_strike = AbilityDefinition::new("ability_poison_strike")
        .with_instancing_policy(InstancingPolicy::InstancedPerExecution)
        .with_behavior(Arc::new(ApplyEffectBehavior {
            effect_id: "poison",
        }))
        .with_cost_effect("mana_cost_50")
        .with_cooldown_effect("cooldown_spell_5s")
        .add_activation_owned_tag(GameplayTag::new("Ability.Casting"), tags_manager)
        .add_source_blocked_tag(GameplayTag::new("State.Stunned"), tags_manager);
    registry.register(poison_strike);

    let heal_self = AbilityDefinition::new("ability_heal_self")
        .with_instancing_policy(InstancingPolicy::InstancedPerExecution)
        .with_behavior(Arc::new(ApplyEffectBehavior {
            effect_id: "heal_potion",
        }))
        .with_cost_effect("mana_cost_40")
        .with_cooldown_effect("cooldown_heal_8s");
    registry.register(heal_self);

    let berserker_rage = AbilityDefinition::new("ability_berserker_rage")
        .with_instancing_policy(InstancingPolicy::InstancedPerExecution)
        .with_behavior(Arc::new(ApplyEffectBehavior {
            effect_id: "rage_stack",
        }))
        .with_cost_effect("mana_cost_20")
        .add_activation_owned_tag(GameplayTag::new("Ability.Attacking"), tags_manager)
        .add_source_blocked_tag(GameplayTag::new("State.Stunned"), tags_manager);
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
        if registry.get(ability_id).is_some() {
            commands.spawn((
                AbilitySpec::new(ability_id, 1),
                AbilityOwner(player),
                AbilityActiveState::default(),
            ));

            info!("Granted ability '{}' to player", ability_id);
        }
    }
}

fn simulate_combat(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<CombatTimer>,
    mut phase: Local<u32>,
    mut ability_targets: ResMut<AbilityTargets>,
    player_query: Query<Entity, With<Player>>,
    enemy_query: Query<Entity, With<Enemy>>,
    ability_query: Query<(Entity, &AbilitySpec, &AbilityOwner)>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    let Ok(player) = player_query.single() else {
        return;
    };
    let Ok(enemy) = enemy_query.single() else {
        return;
    };

    *phase += 1;

    match *phase {
        1 => {
            info!("\n=== Phase 1: Player uses Berserker Rage (3 stacks) ===");
            activate_ability(
                &mut commands,
                &mut ability_targets,
                &ability_query,
                player,
                "ability_berserker_rage",
                player,
            );
        }
        2 => {
            activate_ability(
                &mut commands,
                &mut ability_targets,
                &ability_query,
                player,
                "ability_berserker_rage",
                player,
            );
        }
        3 => {
            activate_ability(
                &mut commands,
                &mut ability_targets,
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
                &mut ability_targets,
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
                &mut ability_targets,
                &ability_query,
                player,
                "ability_poison_strike",
                enemy,
            );
        }
        6 | 7 => {
            info!("Poison ticking...");
        }
        8 => {
            info!("\n=== Phase 4: Player uses Defensive Stance ===");
            activate_ability(
                &mut commands,
                &mut ability_targets,
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
                &mut ability_targets,
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
    ability_targets: &mut AbilityTargets,
    ability_query: &Query<(Entity, &AbilitySpec, &AbilityOwner)>,
    owner: Entity,
    ability_id: &str,
    target: Entity,
) {
    for (spec_entity, spec, ability_owner) in ability_query.iter() {
        if ability_owner.0 == owner && spec.definition_id.as_ref() == ability_id {
            ability_targets.0.insert(spec_entity, target);
            commands.trigger(TryActivateAbilityEvent {
                owner,
                ability_spec: spec_entity,
            });
            info!("Activated ability '{}' targeting {:?}", ability_id, target);
            return;
        }
    }
}

fn check_combat_results(
    mut last_print: Local<f32>,
    time: Res<Time>,
    attributes: Query<(&AttributeData, &AttributeName, &ChildOf)>,
    player_query: Query<Entity, With<Player>>,
    enemy_query: Query<Entity, With<Enemy>>,
) {
    let current_time = time.elapsed_secs();
    if current_time - *last_print < 1.0 {
        return;
    }
    *last_print = current_time;

    let Ok(player) = player_query.single() else {
        return;
    };
    let Ok(enemy) = enemy_query.single() else {
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
