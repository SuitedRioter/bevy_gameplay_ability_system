# Ability Activation Flow Design

## Overview

Implement UE's GameplayAbility activation flow in pure Bevy Observer pattern. Covers TryActivate → CanActivate → PreActivate → ActivateAbility → CommitAbility → EndAbility, plus the missing `apply_gameplay_effect_system`.

Target: Bevy 0.18, single-player, pure ECS.

## Architecture Decision

**Pure Observer pattern.** All cross-module communication via `commands.trigger()` / `commands.trigger_targets()`. Same-frame synchronous execution, no frame delay.

User-defined ability logic via observing `AbilityActivatedEvent` — the ECS equivalent of UE's virtual `ActivateAbility()`.

## Activation Flow

```
trigger(TryActivateAbilityEvent)
  │
  ├─ CanActivateAbility checks (in order):
  │   1. Already active? (NonInstanced/InstancedPerActor)
  │   2. Cooldown? (owner has cooldown effect's granted_tags)
  │   3. Cost? (owner attributes sufficient for cost effect modifiers)
  │   4. Required tags? (owner has activation_required_tags)
  │   5. Blocked tags? (owner lacks activation_blocked_tags)
  │
  │   Any fail → trigger(AbilityActivationFailedEvent) with reason
  │
  ├─ PreActivate:
  │   1. spec.is_active = true, spec.active_count += 1
  │   2. Add activation_owned_tags to owner's GameplayTagCountContainer
  │   3. Add block_abilities_with_tags to owner's GameplayTagCountContainer
  │   4. Cancel other active abilities matching cancel_abilities_with_tags
  │      (matched against target ability's ability_tags)
  │
  └─ trigger_targets(AbilityActivatedEvent, owner)
      │
      └─ User observes this, writes custom logic:
          ├─ trigger(CommitAbilityEvent)  ← must call
          └─ trigger(EndAbilityEvent)     ← must call when done
```

### CommitAbility Flow

```
trigger(CommitAbilityEvent)
  │
  ├─ CommitCheck:
  │   1. Cooldown check (same as CanActivate)
  │   2. Cost check (same as CanActivate)
  │   Fail → trigger(CommitAbilityResultEvent { success: false })
  │
  └─ CommitExecute:
      1. trigger(ApplyGameplayEffectEvent) for cost_effect
      2. trigger(ApplyGameplayEffectEvent) for cooldown_effect
      3. Mark instance as committed
      4. trigger(CommitAbilityResultEvent { success: true })
```

### EndAbility Flow

```
trigger(EndAbilityEvent)
  │
  1. Remove activation_owned_tags from owner (update_tag_count -1)
  2. Remove block_abilities_with_tags from owner (update_tag_count -1)
  3. spec.is_active = false, spec.active_count -= 1
  4. Despawn ActiveAbilityInstance (if InstancedPerExecution)
  5. trigger_targets(AbilityEndedEvent, owner)
```

### CancelAbility Flow

Same as EndAbility, but `AbilityEndedEvent.was_cancelled = true`.

## Event Design

| Event | Direction | Purpose |
|-------|-----------|---------|
| `TryActivateAbilityEvent` | user→framework | Request activation |
| `AbilityActivatedEvent` | framework→user | Notify success, user writes custom logic |
| `AbilityActivationFailedEvent` | framework→user | Notify failure with reason |
| `CommitAbilityEvent` | user→framework | Request cost/cooldown application |
| `CommitAbilityResultEvent` | framework→user | Notify commit result |
| `EndAbilityEvent` | user→framework | Request ability end |
| `AbilityEndedEvent` | framework→user | Notify ability ended |
| `CancelAbilityEvent` | user/framework→framework | Request ability cancel |
| `ApplyGameplayEffectEvent` | any→framework | Request effect application |

## Component Changes

### AbilityDefinition — new fields

```rust
pub ability_tags: GameplayTagContainer,           // Tags describing this ability
pub block_abilities_with_tags: GameplayTagContainer, // Tags added to owner to block other abilities
```

