//! Ability Task demonstration example.
//!
//! This example showcases various Task types:
//! - WaitDelayTask: Charged attack with 2s charge time
//! - WaitAttributeChangeTask: Auto-heal when health drops below 30%
//! - WaitInputPressTask: Channeled ability that can be cancelled
//! - WaitTargetDataTask: Area-of-effect ability requiring target selection
//! - WaitGameplayEventTask: Counter-attack triggered by taking damage

use bevy::prelude::*;
use bevy_gameplay_ability_system::{
    GasPlugin, abilities::*, attributes::*, core::*, effects::*, abilities::tasks::*,
};
use bevy_gameplay_tag::{GameplayTag, GameplayTagsManager, GameplayTagsPlugin};
use std::sync::Arc;

fn main() {
    App::new()
        .add_plugins((
            MinimalPlugins,
            GameplayTagsPlugin::with_data_path("assets/gameplay_tags.json".to_string()),
            GasPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (
            simulate_input,
            simulate_damage,
            log_task_completion,
        ))
        .run();
}

#[derive(Component)]
struct Player;

struct SimpleAttributeSet;

impl AttributeSetDefinition for SimpleAttributeSet {
    fn attribute_names() -> &'static [&'static str] {
        &["Health", "MaxHealth", "Mana", "MaxMana"]
    }

    fn attribute_metadata(name: &str) -> Option<AttributeMetadata> {
        match name {
            "Health" => Some(AttributeMetadata::new("Health").with_min(0.0).with_max(100.0)),
            "MaxHealth" => Some(AttributeMetadata::new("MaxHealth").with_min(1.0)),
            "Mana" => Some(AttributeMetadata::new("Mana").with_min(0.0).with_max(100.0)),
            "MaxMana" => Some(AttributeMetadata::new("MaxMana").with_min(1.0)),
            _ => None,
        }
    }

    fn default_value(name: &str) -> f32 {
        match name {
            "Health" | "MaxHealth" => 100.0,
            "Mana" | "MaxMana" => 100.0,
            _ => 0.0,
        }
    }
}

fn setup(
    mut commands: Commands,
    tags_manager: Res<GameplayTagsManager>,
    mut effect_registry: ResMut<GameplayEffectRegistry>,
    mut ability_registry: ResMut<AbilityRegistry>,
) {
    info!("=== Ability Task Demo ===\n");

    let player = commands.spawn((Player, Name::new("Player"))).id();
    SimpleAttributeSet::spawn_attributes(&mut commands, player);

    setup_effects(&mut effect_registry, &tags_manager);
    setup_abilities(&mut ability_registry, &tags_manager);
    grant_abilities(&mut commands, player, &ability_registry);

    info!("Player spawned with abilities:");
    info!("  1. Charged Attack (WaitDelayTask)");
    info!("  2. Auto Heal (WaitAttributeChangeTask)");
    info!("  3. Channeled Spell (WaitInputPressTask)");
    info!("  4. Area Blast (WaitTargetDataTask)");
    info!("  5. Counter Attack (WaitGameplayEventTask)\n");
}

fn setup_effects(registry: &mut GameplayEffectRegistry, tags_manager: &GameplayTagsManager) {
    let damage = GameplayEffectDefinition::new("damage_20")
        .with_duration_policy(DurationPolicy::Instant)
        .add_modifier(
            "Health",
            ModifierOperation::AddCurrent,
            MagnitudeCalculation::scalar(-20.0),
        );
    registry.register(damage);

    let heal = GameplayEffectDefinition::new("heal_30")
        .with_duration_policy(DurationPolicy::Instant)
        .add_modifier(
            "Health",
            ModifierOperation::AddCurrent,
            MagnitudeCalculation::scalar(30.0),
        );
    registry.register(heal);

    let mana_cost = GameplayEffectDefinition::new("mana_cost_20")
        .with_duration_policy(DurationPolicy::Instant)
        .add_modifier(
            "Mana",
            ModifierOperation::AddCurrent,
            MagnitudeCalculation::scalar(-20.0),
        );
    registry.register(mana_cost);
}

