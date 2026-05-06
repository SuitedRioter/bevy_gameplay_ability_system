# Bevy Gameplay Ability System

A comprehensive gameplay ability system for Bevy, inspired by Unreal Engine's GameplayAbilitySystem (GAS). This library provides a flexible and powerful framework for implementing RPG-style abilities, attributes, and effects using pure ECS architecture.

[![Crates.io](https://img.shields.io/crates/v/bevy_gameplay_ability_system.svg)](https://crates.io/crates/bevy_gameplay_ability_system)
[![Docs](https://docs.rs/bevy_gameplay_ability_system/badge.svg)](https://docs.rs/bevy_gameplay_ability_system)
[![License](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/yourusername/bevy_gameplay_ability_system#license)

## Features

- **Attribute System**: Define custom attribute sets with 6 lifecycle hooks (matching UE's AttributeSet callbacks)
- **Gameplay Effects**: Modify attributes with instant, duration, or infinite effects
  - 10 evaluation channels for complex stacking rules
  - Periodic execution for damage/healing over time
  - Custom magnitude calculations and execution calculations
  - GameplayEffect components for modular behavior extension
- **Gameplay Abilities**: Implement abilities with costs, cooldowns, and activation requirements
  - 3 instancing policies (NonInstanced, InstancedPerActor, InstancedPerExecution)
  - 12 built-in ability task types for async operations
  - Tag-based activation requirements, blocking, and cancellation
- **Gameplay Cues**: Visual and audio feedback system with hierarchical tag matching
  - Static cues (lightweight, no entity) and Actor cues (spawned entities)
  - Specialized cue types (Burst, Looping, HitImpact)
- **Tag-based System**: Uses `bevy_gameplay_tag` for flexible hierarchical tag matching
- **Pure ECS Architecture**: Fully leverages Bevy's ECS for performance and flexibility
- **Entity-based Design**: Attributes, effects, and abilities are all separate entities
- **Bevy 0.18 Integration**: Uses ChildOf relationships for automatic cleanup

## Documentation

- [API Documentation](https://docs.rs/bevy_gameplay_ability_system)
- [Complete RPG Example](examples/complete_rpg.rs) - Full combat system demonstration
- [Stress Test Example](examples/stress_test.rs) - Performance testing tool

## Bevy Compatibility

| Bevy Version | Plugin Version |
| ------------ | -------------- |
| 0.18         | 0.1            |

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
bevy = "0.18.1"
bevy_gameplay_ability_system = "0.1"
bevy_gameplay_tag = "0.2.0"
```

## Quick Start

```rust
use bevy::prelude::*;
use bevy_gameplay_ability_system::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GasPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    // Create an entity with attributes
    let player = commands.spawn_empty().id();

    // Create attributes using a custom attribute set
    CharacterAttributes::create_attributes(&mut commands, player);
}

// Define your custom attribute set
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
}
```

## Architecture

The system is built on four core modules:

### 1. Attributes

The attribute system provides a dual-value model (BaseValue/CurrentValue) with automatic modifier aggregation and 6 lifecycle hooks.

- **BaseValue**: Permanent value, modified by instant effects
- **CurrentValue**: Temporary value, modified by duration/infinite effects
- **Modifiers**: Applied in order: Add → Multiply → Override
- **Lifecycle Hooks**: 6 hooks matching UE's AttributeSet callbacks
  - `pre_effect_execute` / `post_effect_execute`: Instant effect application
  - `pre_attribute_change` / `post_attribute_change`: Current value changes
  - `pre_attribute_base_change` / `post_attribute_base_change`: Base value changes

```rust
// Attributes are entities with components
#[derive(Component)]
pub struct AttributeData {
    pub base_value: f32,
    pub current_value: f32,
}

// Each attribute is linked to its owner via ChildOf relationship
commands.spawn((
    AttributeData { base_value: 100.0, current_value: 100.0 },
    AttributeName("Health".into()),
    AttributeSetId(TypeId::of::<CharacterAttributes>()),
)).set_parent_in_place(owner_entity);
```

### 2. Gameplay Effects

Effects modify attributes and can be instant, duration-based, or infinite.

```rust
// Create an effect definition
let damage_effect = GameplayEffectDefinition::new("effect.damage.fire")
    .with_duration_policy(DurationPolicy::Instant)
    .add_modifier(ModifierInfo {
        attribute_name: "Health".to_string(),
        operation: ModifierOperation::AddBase,
        magnitude: MagnitudeCalculation::ScalableFloat { base_value: -20.0 },
    });

// Apply the effect to a target
commands.spawn((
    ActiveGameplayEffect {
        definition_id: "effect.damage.fire".to_string(),
        level: 1,
        start_time: 0.0,
        stack_count: 1,
    },
    EffectTarget(target_entity),
));
```

**Effect Features:**

- Duration policies: Instant, HasDuration, Infinite
- Periodic execution (damage/healing over time)
- Stacking policies: Independent, RefreshDuration, StackCount
- Tag requirements for application
- Granted tags while active
- 10 evaluation channels (Channel0-Channel9) for complex stacking rules
- GameplayEffect components for modular behavior extension
- Custom magnitude calculations and execution calculations

### 3. Gameplay Abilities

Abilities are player-activated actions with costs, cooldowns, and requirements.

```rust
// Define an ability (tag methods require &Res<GameplayTagsManager>)
let fireball = AbilityDefinition::new("ability.fireball")
    .with_instancing_policy(InstancingPolicy::InstancedPerExecution)
    .add_activation_required_tag(GameplayTag::new("State.Alive"), &tags_manager)
    .add_activation_blocked_tag(GameplayTag::new("State.Stunned"), &tags_manager)
    .with_cost_effect("effect.cost.mana")
    .with_cooldown_effect("effect.cooldown.fireball");

// Grant the ability to an entity
commands.spawn((
    AbilitySpec {
        definition_id: "ability.fireball".to_string(),
        level: 1,
        input_id: None,
        is_active: false,
    },
    AbilityOwner(player_entity),
));
```

**Ability Features:**

- Instancing policies: NonInstanced, InstancedPerActor, InstancedPerExecution
- Cost effects (mana, stamina, etc.)
- Cooldown effects (tag-based)
- Tag requirements and blocking
- Activation events
- 12 built-in ability task types:
  - WaitDelayTask, WaitGameplayEventTask, WaitTargetDataTask
  - WaitAttributeChangeTask, WaitGameplayEffectAppliedTask, WaitGameplayEffectRemovedTask
  - ApplyEffectToTargetDataTask, ApplyRootMotionTask, PlayMontageTask
  - SpawnProjectileTask, RepeatTask, WaitInputPressTask, WaitInputReleaseTask

### 4. Gameplay Cues

Cues provide visual and audio feedback for gameplay events.

```rust
// Register a static cue
let mut cue_manager = world.resource_mut::<GameplayCueManager>();
cue_manager.register_static_cue(GameplayTag::new("GameplayCue.Damage.Fire"));

// Trigger a cue
commands.trigger(TriggerGameplayCueEvent {
    cue_tag: GameplayTag::new("GameplayCue.Damage.Fire"),
    event_type: GameplayCueEvent::Executed,
    parameters: GameplayCueParameters::default(),
});
```

**Cue Features:**

- Static cues (lightweight, no entity)
- Actor cues (spawned entities with lifetime)
- Specialized cue types: BurstCue, LoopingCue, HitImpactCue
- Hierarchical tag matching
- Batching for performance
- Event types: OnActive, WhileActive, Executed, Removed

## Core Concepts

### Entity-Based Design

Unlike traditional component-based approaches, this system uses entities for attributes, abilities, and effects:

```rust
// Each attribute is an entity linked via ChildOf
let attribute = commands.spawn((
    AttributeData { base_value: 100.0, current_value: 100.0 },
    AttributeName("Health".into()),
)).set_parent_in_place(owner).id();

// Each active effect is an entity
let effect_entity = commands.spawn((
    ActiveGameplayEffect { /* ... */ },
    EffectTarget(target),
    EffectDuration { remaining: 5.0, total: 5.0 },
)).id();

// Each granted ability is an entity
let ability_entity = commands.spawn((
    AbilitySpec { /* ... */ },
    AbilityOwner(owner),
)).id();
```

**Benefits:**

- Better ECS performance (query optimization)
- Parallel system execution
- Automatic cleanup via ChildOf relationships (Bevy 0.18)
- Memory locality
- Easier to extend with custom components

### Tag Requirements

Tag requirements control when effects and abilities can be applied/activated:

```rust
use bevy_gameplay_tag::GameplayTagRequirements;

let mut requirements = GameplayTagRequirements::new();

// Must have these tags
requirements.require_tags.add_tag(
    GameplayTag::new("Ability.Skill"),
    &tags_manager
);

// Must NOT have these tags
requirements.ignore_tags.add_tag(
    GameplayTag::new("Status.Debuff.Silence"),
    &tags_manager
);

// Check if requirements are met
if requirements.requirements_met(&entity_tags) {
    println!("Can use ability!");
}
```

### Modifier Operations

Effects can modify attributes in different ways:

- **AddBase**: Permanently adds to base value (instant effects)
- **AddCurrent**: Temporarily adds to current value (duration effects)
- **MultiplyAdditive**: Multiplies with additive stacking (1 + sum of multipliers)
- **MultiplyMultiplicative**: Multiplies with multiplicative stacking (product of multipliers)
- **Override**: Sets the current value directly

### System Ordering

The plugin configures proper system ordering for deterministic execution:

```rust
GasSystemSet::Input
  → GasSystemSet::Attributes
  → GasSystemSet::Effects
  → GasSystemSet::Abilities
  → GasSystemSet::Cues
  → GasSystemSet::Cleanup
```

## Examples

### Basic Attributes

See `examples/basic_attributes.rs` for a complete example of:

- Defining custom attribute sets
- Creating attributes for entities
- Modifying attribute values
- Querying attributes

Run with:

```bash
cargo run --example basic_attributes
```

### Gameplay Effects

See `examples/gameplay_effects.rs` for examples of:

- Instant effects (damage, healing)
- Duration effects (buffs, debuffs)
- Periodic effects (damage over time)
- Effect stacking

Run with:

```bash
cargo run --example gameplay_effects
```

### Ability Activation

See `examples/ability_activation.rs` for examples of:

- Granting abilities to entities
- Activating abilities with costs
- Cooldown management
- Tag-based requirements

Run with:

```bash
cargo run --example ability_activation
```

### Complete RPG Example

See `examples/complete_rpg.rs` for a full combat simulation featuring:

- Player vs Enemy combat
- Multiple abilities (attacks, spells, healing)
- AI-controlled enemy
- Combat log and death detection
- All four core systems working together

Run with:

```bash
cargo run --example complete_rpg
```

## Utility Functions

The library includes helpful utilities:

### Math Utilities

```rust
use bevy_gameplay_ability_system::utils::*;

// Clamping with optional bounds
let clamped = clamp_optional(value, Some(0.0), Some(100.0));

// Linear interpolation
let interpolated = lerp(0.0, 100.0, 0.5); // 50.0

// Remap value from one range to another
let remapped = remap(50.0, 0.0, 100.0, 0.0, 1.0); // 0.5

// Smooth interpolation
let smooth = smoothstep(0.0, 1.0, 0.5);

// Percentage calculation
let percent = percentage(75.0, 100.0); // 75.0
```

### Query Helpers

```rust
use bevy_gameplay_ability_system::utils::*;

// Find attribute by name
if let Some((entity, data)) = find_attribute_by_name(owner, "Health", &query) {
    println!("Health: {}", data.current_value);
}

// Get all attributes for an owner
let attributes = get_owner_attributes(owner, &query);

// Get active effects on target
let effects = get_active_effects_on_target(target, &query);

// Find abilities by definition
if let Some((entity, spec)) = find_ability_by_definition(owner, "ability.fireball", &query) {
    println!("Fireball level: {}", spec.level);
}
```

## Best Practices

### 1. Use the Built-in Registries

The library provides `GameplayEffectRegistry` and `AbilityRegistry` for storing definitions:

```rust
// Register effect definitions
let mut effect_registry = world.resource_mut::<GameplayEffectRegistry>();
effect_registry.register(damage_effect);

// Register ability definitions
let mut ability_registry = world.resource_mut::<AbilityRegistry>();
ability_registry.register(fireball_ability);
```

### 2. Use Tag Hierarchies

Organize tags hierarchically for flexible matching:

```rust
GameplayTag::new("State.Alive")
GameplayTag::new("State.Dead")
GameplayTag::new("Effect.Buff.Attack")
GameplayTag::new("Effect.Debuff.Stun")
GameplayTag::new("Cooldown.Attack")
GameplayTag::new("Cooldown.Spell")
```

### 3. Separate Cost and Cooldown Effects

Create reusable cost and cooldown effects:

```rust
// Cost effects
let mana_cost = GameplayEffectDefinition::new("effect.cost.mana")
    .with_duration_policy(DurationPolicy::Instant)
    .add_modifier(/* subtract mana */);

// Cooldown effects (grant_tag requires &tags_manager)
let spell_cooldown = GameplayEffectDefinition::new("effect.cooldown.spell")
    .with_duration_policy(DurationPolicy::HasDuration)
    .with_duration(3.0)
    .grant_tag(GameplayTag::new("Cooldown.Spell"), &tags_manager);
```

### 4. Use Handles for External References

When storing references outside the ECS, use handles:

```rust
let handle = AbilityHandle::new(ability_entity, generation);

// Later, check if still valid
if handle.is_valid(&world) {
    // Use the handle
}
```

### 5. Leverage System Sets

Add your custom systems to the appropriate set:

```rust
app.add_systems(
    Update,
    my_custom_ability_logic.in_set(GasSystemSet::Abilities)
);
```

## Performance Considerations

- **Entity-based design** enables parallel system execution
- **Query optimization** through Bevy's archetype system
- **Change detection** minimizes unnecessary updates
- **Cue batching** reduces overhead for visual effects
- **ChildOf relationships** provide automatic cleanup (Bevy 0.18)
- **Interned strings** (string_cache::Atom) for efficient lookups

## Project Status

✅ **Production Ready for Single-Player Games** - All core systems complete with comprehensive test coverage.

### Test Coverage

**Total: 127/127 tests passing (100% pass rate) ✅**

- Unit tests: 41/41 passed ✅
- Integration tests: 81/81 passed ✅
  - `ability_granting_lifecycle_test`: 1 test
  - `ability_task_test`: 12 tests (all task types)
  - `application_requirement_test`: 2 tests (custom requirements)
  - `attribute_aggregation_test`: 2 tests
  - `enhanced_requirements_test`: 4 tests (percent-based, source vs target, tags, level range)
  - `evaluation_channel_test`: 3 tests (channel evaluation order, same-channel combination, complex stacking)
  - `gameplay_effect_spec_test`: 2 tests
  - `instancing_policy_test`: 3 tests (NonInstanced, InstancedPerActor, InstancedPerExecution)
  - `periodic_effect_spec_test`: 2 tests
  - `stack_count_test`: 2 tests
  - `stacking_reapply_spec_test`: 2 tests
  - Additional tests for specialized cues, input tasks, dynamic magnitudes, etc.
- Doc tests: 5/5 passed ✅

### What's Complete

- ✅ Attribute system with 6 lifecycle hooks (matching UE's AttributeSet callbacks)
- ✅ Gameplay effects (instant, duration, infinite, periodic)
- ✅ Effect stacking policies (Independent, RefreshDuration, StackCount)
- ✅ 10 evaluation channels for complex stacking rules
- ✅ GameplayEffect components for modular behavior extension
- ✅ Ability activation with 3 instancing policies
- ✅ 12 ability task types for async operations
- ✅ Gameplay cues with specialized types (Burst, Looping, HitImpact)
- ✅ Tag-based requirements and blocking
- ✅ Custom application requirements and magnitude calculations
- ✅ SetByCaller magnitudes with spec persistence

### Known Limitations

- Single-player only (no networking/replication)
- Performance optimization deferred (current design handles <50 entities with <10 attributes each)
- Benchmark suite broken for Bevy 0.18 (criterion compatibility issue)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Acknowledgments

This library is inspired by Unreal Engine's GameplayAbilitySystem (GAS) and adapted for Bevy's ECS architecture. Special thanks to:

- Epic Games for the original GAS design
- The Bevy community for the excellent game engine
- Contributors to `bevy_gameplay_tag`

## Resources

- [Bevy Engine](https://bevyengine.org/)
- [Unreal Engine GAS Documentation](https://docs.unrealengine.com/en-US/gameplay-ability-system-for-unreal-engine/)
- [bevy_gameplay_tag](https://github.com/SuitedRioter/bevy_gameplay_tag)
