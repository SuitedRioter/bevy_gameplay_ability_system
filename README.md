# Bevy Gameplay Ability System

A comprehensive gameplay ability system for Bevy, inspired by Unreal Engine's GameplayAbilitySystem (GAS). This library provides a flexible and powerful framework for implementing RPG-style abilities, attributes, and effects using pure ECS architecture.

[![Crates.io](https://img.shields.io/crates/v/bevy_gameplay_ability_system.svg)](https://crates.io/crates/bevy_gameplay_ability_system)
[![Docs](https://docs.rs/bevy_gameplay_ability_system/badge.svg)](https://docs.rs/bevy_gameplay_ability_system)
[![License](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/yourusername/bevy_gameplay_ability_system#license)

## Features

- **Attribute System**: Define custom attribute sets with constraints (min/max values)
- **Gameplay Effects**: Modify attributes with instant, duration, or infinite effects
- **Gameplay Abilities**: Implement abilities with costs, cooldowns, and activation requirements
- **Gameplay Cues**: Visual and audio feedback system for gameplay events
- **Tag-based System**: Uses `bevy_gameplay_tag` for flexible tag matching and requirements
- **Pure ECS Architecture**: Fully leverages Bevy's ECS for performance and flexibility
- **Entity-based Design**: Effects and abilities are entities, not stored in vectors
- **Handle System**: Safe references with generation counters to prevent dangling references
- **Performance Optimized**: Benchmarked and optimized for 1000+ entities with abilities

## Documentation

- [API Documentation](https://docs.rs/bevy_gameplay_ability_system)
- [Performance Guide](PERFORMANCE.md) - Optimization strategies and best practices
- [Complete RPG Example](examples/complete_rpg.rs) - Full combat system demonstration
- [Stress Test Example](examples/stress_test.rs) - Performance testing tool
- **Performance Optimized**: Benchmarked and optimized for 1000+ entities with abilities

## Documentation

- [API Documentation](https://docs.rs/bevy_gameplay_ability_system)
- [Performance Guide](PERFORMANCE.md) - Optimization strategies and best practices
- [Complete RPG Example](examples/complete_rpg.rs) - Full combat system demonstration
- [Stress Test Example](examples/stress_test.rs) - Performance testing tool

## Bevy Compatibility

| Bevy Version | Plugin Version |
|--------------|----------------|
| 0.18         | 0.1            |

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
bevy = "0.18"
bevy_gameplay_ability_system = "0.1"
bevy_gameplay_tag = { git = "https://github.com/SuitedRioter/bevy_gameplay_tag.git", branch = "main" }
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

The attribute system provides a dual-value model (BaseValue/CurrentValue) with automatic modifier aggregation.

- **BaseValue**: Permanent value, modified by instant effects
- **CurrentValue**: Temporary value, modified by duration/infinite effects
- **Modifiers**: Applied in order: Add → Multiply → Override

```rust
// Attributes are entities with components
#[derive(Component)]
pub struct AttributeData {
    pub base_value: f32,
    pub current_value: f32,
}

// Each attribute has metadata
pub struct AttributeMetadata {
    pub name: &'static str,
    pub min_value: Option<f32>,
    pub max_value: Option<f32>,
}
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

### 3. Gameplay Abilities

Abilities are player-activated actions with costs, cooldowns, and requirements.

```rust
// Define an ability
let fireball = AbilityDefinition::new("ability.fireball")
    .with_instancing_policy(InstancingPolicy::InstancedPerExecution)
    .add_activation_required_tag(GameplayTag::new("State.Alive"))
    .add_activation_blocked_tag(GameplayTag::new("State.Stunned"))
    .add_cost_effect("effect.cost.mana".to_string())
    .with_cooldown_effect("effect.cooldown.fireball".to_string());

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

### 4. Gameplay Cues

Cues provide visual and audio feedback for gameplay events.

```rust
// Register a static cue
let mut cue_manager = world.resource_mut::<GameplayCueManager>();
cue_manager.register_static_cue(
    GameplayTag::new("GameplayCue.Damage.Fire"),
    Box::new(FireDamageCue),
);

// Trigger a cue
commands.trigger(TriggerGameplayCueEvent {
    cue_tag: GameplayTag::new("GameplayCue.Damage.Fire"),
    target: target_entity,
    parameters: GameplayCueParameters::default(),
});
```

**Cue Features:**
- Static cues (lightweight, no entity)
- Actor cues (spawned entities with lifetime)
- Hierarchical tag matching
- Batching for performance
- Event types: OnActive, WhileActive, Executed, Removed

## Core Concepts

### Entity-Based Design

Unlike traditional component-based approaches, this system uses entities for abilities and effects:

```rust
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
- Memory locality
- Easier to extend with custom components

### Tag Requirements

Tag requirements control when effects and abilities can be applied/activated:

```rust
use bevy_gameplay_ability_system::utils::TagRequirements;

let requirements = TagRequirements::new()
    .require_tag(GameplayTag::new("State.Alive"))
    .ignore_tag(GameplayTag::new("State.Stunned"))
    .require_all_tag(GameplayTag::new("Ability.CanCast"));

if requirements.are_requirements_met(&entity_tags) {
    // Apply effect or activate ability
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

### 1. Define Effect and Ability Registries

Store your definitions in resources for easy access:

```rust
#[derive(Resource)]
struct EffectRegistry {
    definitions: HashMap<String, GameplayEffectDefinition>,
}

#[derive(Resource)]
struct AbilityRegistry {
    definitions: HashMap<String, AbilityDefinition>,
}
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

// Cooldown effects
let spell_cooldown = GameplayEffectDefinition::new("effect.cooldown.spell")
    .with_duration_policy(DurationPolicy::HasDuration)
    .with_duration(3.0)
    .grant_tag(GameplayTag::new("Cooldown.Spell"));
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
- **Handle system** prevents expensive entity lookups

## Roadmap

- [ ] Attribute curves for level scaling
- [ ] Ability tasks for complex multi-frame abilities
- [ ] Prediction and rollback for networking
- [ ] Visual editor integration
- [ ] More built-in cue implementations
- [ ] Performance profiling and optimization

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
