# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A Bevy plugin implementing Unreal Engine's GameplayAbilitySystem (GAS) in pure ECS architecture. Targets Bevy 0.18 (Rust edition 2024) and single-player games. Uses `bevy_gameplay_tag` (git dependency from SuitedRioter/bevy_gameplay_tag) for hierarchical tag matching and `string_cache` for interned strings.

## Build & Test Commands

```bash
cargo build                              # Build the library
cargo test                               # Run all tests
cargo test --lib                         # Run unit tests only
cargo test <test_name>                   # Run a single test
cargo run --example basic_attributes     # Run an example
cargo run --example complete_rpg         # Full combat simulation
cargo bench                              # Run benchmarks (criterion, currently broken for Bevy 0.18)
```

Examples: `basic_attributes`, `ability_activation`, `gameplay_effects`, `complete_rpg`, `stress_test`.

## Architecture

Six modules. Effects, abilities, and cues follow `components.rs`/`definition.rs`/`plugin.rs`/`systems.rs`. Attributes uses `traits.rs` instead of `definition.rs`.

**Attributes** (`src/attributes/`) — Dual-value model (BaseValue/CurrentValue). Each attribute is a separate entity linked to its owner via `AttributeOwner`. Custom attribute sets implement the `AttributeSetDefinition` trait (in `traits.rs`). Modifiers applied in order: Add → Multiply → Override.

**Effects** (`src/effects/`) — Modify attributes via `GameplayEffectDefinition` templates stored in `GameplayEffectRegistry`. Each active effect is its own entity with `ActiveGameplayEffect` + `EffectTarget` components. Supports three duration policies (Instant, HasDuration, Infinite), periodic execution, and stacking (Independent, RefreshDuration, StackCount). Tag requirements gate application.

**Abilities** (`src/abilities/`) — Activated actions defined via `AbilityDefinition` templates in `AbilityRegistry`. Each granted ability is an entity with `AbilitySpec` + `AbilityOwner`. Activation flow: TryActivate → Commit (costs/cooldowns) → End/Cancel. Tag-based requirements, blocking, and cancellation.

**Cues** (`src/cues/`) — Visual/audio feedback. `GameplayCueManager` resource routes cue events to static (trait-based, no entity) or actor (spawned entity) handlers via hierarchical tag matching.

**Core** (`src/core/`) — Shared types: system sets, event re-exports, handle types with generation counters.

**Utils** (`src/utils/`) — Math utilities (`clamp_optional`, `lerp`, `remap`, `smoothstep`) and query helpers (`find_attribute_by_name`, `get_owner_attributes`, `get_active_effects_on_target`, `find_ability_by_definition`).

## System Execution Order

All systems run in `Update`, chained via `GasSystemSet`:

```
Input → Attributes → Effects → Abilities → Cues → Cleanup
```

Sub-sets (all chained within their parent):
- `AttributeSystemSet`: Clamp → Events
- `EffectSystemSet`: Apply → CreateModifiers → Aggregate → UpdateDurations → ExecutePeriodic → RemoveExpired → RemoveInstant
- `AbilitySystemSet`: TryActivate → Commit → End → Cancel → UpdateStates → UpdateCooldowns
- `CueSystemSet`: Handle → Route → ExecuteStatic → ManageActors → Cleanup → UpdateWhileActive

Add custom systems to the appropriate set with `.in_set(GasSystemSet::X)`.

## Key Patterns

- **Entity-per-thing**: Attributes, effects, and abilities are all separate entities (not stored in Vec on the owner). This enables Bevy query optimization and parallel execution.
- **Observer pattern**: Effects and abilities use Bevy 0.18 observers for event handling. `EffectPlugin` registers `on_apply_gameplay_effect`; `AbilityPlugin` registers `on_try_activate_ability`, `on_commit_ability`, `on_end_ability`, `on_cancel_ability`. Observer signature: `fn on_event(ev: On<EventType>, mut commands: Commands, ...)`.
- **Definition/Registry pattern**: `GameplayEffectDefinition` and `AbilityDefinition` are templates stored in `Resource` registries. Runtime instances are spawned as entities.
- **Builder pattern**: Definitions use builder methods (`GameplayEffectDefinition::new("id").with_duration(5.0).add_modifier(...)`).
- **SystemParam bundles**: Complex systems use `#[derive(SystemParam)]` to group related queries (e.g., `ActivationCheckParams`, `EndAbilityParams`, `ApplyEffectParams`).
- **Tag methods require `&Res<GameplayTagsManager>`**: Any method that adds tags to a `GameplayTagContainer` needs the tags manager resource.

## Plugin Composition

`GasPlugin` combines `AttributePlugin`, `EffectPlugin`, `AbilityPlugin`, and `CuePlugin`. Each can be added independently if needed.

## Gameplay Tags

Tags are defined in `assets/gameplay_tags.json` with hierarchical naming: `State.*` (Alive, Stunned, Disarmed), `Ability.*` (Casting, Blocking), `Cooldown.*` (Fireball, Attack), `Effect.*` (HealOverTime, Buff.Attack, Debuff.Poison).

The `bevy_gameplay_tag` plugin loads this at startup. Tests that use tags must add `GameplayTagsPlugin::with_data_path("assets/gameplay_tags.json".to_string())` and call `app.update()` before accessing the manager.

## Testing Patterns

Unit tests use `App::new()` with necessary plugins, then `app.world_mut().run_system_once(|...|{ ... })` for system-parameter-dependent tests. See `src/abilities/definition.rs` and `src/effects/definition.rs` for examples.

Integration tests in `tests/` (`ability_activation_flow.rs`, `effect_application.rs`) test full lifecycles. They use a `TestEvents` resource with `Arc<Mutex<Vec<T>>>` to capture events across observers.

## Code Quality

- Correctness over convenience — crash on invalid state rather than silently continuing
- Make illegal states unrepresentable (enums over strings/sentinels)
- Exhaustive pattern matching
- Document WHY, not what
- No over-engineering: only make changes directly requested
- Delete unused code completely, no backwards-compat hacks

## Project Status

**⚠️ In Active Development** — Core systems functional but incomplete. Examples and comprehensive tests pending.

## Known Issues & Technical Debt

**Critical:**
1. `AttributeData::set_base_value()` overwrites `current_value`, losing all active modifiers. Should only set `base_value` and let aggregation recalculate.
2. Instant effects with `granted_tags` cause tag leaks (tags added but never removed since no entity exists).
3. `execute_periodic_effects_system` has TODO — periodic effects tick but don't execute modifiers.
4. `ModifierOperation::AddBase` is skipped in aggregation (line 309 in effects/systems.rs) — semantic unclear.

**Design:**
5. `StackCount` policy spawns duplicate modifiers on each stack without cleanup.
6. Handle types (`AbilityHandle`, `EffectHandle`, `AttributeHandle`) defined but unused. Delete or implement generation tracking.
7. String IDs (`definition_id`, `attribute_name`) used everywhere despite `string_cache` dependency. Consider using `Atom` for performance.

**Code Quality:**
8. `Changed<AttributeData>` filter used in both clamp and event systems — may cause duplicate events in same frame.
9. Tests use hardcoded `"assets/gameplay_tags.json"` path — fails in CI/different environments.
10. Registry lookup failures use `warn!` + early return — callers can't detect failures. Consider error events.
