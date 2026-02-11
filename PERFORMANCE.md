# Performance Optimization Guide

This document provides performance optimization strategies and best practices for the Bevy Gameplay Ability System.

## Table of Contents

1. [Performance Benchmarks](#performance-benchmarks)
2. [Optimization Strategies](#optimization-strategies)
3. [Best Practices](#best-practices)
4. [Profiling](#profiling)
5. [Common Performance Pitfalls](#common-performance-pitfalls)

---

## Performance Benchmarks

### Running Benchmarks

```bash
cargo bench --bench gas_benchmarks
```

This will generate HTML reports in `target/criterion/` with detailed performance metrics.

### Benchmark Categories

1. **Attribute Modification** - Tests attribute value updates across multiple entities
2. **Effect Application** - Measures effect application overhead
3. **Duration Effect Updates** - Tests per-frame duration tracking
4. **Ability Activation Checks** - Measures activation requirement validation
5. **Tag Requirement Checks** - Tests tag matching performance
6. **Modifier Aggregation** - Measures attribute modifier calculation
7. **Combat Scenario** - Full integration test with 100 entities

---

## Optimization Strategies

### 1. Entity-Based Design Benefits

The system uses entity-based storage for abilities, effects, and attributes, which provides:

- **Cache Locality**: Components are stored contiguously in memory
- **Parallel Processing**: Bevy's ECS can run systems in parallel
- **Change Detection**: Only modified components trigger updates

### 2. Query Optimization

#### Use Specific Queries

```rust
// ❌ Bad: Queries all entities with AttributeData
fn system(query: Query<&AttributeData>) {
    for attr in query.iter() {
        // Process all attributes
    }
}

// ✅ Good: Queries only changed attributes
fn system(query: Query<&AttributeData, Changed<AttributeData>>) {
    for attr in query.iter() {
        // Only process changed attributes
    }
}
```

#### Filter Queries Appropriately

```rust
// ❌ Bad: Manual filtering in system
fn system(query: Query<(&AbilitySpec, &AbilityOwner)>) {
    for (spec, owner) in query.iter() {
        if owner.0 == target_entity {
            // Process
        }
    }
}

// ✅ Good: Use query filters
fn system(
    target: Entity,
    query: Query<&AbilitySpec, With<AbilityOwner>>
) {
    // Bevy filters at query time
}
```

### 3. Modifier Aggregation Optimization

The attribute aggregation system is designed for efficiency:

```rust
// Modifiers are applied in order:
// 1. AddBase - Direct base value modifications
// 2. AddCurrent - Flat bonuses to current value
// 3. MultiplyAdditive - Additive multipliers (10% + 15% = 25%)
// 4. MultiplyMultiplicative - Multiplicative multipliers (1.1 * 1.15 = 1.265)
// 5. Override - Final override (if present)
```

**Optimization Tips:**
- Use `AddBase` for permanent stat changes (level-ups, equipment)
- Use `AddCurrent` for temporary flat bonuses (buffs)
- Prefer `MultiplyAdditive` over `MultiplyMultiplicative` when possible
- Avoid excessive `Override` modifiers (they invalidate all other modifiers)

### 4. Effect Stacking Strategies

Choose the right stacking policy for your use case:

```rust
// ✅ RefreshDuration - Best for simple buffs/debuffs
StackingPolicy::RefreshDuration

// ✅ StackCount - Best for stackable effects with max limit
StackingPolicy::StackCount { max_stacks: 5 }

// ⚠️ Independent - Creates new entity per application (higher overhead)
StackingPolicy::Independent
```

### 5. Tag Requirement Optimization

Tag checks are performed frequently, so optimize them:

```rust
// ❌ Bad: Many individual tag requirements
TagRequirements::new()
    .require_tag(tag1)
    .require_tag(tag2)
    .require_tag(tag3)
    // Each tag checked individually

// ✅ Good: Use require_all_tags for AND logic
TagRequirements::new()
    .require_all_tags(vec![tag1, tag2, tag3])
    // Single hierarchical check

// ✅ Good: Use require_any_tags for OR logic
TagRequirements::new()
    .require_any_tags(vec![tag1, tag2, tag3])
    // Early exit on first match
```

### 6. Periodic Effect Optimization

For periodic effects, balance frequency vs. overhead:

```rust
// ❌ Bad: Very frequent ticks (high overhead)
period: 0.1, // Every 0.1 seconds

// ✅ Good: Reasonable tick rate
period: 1.0, // Every 1 second

// ✅ Better: Use instant effects for immediate damage
duration_policy: DurationPolicy::Instant
```

### 7. GameplayCue Batching

The cue system supports batching for performance:

```rust
// Cues triggered in the same frame are automatically batched
// when using the same cue tag

// ✅ Good: Use hierarchical tags for batching
GameplayTag::new("GameplayCue.Damage.Physical")
GameplayTag::new("GameplayCue.Damage.Magical")
// Both match "GameplayCue.Damage" and can be batched
```

---

## Best Practices

### 1. Minimize Effect Count

```rust
// ❌ Bad: Multiple effects for related stats
registry.register(GameplayEffectDefinition {
    id: "StrengthBuff",
    modifiers: vec![
        ModifierInfo {
            attribute_name: "Strength".to_string(),
            operation: ModifierOperation::AddBase,
            magnitude: MagnitudeCalculation::ScalableFloat { value: 10.0 },
        },
    ],
    // ...
});
registry.register(GameplayEffectDefinition {
    id: "AttackBuff",
    modifiers: vec![
        ModifierInfo {
            attribute_name: "Attack".to_string(),
            operation: ModifierOperation::AddBase,
            magnitude: MagnitudeCalculation::ScalableFloat { value: 5.0 },
        },
    ],
    // ...
});

// ✅ Good: Combine into single effect
registry.register(GameplayEffectDefinition {
    id: "CombatBuff",
    modifiers: vec![
        ModifierInfo {
            attribute_name: "Strength".to_string(),
            operation: ModifierOperation::AddBase,
            magnitude: MagnitudeCalculation::ScalableFloat { value: 10.0 },
        },
        ModifierInfo {
            attribute_name: "Attack".to_string(),
            operation: ModifierOperation::AddBase,
            magnitude: MagnitudeCalculation::ScalableFloat { value: 5.0 },
        },
    ],
    // ...
});
```

### 2. Use Appropriate Duration Policies

```rust
// ✅ Instant - For immediate one-time effects (damage, healing)
DurationPolicy::Instant

// ✅ HasDuration - For temporary buffs/debuffs
DurationPolicy::HasDuration

// ✅ Infinite - For permanent effects (equipment, passive abilities)
DurationPolicy::Infinite
```

### 3. Limit Active Effects

Monitor and limit the number of active effects per entity:

```rust
fn limit_active_effects(
    query: Query<(Entity, &EffectTarget)>,
    effects: Query<&ActiveGameplayEffect>,
    mut commands: Commands,
) {
    const MAX_EFFECTS_PER_ENTITY: usize = 50;

    // Count effects per entity
    let mut effect_counts: HashMap<Entity, Vec<Entity>> = HashMap::new();

    for (effect_entity, target) in query.iter() {
        effect_counts.entry(target.0)
            .or_default()
            .push(effect_entity);
    }

    // Remove oldest effects if over limit
    for (target, mut effect_list) in effect_counts {
        if effect_list.len() > MAX_EFFECTS_PER_ENTITY {
            effect_list.sort_by_key(|e| {
                effects.get(*e).map(|eff| eff.start_time).unwrap_or(0.0)
            });

            for effect_entity in effect_list.iter().take(effect_list.len() - MAX_EFFECTS_PER_ENTITY) {
                commands.entity(*effect_entity).despawn();
            }
        }
    }
}
```

### 4. Optimize Attribute Sets

Keep attribute sets focused and minimal:

```rust
// ❌ Bad: Too many attributes
pub struct CharacterAttributes {
    // 50+ attributes
}

// ✅ Good: Focused attribute sets
pub struct CombatAttributes {
    // Health, Attack, Defense, etc. (5-10 attributes)
}

pub struct ResourceAttributes {
    // Mana, Stamina, Energy (3-5 attributes)
}
```

### 5. Use Handles for Long-Term References

```rust
// ✅ Good: Store handles, not entities directly
#[derive(Component)]
pub struct PlayerAbilities {
    pub primary_attack: AbilityHandle,
    pub special_ability: AbilityHandle,
}

// Handles include generation counters to detect stale references
if handle.is_valid(&world) {
    // Safe to use
}
```

---

## Profiling

### Using Bevy's Built-in Profiling

```rust
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};

app.add_plugins(FrameTimeDiagnosticsPlugin)
   .add_plugins(LogDiagnosticsPlugin::default());
```

### Using Tracy Profiler

Add to `Cargo.toml`:

```toml
[dependencies]
bevy = { version = "0.18.0", features = ["trace_tracy"] }
```

Run with:

```bash
cargo run --release --features bevy/trace_tracy
```

### Key Metrics to Monitor

1. **Frame Time** - Should stay under 16.67ms for 60 FPS
2. **System Execution Time** - Identify slow systems
3. **Entity Count** - Monitor active effects and abilities
4. **Memory Usage** - Watch for memory leaks

---

## Common Performance Pitfalls

### 1. Excessive Tag Checks

```rust
// ❌ Bad: Checking tags every frame for all entities
fn system(query: Query<(&AbilitySystemComponent, Entity)>) {
    for (asc, entity) in query.iter() {
        if asc.owned_tags.has_matching_gameplay_tag(&some_tag) {
            // Do something
        }
    }
}

// ✅ Good: Use change detection
fn system(query: Query<(&AbilitySystemComponent, Entity), Changed<AbilitySystemComponent>>) {
    // Only runs when tags change
}
```

### 2. Unnecessary Effect Applications

```rust
// ❌ Bad: Applying effects every frame
fn system(mut events: EventWriter<ApplyGameplayEffectEvent>) {
    events.send(ApplyGameplayEffectEvent { /* ... */ });
}

// ✅ Good: Apply effects only when needed
fn system(
    mut events: EventWriter<ApplyGameplayEffectEvent>,
    trigger: Res<SomeTrigger>,
) {
    if trigger.is_changed() {
        events.send(ApplyGameplayEffectEvent { /* ... */ });
    }
}
```

### 3. Over-Aggregation

```rust
// ❌ Bad: Aggregating all attributes every frame
fn aggregate_all_attributes(
    mut attributes: Query<&mut AttributeData>,
    modifiers: Query<&AttributeModifier>,
) {
    // Runs for all attributes every frame
}

// ✅ Good: Only aggregate when modifiers change
fn aggregate_attributes(
    mut attributes: Query<&mut AttributeData>,
    modifiers: Query<&AttributeModifier, Or<(Added<AttributeModifier>, Changed<AttributeModifier>)>>,
) {
    // Only runs when modifiers are added or changed
}
```

### 4. Inefficient Ability Lookups

```rust
// ❌ Bad: Linear search through all abilities
fn find_ability(
    ability_id: &str,
    query: Query<(Entity, &AbilitySpec)>,
) -> Option<Entity> {
    query.iter()
        .find(|(_, spec)| spec.definition_id == ability_id)
        .map(|(entity, _)| entity)
}

// ✅ Good: Use helper functions with optimized queries
use bevy_gameplay_ability_system::utils::find_ability_by_definition;

let ability = find_ability_by_definition(owner, "FireballAbility", &query);
```

### 5. Spawning Too Many Entities

```rust
// ❌ Bad: Creating entities for every modifier
for modifier in modifiers {
    commands.spawn((
        AttributeModifier { /* ... */ },
        ModifierSource(effect_entity),
    ));
}

// ✅ Good: Store modifiers in effect definition
GameplayEffectDefinition {
    modifiers: vec![/* all modifiers */],
    // Modifiers are applied without spawning entities
}
```

---

## Performance Targets

### Recommended Limits

- **Entities with GAS components**: 1000+
- **Active effects per entity**: 20-50
- **Abilities per entity**: 10-20
- **Attributes per entity**: 10-30
- **Modifiers per attribute**: 20-50
- **Tag checks per frame**: 100-500

### Expected Performance

Based on benchmarks (results will vary by hardware):

- **Attribute modification**: <1μs per entity
- **Effect application**: <10μs per effect
- **Duration updates**: <1μs per effect
- **Ability activation check**: <5μs per ability
- **Tag requirement check**: <100ns per tag
- **Modifier aggregation**: <2μs per attribute

---

## Optimization Checklist

- [ ] Use change detection in queries
- [ ] Minimize active effect count
- [ ] Choose appropriate stacking policies
- [ ] Batch GameplayCue triggers
- [ ] Use hierarchical tags for matching
- [ ] Limit periodic effect frequency
- [ ] Profile with Tracy or Bevy diagnostics
- [ ] Monitor frame time and entity count
- [ ] Use handles for long-term references
- [ ] Combine related effects into single definitions
- [ ] Use appropriate duration policies
- [ ] Optimize tag requirement checks
- [ ] Avoid unnecessary effect applications
- [ ] Use query filters instead of manual filtering

---

## Conclusion

The Bevy Gameplay Ability System is designed for performance with entity-based storage, efficient queries, and change detection. Follow these optimization strategies and best practices to maintain high performance even with hundreds of entities using the system.

For specific performance issues, run the benchmarks and use profiling tools to identify bottlenecks.
