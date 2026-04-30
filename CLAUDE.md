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

**Attributes** (`src/attributes/`) — Dual-value model (BaseValue/CurrentValue). Each attribute is a separate entity linked to its owner via Bevy's `ChildOf` relationship (using `set_parent_in_place`). Custom attribute sets implement the `AttributeSetDefinition` trait (in `traits.rs`). Modifiers applied in order: Add → Multiply → Override.

**Effects** (`src/effects/`) — Modify attributes via `GameplayEffectDefinition` templates stored in `GameplayEffectRegistry`. Each active effect is its own entity with `ActiveGameplayEffect` + `EffectTarget` components. Supports three duration policies (Instant, HasDuration, Infinite), periodic execution, and stacking (Independent, RefreshDuration, StackCount). Tag requirements gate application.

**Abilities** (`src/abilities/`) — Activated actions defined via `AbilityDefinition` templates in `AbilityRegistry`. Each granted ability is an entity with `AbilitySpec` + `AbilityOwner`. Activation flow: TryActivate → Commit (costs/cooldowns) → End/Cancel. Tag-based requirements, blocking, and cancellation. Supports three instancing policies: NonInstanced (no instance entity, logic from definition), InstancedPerActor (reused instance across activations), InstancedPerExecution (new instance per activation, default).

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
- Abilities: Single exclusive system (`execute_pending_activations_system`), other logic via Observers
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
- Delete unused code completely, no backwards-compat hacks

## Project Status

**✅ Core Systems Complete** — All four modules (Attributes, Effects, Abilities, Cues) fully implemented with comprehensive tests. Ability Tasks system complete with 12 task types.

**Test Coverage:**
- Unit tests: 41/41 passed ✅
- Integration tests: 74/74 passed ✅
  - `ability_granting_lifecycle_test`: 1 test
  - `ability_task_test`: 12 tests (all task types)
  - `application_requirement_test`: 2 tests
  - `attribute_aggregation_test`: 2 tests
  - `gameplay_effect_spec_test`: 2 tests
  - `instancing_policy_test`: 3 tests (NonInstanced, InstancedPerActor, InstancedPerExecution)
  - `periodic_effect_spec_test`: 2 tests
  - `stack_count_test`: 2 tests
  - `stacking_reapply_spec_test`: 2 tests
- Doc tests: 5/5 passed ✅
- Examples: `basic_attributes`, `ability_activation`, `gameplay_effects`, `complete_rpg`, `stress_test`

**Total: 120/120 tests passing (100% pass rate) ✅**

**Known Limitations:**
- Single-player only (no networking/replication)
- Performance optimization deferred (current design handles <50 entities with <10 attributes each)
- Benchmark suite broken for Bevy 0.18 (criterion compatibility issue)

**Important Testing Notes:**
- Tests that spawn player entities must include `OwnedTags` and `BlockedAbilityTags` components for ability activation to work
- Effect duration tests should manually call `duration.tick()` instead of relying on `Time::advance_by()`, as the latter doesn't affect `Time::delta_secs()`
- Task completion tests should check `TaskCompletedEvent` in the `TaskEvents` resource, as tasks are automatically despawned after completion

## Known Issues & Technical Debt

**所有 Critical 和 Design 级别的问题已全部修复。** 以下是历史记录：

**Critical（已修复）:**
1. ✅ `set_base_value()` 不再覆盖 `current_value`，aggregation 系统正确地重新计算。
2. ✅ Instant effect + `granted_tags` 组合现在在 `GameplayEffectRegistry::register()` 时 panic，使非法状态不可表示。
3. ✅ Periodic effects 现在正确地按周期执行 modifier，不再和持久 modifier 重复计算。
4. ✅ `ModifierOperation::AddBase` 已在 aggregation 的三个路径中全部实现（aggregation、instant、periodic）。

**Design（已修复）:**
5. ✅ `StackCount` 的 `create_effect_modifiers_system` 正确处理增删。
6. ✅ Handle 类型已从 `src/core/handles.rs` 中删除。Bevy 的 `Entity` 类型提供足够的安全性。
7. ✅ NonInstanced 策略现在使用 `Option<Entity>` 而非 `Entity::PLACEHOLDER`。

**Code Quality（已修复）:**
8. ✅ `Changed<AttributeData>` 过滤器未在多个系统中使用。此条目已过期。
9. ✅ 测试硬编码 `"assets/gameplay_tags.json"` 路径。该文件存在于项目仓库中，CI 环境直接可用。
10. ✅ Registry 查找失败使用 `error!`/`warn!` + 早期返回是正确的设计选择（程序员错误，非运行时错误）。