fn setup_abilities(registry: &mut AbilityRegistry, tags_manager: &GameplayTagsManager) {
    // 1. Charged Attack - uses WaitDelayTask
    let charged_attack = AbilityDefinition::new("ability_charged_attack")
        .with_instancing_policy(InstancingPolicy::InstancedPerExecution)
        .with_behavior(Arc::new(ChargedAttackBehavior {
            charge_time: 2.0,
            effect_id: "damage_20",
        }))
        .with_cost_effect("mana_cost_20");
    registry.register(charged_attack);

    // 2. Auto Heal - uses WaitAttributeChangeTask
    let auto_heal = AbilityDefinition::new("ability_auto_heal")
        .with_instancing_policy(InstancingPolicy::InstancedPerExecution)
        .with_behavior(Arc::new(AutoHealBehavior {
            threshold: 30.0,
            heal_effect_id: "heal_30",
        }));
    registry.register(auto_heal);

    // 3. Channeled Spell - uses WaitInputPressTask
    let channeled_spell = AbilityDefinition::new("ability_channeled_spell")
        .with_instancing_policy(InstancingPolicy::InstancedPerExecution)
        .with_behavior(Arc::new(ChanneledSpellBehavior {
            effect_id: "damage_20",
        }))
        .with_cost_effect("mana_cost_20");
    registry.register(channeled_spell);

    // 4. Area Blast - uses WaitTargetDataTask
    let area_blast = AbilityDefinition::new("ability_area_blast")
        .with_instancing_policy(InstancingPolicy::InstancedPerExecution)
        .with_behavior(Arc::new(AreaBlastBehavior {
            effect_id: "damage_20",
        }))
        .with_cost_effect("mana_cost_20");
    registry.register(area_blast);

    // 5. Counter Attack - uses WaitGameplayEventTask
    let counter_attack = AbilityDefinition::new("ability_counter_attack")
        .with_instancing_policy(InstancingPolicy::InstancedPerExecution)
        .with_behavior(Arc::new(CounterAttackBehavior {
            effect_id: "damage_20",
        }));
    registry.register(counter_attack);
}

fn grant_abilities(commands: &mut Commands, player: Entity, registry: &AbilityRegistry) {
    for ability_id in [
        "ability_charged_attack",
        "ability_auto_heal",
        "ability_channeled_spell",
        "ability_area_blast",
        "ability_counter_attack",
    ] {
        if registry.get(ability_id).is_some() {
            commands.spawn((
                AbilitySpec::new(ability_id, 1),
                AbilityOwner(player),
                AbilityActiveState::default(),
            ));
        }
    }
}

// === Ability Behaviors ===

struct ChargedAttackBehavior {
    charge_time: f32,
    effect_id: &'static str,
}

impl AbilityBehavior for ChargedAttackBehavior {
    fn on_activate(
        &self,
        commands: &mut Commands,
        ability_spec: Entity,
        instance: Entity,
        owner: Entity,
        _target: Entity,
        _level: i32,
        _tags_manager: &GameplayTagsManager,
    ) {
        info!("[ChargedAttack] Starting {} second charge...", self.charge_time);

        commands.spawn((
            WaitDelayTask::new(self.charge_time),
            TaskState::Running,
            AbilityTask {
                ability_instance: Some(instance),
                ability_spec,
                owner,
            },
            Name::new("ChargeTask"),
        )).set_parent(instance);
    }
}

struct AutoHealBehavior {
    threshold: f32,
    heal_effect_id: &'static str,
}

