//! Stress test example for performance testing.
//!
//! This example creates many entities with abilities and effects to test system performance.
//! Run with: cargo run --example stress_test --release

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_gameplay_ability_system::prelude::*;
use bevy_gameplay_tag::gameplay_tag::GameplayTag;
use bevy_gameplay_tag::{GameplayTagContainer, GameplayTagRequirements};
const NUM_ENTITIES: usize = 100;
const EFFECTS_PER_ENTITY: usize = 10;
const ABILITIES_PER_ENTITY: usize = 5;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GasPlugin)
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, stress_test_system)
        .add_systems(Update, stress_test_system)
        .run();
}

#[derive(Component)]
struct StressTestEntity {
    id: usize,
}

#[derive(Resource)]
struct StressTestConfig {
    spawn_timer: Timer,
    effect_timer: Timer,
    ability_timer: Timer,
    entities_spawned: usize,
}

impl Default for StressTestConfig {
    fn default() -> Self {
        Self {
            spawn_timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            effect_timer: Timer::from_seconds(0.5, TimerMode::Repeating),
            ability_timer: Timer::from_seconds(1.0, TimerMode::Repeating),
            entities_spawned: 0,
        }
    }
}

fn setup(
    mut commands: Commands,
    mut effect_registry: ResMut<GameplayEffectRegistry>,
    mut ability_registry: ResMut<AbilityRegistry>,
) {
    info!("=== Stress Test Starting ===");
    info!("Target: {} entities", NUM_ENTITIES);
    info!("Effects per entity: {}", EFFECTS_PER_ENTITY);
    info!("Abilities per entity: {}", ABILITIES_PER_ENTITY);

    commands.insert_resource(StressTestConfig::default());

    // Register test effects
    for i in 0..EFFECTS_PER_ENTITY {
        effect_registry.register(GameplayEffectDefinition {
            id: format!("TestEffect{}", i),
            duration_policy: if i % 3 == 0 {
                DurationPolicy::Instant
            } else if i % 3 == 1 {
                DurationPolicy::HasDuration
            } else {
                DurationPolicy::Infinite
            },
            duration_magnitude: 5.0,
            period: if i % 2 == 0 { 1.0 } else { 0.0 },
            modifiers: vec![ModifierInfo {
                attribute_name: "Health".to_string(),
                operation: match i % 5 {
                    0 => ModifierOperation::AddBase,
                    1 => ModifierOperation::AddCurrent,
                    2 => ModifierOperation::MultiplyAdditive,
                    3 => ModifierOperation::MultiplyMultiplicative,
                    _ => ModifierOperation::Override,
                },
                magnitude: MagnitudeCalculation::ScalableFloat { base_value: 10.0 },
            }],
            granted_tags: vec![GameplayTag::new(&format!("Effect.Test{}", i))],
            application_tag_requirements: GameplayTagRequirements::default(),
            stacking_policy: match i % 3 {
                0 => StackingPolicy::RefreshDuration,
                1 => StackingPolicy::StackCount { max_stacks: 5 },
                _ => StackingPolicy::Independent,
            },
        });
    }

    // Register test abilities
    for i in 0..ABILITIES_PER_ENTITY {
        ability_registry.register(AbilityDefinition {
            id: format!("TestAbility{}", i),
            instancing_policy: if i % 2 == 0 {
                InstancingPolicy::NonInstanced
            } else {
                InstancingPolicy::InstancedPerExecution
            },
            net_execution_policy: NetExecutionPolicy::LocalOnly,
            cost_effects: vec![],
            cooldown_effect: None,
            activation_owned_tags: GameplayTagContainer::default(),
            activation_required_tags: GameplayTagContainer::default(),
            activation_blocked_tags: GameplayTagContainer::default(),
            cancel_abilities_with_tags: GameplayTagContainer::default(),
            cancel_on_tags_added: GameplayTagContainer::default(),
        });
    }

    info!(
        "Registered {} effects and {} abilities",
        EFFECTS_PER_ENTITY, ABILITIES_PER_ENTITY
    );
}

fn stress_test_system(
    mut commands: Commands,
    mut config: ResMut<StressTestConfig>,
    time: Res<Time>,
    entities: Query<Entity, With<StressTestEntity>>,
    abilities: Query<Entity, With<AbilitySpec>>,
) {
    // Spawn entities gradually
    config.spawn_timer.tick(time.delta());
    if config.spawn_timer.just_finished() && config.entities_spawned < NUM_ENTITIES {
        let entity = commands
            .spawn(StressTestEntity {
                id: config.entities_spawned,
            })
            .id();

        // Create attributes
        commands.spawn((
            AttributeData::new(100.0),
            AttributeOwner(entity),
            AttributeName("Health".to_string()),
        ));

        commands.spawn((
            AttributeData::new(50.0),
            AttributeOwner(entity),
            AttributeName("Mana".to_string()),
        ));

        // Grant abilities
        for i in 0..ABILITIES_PER_ENTITY {
            commands.spawn((
                AbilitySpec {
                    definition_id: format!("TestAbility{}", i),
                    level: 1,
                    input_id: Some(i as i32),
                    is_active: false,
                },
                AbilityOwner(entity),
            ));
        }

        config.entities_spawned += 1;

        if config.entities_spawned % 10 == 0 {
            info!(
                "Spawned {} / {} entities",
                config.entities_spawned, NUM_ENTITIES
            );
        }
    }
}
