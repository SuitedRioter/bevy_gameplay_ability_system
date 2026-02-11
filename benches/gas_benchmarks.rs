//! Performance benchmarks for the Gameplay Ability System.
//!
//! Run with: cargo bench
//!
//! NOTE: Many benchmarks are disabled due to Bevy 0.18 API changes.
//! Bevy 0.18 removed `run_system_once` which was heavily used in these benchmarks.
//! These benchmarks need to be rewritten to use proper system scheduling.

use bevy::prelude::*;
use bevy_gameplay_ability_system::prelude::*;
use bevy_gameplay_tag::gameplay_tag_count_container::GameplayTagCountContainer;
use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};

// Helper to create a test app with GAS plugin
fn create_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(GasPlugin);
    app
}

// Helper to create an entity with attribute set
fn create_entity_with_attributes(app: &mut App) -> Entity {
    let entity = app.world_mut().spawn_empty().id();

    // Create basic attributes
    app.world_mut().spawn((
        AttributeData::new(100.0),
        AttributeOwner(entity),
        AttributeName("Health".to_string()),
    ));

    app.world_mut().spawn((
        AttributeData::new(50.0),
        AttributeOwner(entity),
        AttributeName("Mana".to_string()),
    ));

    app.world_mut().spawn(GameplayTagCountContainer::default());

    app.update();
    entity
}

// Benchmark: Attribute modification
fn bench_attribute_modification(c: &mut Criterion) {
    let mut group = c.benchmark_group("attribute_modification");

    for num_entities in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_entities),
            num_entities,
            |b, &num_entities| {
                let mut app = create_test_app();

                // Create entities with attributes
                let _entities: Vec<Entity> = (0..num_entities)
                    .map(|_| create_entity_with_attributes(&mut app))
                    .collect();

                b.iter(|| {
                    // Modify all attributes via direct world access
                    let mut query = app.world_mut().query::<&mut AttributeData>();
                    for mut attr in query.iter_mut(app.world_mut()) {
                        attr.base_value = black_box(attr.base_value + 1.0);
                    }
                    app.update();
                });
            },
        );
    }

    group.finish();
}

// Benchmark: Duration effect updates
fn bench_duration_effect_updates(c: &mut Criterion) {
    let mut group = c.benchmark_group("duration_effect_updates");

    for num_effects in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_effects),
            num_effects,
            |b, &num_effects| {
                let mut app = create_test_app();
                let target = create_entity_with_attributes(&mut app);

                // Create duration effects
                for _ in 0..num_effects {
                    app.world_mut().spawn((
                        ActiveGameplayEffect {
                            definition_id: "TestEffect".to_string(),
                            level: 1,
                            start_time: 0.0,
                            stack_count: 1,
                        },
                        EffectTarget(target),
                        EffectDuration {
                            remaining: 10.0,
                            total: 10.0,
                        },
                    ));
                }
                app.update();

                b.iter(|| {
                    app.update();
                });
            },
        );
    }

    group.finish();
}

// Benchmark: Ability spec creation
fn bench_ability_spec_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("ability_spec_creation");

    for num_abilities in [5, 10, 20, 50].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_abilities),
            num_abilities,
            |b, &num_abilities| {
                let mut app = create_test_app();
                let owner = create_entity_with_attributes(&mut app);

                // Register abilities
                {
                    let mut registry = app.world_mut().resource_mut::<AbilityRegistry>();
                    for i in 0..num_abilities {
                        let ability_id = format!("Ability{}", i);
                        registry.register(AbilityDefinition {
                            id: ability_id.clone(),
                            instancing_policy: InstancingPolicy::NonInstanced,
                            net_execution_policy: NetExecutionPolicy::LocalOnly,
                            cost_effects: vec![],
                            cooldown_effect: None,
                            activation_owned_tags: vec![],
                            activation_required_tags: vec![],
                            activation_blocked_tags: vec![],
                            cancel_abilities_with_tags: vec![],
                            cancel_on_tags_added: vec![],
                        });
                    }
                }

                b.iter(|| {
                    // Create ability specs
                    for i in 0..num_abilities {
                        let ability_id = format!("Ability{}", i);
                        app.world_mut().spawn((
                            AbilitySpec {
                                definition_id: ability_id,
                                level: 1,
                                input_id: Some(i as i32),
                                is_active: false,
                            },
                            AbilityOwner(owner),
                            AbilityState::Ready,
                        ));
                    }
                    app.update();
                });
            },
        );
    }

    group.finish();
}

// Benchmark: Attribute aggregation
fn bench_attribute_aggregation(c: &mut Criterion) {
    let mut group = c.benchmark_group("attribute_aggregation");

    for num_modifiers in [5, 10, 20, 50].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_modifiers),
            num_modifiers,
            |b, &num_modifiers| {
                let mut app = create_test_app();
                let target = create_entity_with_attributes(&mut app);

                // Create modifiers
                for i in 0..num_modifiers {
                    let effect_entity = app
                        .world_mut()
                        .spawn((
                            ActiveGameplayEffect {
                                definition_id: format!("Effect{}", i),
                                level: 1,
                                start_time: 0.0,
                                stack_count: 1,
                            },
                            EffectTarget(target),
                        ))
                        .id();

                    app.world_mut().spawn((
                        AttributeModifier {
                            target_entity: target,
                            target_attribute: "Health".to_string(),
                            operation: ModifierOperation::AddCurrent,
                            magnitude: 10.0,
                        },
                        ModifierSource(effect_entity),
                    ));
                }
                app.update();

                b.iter(|| {
                    app.update();
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_attribute_modification,
    bench_duration_effect_updates,
    bench_ability_spec_creation,
    bench_attribute_aggregation,
);
criterion_main!(benches);
