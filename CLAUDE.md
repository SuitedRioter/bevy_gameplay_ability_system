# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A Bevy plugin implementing Unreal Engine's GameplayAbilitySystem (GAS) in pure ECS architecture. Targets Bevy 0.18 and single-player games. Uses `bevy_gameplay_tag` (git dependency) for hierarchical tag matching and `string_cache` for interned strings.

## Build & Test Commands

```bash
cargo build                              # Build the library
cargo test                               # Run all tests
cargo test --lib                         # Run unit tests only
cargo test <test_name>                   # Run a single test
cargo run --example basic_attributes     # Run an example
cargo run --example complete_rpg         # Full combat simulation
cargo bench                              # Run benchmarks (criterion)
```

## Architecture

Four core modules, each following the same internal structure (`components.rs`, `definition.rs`, `plugin.rs`, `systems.rs`):

**Attributes** (`src/attributes/`) — Dual-value model (BaseValue/CurrentValue). Each attribute is a separate entity linked to its owner via `AttributeOwner`. Custom attribute sets implement the `AttributeSetDefinition` trait. Modifiers applied in order: Add → Multiply → Override.

**Effects** (`src/effects/`) — Modify attributes via `GameplayEffectDefinition` templates stored in `GameplayEffectRegistry`. Each active effect is its own entity with `ActiveGameplayEffect` + `EffectTarget` components. Supports three duration policies (Instant, HasDuration, Infinite), periodic execution, and stacking (Independent, RefreshDuration, StackCount). Tag requirements gate application.

**Abilities** (`src/abilities/`) — Activated actions defined via `AbilityDefinition` templates in `AbilityRegistry`. Each granted ability is an entity with `AbilitySpec` + `AbilityOwner`. Activation flow: TryActivate → Commit (costs/cooldowns) → End/Cancel. Tag-based requirements, blocking, and cancellation.

**Cues** (`src/cues/`) — Visual/audio feedback. `GameplayCueManager` resource routes cue events to static (trait-based, no entity) or actor (spawned entity) handlers via hierarchical tag matching.

**Core** (`src/core/`) — Shared types: system sets, event re-exports, handle types with generation counters.

## System Execution Order

All systems run in `Update`, chained via `GasSystemSet`:

```
Input → Attributes → Effects → Abilities → Cues → Cleanup
```

Each top-level set has sub-sets (e.g., `EffectSystemSet::Apply → CreateModifiers → Aggregate → UpdateDurations → ExecutePeriodic → RemoveExpired → RemoveInstant`). Add custom systems to the appropriate set with `.in_set(GasSystemSet::X)`.

## Key Patterns

- **Entity-per-thing**: Attributes, effects, and abilities are all separate entities (not stored in Vec on the owner). This enables Bevy query optimization and parallel execution.
- **Event-driven**: Cross-module communication uses Bevy events (`ApplyGameplayEffectEvent`, `TryActivateAbilityEvent`, etc.).
- **Definition/Registry pattern**: `GameplayEffectDefinition` and `AbilityDefinition` are templates stored in `Resource` registries. Runtime instances are spawned as entities.
- **Builder pattern**: Definitions use builder methods (`GameplayEffectDefinition::new("id").with_duration(5.0).add_modifier(...)`).
- **Tag methods require `&Res<GameplayTagsManager>`**: Any method that adds tags to a `GameplayTagContainer` needs the tags manager resource.

## Gameplay Tags

Tags are defined in `assets/gameplay_tags.json`. The `bevy_gameplay_tag` plugin loads this at startup. Tests that use tags must add `GameplayTagsPlugin::with_data_path("assets/gameplay_tags.json".to_string())` and call `app.update()` before accessing the manager.

## Testing Patterns

Tests use `App::new()` with necessary plugins, then `app.world_mut().run_system_once(|...|{ ... })` for system-parameter-dependent tests. See `src/abilities/definition.rs` and `src/effects/definition.rs` for examples.

## Code Quality

- Correctness over convenience — crash on invalid state rather than silently continuing
- Make illegal states unrepresentable (enums over strings/sentinels)
- Exhaustive pattern matching
- Document WHY, not what
- No over-engineering: only make changes directly requested
- Delete unused code completely, no backwards-compat hacks