impl AbilityBehavior for AutoHealBehavior {
    fn on_activate(
        &self,
        commands: &mut Commands,
        ability_spec: Entity,
        instance: Entity,
        owner: Entity,
        _target: Entity,
        _level: i32,
        _tags_manager: &GameplayTagsManager,
    ) {
        info!("[AutoHeal] Monitoring health, will heal when < {}", self.threshold);

        commands.spawn((
            WaitAttributeChangeTask::new(
                "Health",
                AttributeComparison::LessThan,
                self.threshold,
            ),
            TaskState::Running,
            AbilityTask {
                ability_instance: Some(instance),
                ability_spec,
                owner,
            },
            Name::new("HealthMonitorTask"),
        )).set_parent(instance);
    }
}

struct ChanneledSpellBehavior {
    effect_id: &'static str,
}

impl AbilityBehavior for ChanneledSpellBehavior {
    fn on_activate(
        &self,
        commands: &mut Commands,
        ability_spec: Entity,
        instance: Entity,
        owner: Entity,
        _target: Entity,
        _level: i32,
        _tags_manager: &GameplayTagsManager,
    ) {
        info!("[ChanneledSpell] Channeling... Press Confirm to release or Cancel to abort");

        commands.spawn((
            WaitInputPressTask::confirm(),
            TaskState::Running,
            AbilityTask {
                ability_instance: Some(instance),
                ability_spec,
                owner,
            },
            Name::new("ChannelTask"),
        )).set_parent(instance);
    }
}

struct AreaBlastBehavior {
    effect_id: &'static str,
}

impl AbilityBehavior for AreaBlastBehavior {
    fn on_activate(
        &self,
        commands: &mut Commands,
        ability_spec: Entity,
        instance: Entity,
        owner: Entity,
        _target: Entity,
        _level: i32,
        _tags_manager: &GameplayTagsManager,
    ) {
        info!("[AreaBlast] Waiting for target selection...");

        commands.spawn((
            WaitTargetDataTask::new(),
            TaskState::Running,
            AbilityTask {
                ability_instance: Some(instance),
                ability_spec,
                owner,
            },
            Name::new("TargetSelectionTask"),
        )).set_parent(instance);
    }
}

struct CounterAttackBehavior {
    effect_id: &'static str,
}

impl AbilityBehavior for CounterAttackBehavior {
    fn on_activate(
        &self,
        commands: &mut Commands,
        ability_spec: Entity,
        instance: Entity,
        owner: Entity,
        _target: Entity,
        _level: i32,
        tags_manager: &GameplayTagsManager,
    ) {
        info!("[CounterAttack] Ready to counter on damage taken");

        commands.spawn((
            WaitGameplayEventTask::new(GameplayTag::new("Event.Damage.Taken")),
            TaskState::Running,
            AbilityTask {
                ability_instance: Some(instance),
                ability_spec,
                owner,
            },
            Name::new("DamageListenerTask"),
        )).set_parent(instance);
    }
}

// === Simulation Systems ===

fn simulate_input(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    player_query: Query<Entity, With<Player>>,
) {
    let Ok(player) = player_query.single() else { return };

    if keyboard.just_pressed(KeyCode::Space) {
        info!("\n[Input] Confirm pressed");
        commands.trigger(InputPressedEvent {
            owner: player,
            action: InputAction::Confirm,
        });
    }

    if keyboard.just_pressed(KeyCode::Escape) {
        info!("\n[Input] Cancel pressed");
        commands.trigger(InputPressedEvent {
            owner: player,
            action: InputAction::Cancel,
        });
    }
}

fn simulate_damage(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    player_query: Query<Entity, With<Player>>,
    tags_manager: Res<GameplayTagsManager>,
) {
    let Ok(player) = player_query.single() else { return };

    if keyboard.just_pressed(KeyCode::KeyD) {
        info!("\n[Simulation] Player takes damage!");
        commands.trigger(GameplayEvent {
            event_tag: GameplayTag::new("Event.Damage.Taken"),
            target: player,
            instigator: None,
            magnitude: 20.0,
        });
    }
}

fn log_task_completion(
    task_query: Query<(&Name, &TaskState), Changed<TaskState>>,
) {
    for (name, state) in task_query.iter() {
        if *state == TaskState::Completed {
            info!("[Task] {} completed!", name);
        }
    }
}
