//! Example demonstrating AttributeBased magnitude calculations.
//!
//! This example shows how to create effects that scale based on the source or target's attributes.

use bevy::ecs::relationship::Relationship;
use bevy::prelude::*;
use bevy_gameplay_ability_system::{GasPlugin, attributes::*, effects::*};
use bevy_gameplay_tag::GameplayTagsPlugin;

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

// Define a simple attribute set
struct CombatAttributeSet;

impl AttributeSetDefinition for CombatAttributeSet {
    fn attribute_names() -> &'static [&'static str] {
        &["Health", "AttackPower"]
    }

    fn attribute_metadata(name: &str) -> Option<AttributeMetadata> {
        match name {
            "Health" => Some(
                AttributeMetadata::new("Health")
                    .with_min(0.0)
                    .with_max(100.0),
            ),
            "AttackPower" => Some(AttributeMetadata::new("AttackPower").with_min(0.0)),
            _ => None,
        }
    }

    fn default_value(name: &str) -> f32 {
        match name {
            "Health" => 100.0,
            "AttackPower" => 50.0,
            _ => 0.0,
        }
    }
}

#[derive(Component)]
struct HealTimer(Timer);

fn setup(mut commands: Commands, mut registry: ResMut<GameplayEffectRegistry>) {
    info!("Setting up AttributeBased magnitude example");

    // Create attacker with high attack power
    let attacker = commands.spawn_empty().id();
    CombatAttributeSet::create_attributes(&mut commands, attacker);

    // Create target with full health
    let target = commands.spawn_empty().id();
    CombatAttributeSet::create_attributes(&mut commands, target);

    // Register an effect that deals damage based on attacker's AttackPower
    // Damage = AttackPower * 0.5 (50% of attack power)
    let damage_effect = GameplayEffectDefinition::new("damage_from_attack_power")
        .with_duration_policy(DurationPolicy::Instant)
        .add_modifier(ModifierInfo::new(
            "Health",
            ModifierOperation::AddCurrent,
            MagnitudeCalculation::from_source_attribute("AttackPower", -0.5),
        ));

    registry.register(damage_effect);

    // Register an effect that heals based on target's max health
    // Heal = MaxHealth * 0.2 (20% of max health)
    let heal_effect = GameplayEffectDefinition::new("heal_percentage")
        .with_duration_policy(DurationPolicy::Instant)
        .add_modifier(ModifierInfo::new(
            "Health",
            ModifierOperation::AddCurrent,
            MagnitudeCalculation::from_target_attribute("Health", 0.2)
                .with_calculation_type(AttributeCalculationType::AttributeBaseValue),
        ));

    registry.register(heal_effect);

    // Apply damage effect (attacker -> target)
    commands.trigger(
        ApplyGameplayEffectEvent::new("damage_from_attack_power", target)
            .with_level(1)
            .with_instigator(attacker),
    );

    // Spawn timer for heal
    commands.spawn((
        HealTimer(Timer::from_seconds(0.1, TimerMode::Once)),
        HealTarget(target),
    ));

    info!("Attacker has 50 AttackPower, Target has 100 Health");
    info!("Damage should be: 50 * -0.5 = -25 (Target Health: 75)");
    info!("Heal should be: 100 * 0.2 = 20 (Target Health: 95)");
}

#[derive(Component)]
struct HealTarget(Entity);

fn check_results(
    mut commands: Commands,
    time: Res<Time>,
    mut timers: Query<(Entity, &mut HealTimer, &HealTarget)>,
    attributes: Query<(&AttributeData, &AttributeName, &ChildOf)>,
) {
    for (timer_entity, mut heal_timer, heal_target) in timers.iter_mut() {
        heal_timer.0.tick(time.delta());
        if heal_timer.0.just_finished() {
            // Apply heal effect
            commands.trigger(
                ApplyGameplayEffectEvent::new("heal_percentage", heal_target.0).with_level(1),
            );

            commands.entity(timer_entity).despawn();
        }
    }

    // Check if we should print results
    static mut PRINTED: bool = false;
    if time.elapsed_secs() > 0.3 && unsafe { !PRINTED } {
        unsafe { PRINTED = true };

        for (data, name, child_of) in attributes.iter() {
            if name.as_str() == "Health" {
                info!(
                    "Final Health for entity {:?}: {}",
                    child_of.get(),
                    data.current_value
                );
            }
        }

        info!("Example complete!");
    }
}