Builder methods: `add_ability_tag()`, `add_block_abilities_with_tag()`.

### AbilitySpec — new field

```rust
pub active_count: u8,  // Number of active instances
```

### AbilityEndedEvent — new field

```rust
pub was_cancelled: bool,
```

### New events

```rust
pub struct EndAbilityEvent { pub ability_spec: Entity, pub owner: Entity }
pub struct AbilityActivationFailedEvent { pub ability_spec: Entity, pub owner: Entity, pub reason: ActivationFailureReason }
pub struct CommitAbilityResultEvent { pub ability_spec: Entity, pub owner: Entity, pub success: bool }
```

### New enum

```rust
pub enum ActivationFailureReason {
    AlreadyActive,
    OnCooldown,
    InsufficientCost,
    MissingRequiredTags,
    BlockedByTags,
}
```

## Cooldown Check

Cooldown is determined by checking if the owner's `GameplayTagCountContainer` has any of the cooldown effect's `granted_tags`. No separate `cooldown_tags` field on `AbilityDefinition` — derived at runtime from `GameplayEffectRegistry`.

## Cost Check

Pre-evaluate cost effect's modifiers against owner's attribute `CurrentValue`. Cost modifiers are typically negative `AddCurrent` operations. Check `current_value + magnitude >= 0` for each modifier.

## apply_gameplay_effect_system (Observer)

```
trigger(ApplyGameplayEffectEvent)
  │
  1. Look up definition from GameplayEffectRegistry
  2. Check application_tag_requirements
  3. Handle stacking (Independent / RefreshDuration / StackCount)
  4. For Instant effects: directly modify attribute base_value, no entity spawn
  5. For HasDuration/Infinite: spawn effect entity with components
  6. Add granted_tags to target's GameplayTagCountContainer
  7. trigger(GameplayEffectAppliedEvent)
```

### Effect removal tag cleanup

`remove_expired_effects_system` must also remove `granted_tags` from target's `GameplayTagCountContainer` and trigger `GameplayEffectRemovedEvent`.

## Plugin Registration

### AbilityPlugin

- Register observers: `on_try_activate_ability`, `on_commit_ability`, `on_end_ability`, `on_cancel_ability`
- Delete stub systems: `try_activate_ability_system`, `commit_ability_system`, `end_ability_system`
- Keep systems: `cancel_abilities_by_tags_system`, `update_ability_states_system`, `update_ability_cooldowns_system`
- Configure system sets: `.in_set(GasSystemSet::Abilities)`

### EffectPlugin

- Register observer: `on_apply_gameplay_effect`
- Delete stub: `apply_gameplay_effect_system`
- Keep systems: `create_effect_modifiers_system`, `aggregate_attribute_modifiers_system`, etc.
- Configure system sets: `.in_set(GasSystemSet::Effects)`

## File Changes

| File | Change |
|------|--------|
| `src/abilities/definition.rs` | Add `ability_tags`, `block_abilities_with_tags` fields + builders |
| `src/abilities/components.rs` | Add `active_count` to `AbilitySpec` |
| `src/abilities/systems.rs` | Delete stubs, add observer fns, add new events/enums |
| `src/abilities/plugin.rs` | Register observers, configure system sets |
| `src/effects/systems.rs` | Implement `on_apply_gameplay_effect`, update `remove_expired_effects_system` |
| `src/effects/plugin.rs` | Register observer, configure system sets |
| `src/core/events.rs` | Re-export new events |

## Testing

Integration tests (App + plugins + app.update()):
- Full activation flow: TryActivate → Activated → Commit → End → Ended
- Activation failures: cooldown, cost, tags
- Tag management: owned_tags added on activate, removed on end
- Blocking: ability A blocks ability B via tags
- Cancellation: ability A cancels ability B via tags
- Effect application: spawn entity, add granted_tags
- Effect expiry: remove entity, remove granted_tags
- Instant effects: direct BaseValue modification

## Out of Scope

- AbilityTask system (async tasks)
- Input binding system
- Trigger system (tag-based auto-activation)
- Networking (NetExecutionPolicy)
- Cue system integration
