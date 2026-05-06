# Ability 激活流程设计文档

本文档详细描述 Ability 系统在 ECS 架构中的完整激活流程，包括每一步生成的 Entity、Component，以及 System 的执行顺序。

## 概览

Ability 激活流程是一个多阶段的管道：

```
TryActivate → CanActivate 检查 → PendingActivation → 生成实例 → ReadyToActivate → Activate → Commit → End/Cancel
```

每个阶段涉及特定的 Entity、Component 和 System，通过 Bevy 的 Observer 模式和 System 排序协同工作。

## Entity 层级结构

Ability 系统使用三层 Entity 层级：

```
Owner Entity (玩家/角色)
  ├── Component: OwnedTags
  ├── Component: BlockedAbilityTags
  └── Child: AbilitySpec Entity (已授予的技能槽位)
        ├── Component: AbilitySpec (definition_id, level, input_id)
        ├── Component: AbilityActiveState (is_active, active_count)
        ├── Component: AbilityOwner (指向 owner 的链接)
        ├── Component: AbilityActivationHistory (统计信息)
        └── Child: AbilitySpecInstance Entity (激活的实例，激活时生成)
              ├── Component: AbilitySpecInstance (behavior, owner, target_data)
              ├── Component: InstanceControlState (is_active, is_blocking, is_cancelable)
              └── Component: ChildOf (指向父 AbilitySpec 的链接)
```

**关键设计决策：**
- **Entity-per-thing**：每个技能槽位和每个激活实例都是独立的 Entity，利用 Bevy 的查询优化和并行执行
- **层级清理**：当 `AbilitySpec` 被 despawn 时，Bevy 自动 despawn 所有子 `AbilitySpecInstance` Entity
- **Observer 驱动**：`Remove<AbilitySpecInstance>` 的 Observer 确保调用 `behavior.end()` 进行清理

## 实例化策略

系统支持三种实例化策略，决定如何创建实例 Entity：

### NonInstanced（无实例）
- **不创建实例 Entity**
- 逻辑直接从 definition 执行
- 最佳性能，适合简单技能（buff、动画播放）
- 无法存储每次激活的状态
- 所有回调中 `instance_entity` 为 `None`

### InstancedPerActor（每角色一个实例）
- **每个角色一个实例 Entity，跨激活复用**
- 状态在激活之间持久化
- 适合引导技能、连击计数器
- 首次激活时生成实例 Entity，后续复用

### InstancedPerExecution（每次执行一个实例，默认）
- **每次激活创建新的实例 Entity**
- 状态仅在激活期间存在
- 最常见模式（火球术、冲刺等）
- 每次激活生成和销毁实例 Entity

## 激活流程：分步详解

### 阶段 1：尝试激活（事件触发）

**用户操作：**
```rust
commands.trigger(TryActivateAbilityEvent::new(ability_spec_entity, owner_entity));
```

**发生的事情：**
- 事件发送到 `on_try_activate_ability` Observer
- 此时尚未创建任何 Entity 或 Component

---

### 阶段 2：激活检查（Observer）

**Observer：** `on_try_activate_ability`

**执行的检查：**
1. 从 `AbilityRegistry` 解析 `AbilityDefinition`
2. 调用 `behavior.can_activate()`：
   - 检查冷却（通过 cooldown effect 的 granted tags）
   - 检查 source 的 required/blocked tags
   - 检查 target 的 required/blocked tags（如果提供了 target）
3. 检查 owner 的 required/blocked tags
4. 检查 owner 是否有阻塞标签（来自其他激活的技能）
5. 取消匹配标签的技能（如果设置了 `cancel_abilities_with_tags`）

**成功时：**
- **添加 Component：** `PendingActivation` 标记到 `AbilitySpec` Entity
  ```rust
  PendingActivation {
      owner: Entity,
      activation_info: AbilityActivationInfo,
  }
  ```

**失败时：**
- 触发 `AbilityActivationFailedEvent`
- 流程在此停止

---

### 阶段 3：生成实例（System）

**System：** `spawn_pending_ability_instances_system`  
**运行于：** `GasSystemSet::Abilities`（链中第一个）

**查询：** `Query<(Entity, &PendingActivation, &AbilitySpec), With<PendingActivation>>`

**对每个待激活的技能：**

1. **从 `AbilityRegistry` 解析 definition**

2. **根据实例化策略确定实例 Entity：**

   **NonInstanced：**
   ```rust
   instance_entity = None;
   ```

   **InstancedPerActor：**
   ```rust
   // 检查实例是否已存在
   if let Some(existing) = find_existing_instance(spec_entity) {
       instance_entity = Some(existing);
   } else {
       // 生成新实例作为 AbilitySpec 的子 Entity
       let new_instance = commands.spawn((
           AbilitySpecInstance { ... },
           InstanceControlState::default(),
       )).set_parent(spec_entity).id();
       instance_entity = Some(new_instance);
   }
   ```

   **InstancedPerExecution：**
   ```rust
   // 总是生成新实例作为 AbilitySpec 的子 Entity
   let new_instance = commands.spawn((
       AbilitySpecInstance {
           definition_id: spec.definition_id.clone(),
           level: spec.level,
           behavior: def.behavior.clone(),
           owner: pending.owner,
           instigator: pending.activation_info.instigator,
           target_data: pending.activation_info.target_data.clone(),
       },
       InstanceControlState {
           is_active: true,
           is_blocking_other_abilities: def.default_blocks_other_abilities,
           is_cancelable: def.default_is_cancelable,
       },
   )).set_parent(spec_entity).id();
   instance_entity = Some(new_instance);
   ```

3. **移除 `PendingActivation` 标记**

4. **添加 `ReadyToActivate` 标记：**
   ```rust
   commands.entity(spec_entity).insert(ReadyToActivate {
       owner: pending.owner,
       instance: instance_entity,
       activation_info: pending.activation_info,
   });
   ```

**创建的 Entity：**
- `AbilitySpecInstance` Entity（对于实例化策略）

**添加的 Component：**
- `AbilitySpecInstance`（在实例 Entity 上）
- `InstanceControlState`（在实例 Entity 上）
- `ChildOf`（链接实例到 spec）
- `ReadyToActivate`（在 spec Entity 上，替换 `PendingActivation`）

---

### 阶段 4：激活（System）

**System：** `call_activate_ability_system`  
**运行于：** `GasSystemSet::Abilities`（链中第二个，在生成系统之后）

**查询：** `Query<(Entity, &ReadyToActivate, &AbilitySpec, &AbilityOwner), With<ReadyToActivate>>`

**对每个准备激活的技能：**

1. **从 `AbilityRegistry` 解析 definition**

2. **调用 `behavior.pre_activate()`：**
   - 将 `activation_owned_tags` 添加到 owner 的 `OwnedTags`
   - 将 `block_abilities_with_tags` 添加到 owner 的 `BlockedAbilityTags`
   - 这些标签通过 `update_tag_container_count(+1)` 添加

3. **调用 `behavior.activate()`：**
   - 主要技能逻辑在此执行
   - 自定义实现可以生成投射物、播放动画等

4. **递增 `AbilityActiveState`：**
   ```rust
   active_state.increment(); // active_count++, is_active = true
   ```

5. **触发 `AbilityActivatedEvent`：**
   ```rust
   commands.trigger(AbilityActivatedEvent {
       ability_spec: spec_entity,
       owner: owner.0,
       instance: ready.instance,
   });
   ```

6. **移除 `ReadyToActivate` 标记**

**修改的 Component：**
- owner 上的 `OwnedTags`（添加标签）
- owner 上的 `BlockedAbilityTags`（添加标签）
- spec 上的 `AbilityActiveState`（递增）

---

### 阶段 5：提交（Observer）

**Observer：** `on_commit_ability`  
**触发者：** `CommitAbilityEvent`（通常从 `behavior.activate()` 发送）

**执行的检查：**
1. 调用 `behavior.commit_check()`：
   - 重新检查冷却（以防激活期间改变）
   - 重新检查消耗（以防资源改变）

**成功时：**
2. 调用 `behavior.commit_execute()`：
   - 应用冷却效果（如果定义）
   - 应用消耗效果（如果定义）

3. 触发 `CommitAbilityResultEvent`，`success: true`

**失败时：**
- 触发 `CommitAbilityResultEvent`，`success: false`
- 技能保持激活但不应用消耗/冷却

**应用的效果：**
- 冷却效果创建带有 granted tags 的 `ActiveGameplayEffect` Entity
- 消耗效果修改属性（例如减少法力）

---

### 阶段 6：结束/取消（Observer）

**Observer：** `on_end_ability` 或 `on_cancel_ability`  
**触发者：** `EndAbilityEvent` 或 `CancelAbilityEvent`

**两个 Observer 都调用：** `end_ability_internal()`

**对每个激活的实例：**

1. **从 `AbilityRegistry` 解析 definition**

2. **调用 `behavior.end()`：**
   - 清理逻辑在此执行
   - 在实例上触发 `OnGameplayAbilityEnded` Entity 事件

3. **从 owner 移除标签：**
   - 从 owner 的 `OwnedTags` 移除 `activation_owned_tags`（通过 `update_tag_container_count(-1)`）
   - 从 owner 的 `BlockedAbilityTags` 移除 `block_abilities_with_tags`（通过 `update_tag_container_count(-1)`）

4. **销毁实例 Entity**（根据实例化策略）：
   - **NonInstanced：** 无 Entity 需要销毁
   - **InstancedPerActor：** 保留实例 Entity（下次激活复用）
   - **InstancedPerExecution：** 销毁实例 Entity

5. **递减 `AbilityActiveState`：**
   ```rust
   active_state.decrement(); // active_count--, 如果 count == 0 则 is_active = false
   ```

**销毁的 Entity：**
- `AbilitySpecInstance` Entity（对于 `InstancedPerExecution`）

**修改的 Component：**
- owner 上的 `OwnedTags`（移除标签）
- owner 上的 `BlockedAbilityTags`（移除标签）
- spec 上的 `AbilityActiveState`（递减）

---

## System 执行顺序

所有 Ability System 在 `Update` schedule 中运行，位于 `GasSystemSet::Abilities` 内：

```
GasSystemSet::Abilities:
  1. spawn_pending_ability_instances_system
  2. call_activate_ability_system
  3. tick_wait_delay_tasks_system
  4. check_wait_target_data_tasks_system
  5. cleanup_finished_tasks_system
  6. handle_owned_tag_added_triggers_system
  7. handle_owned_tag_present_triggers_system
  8. check_wait_attribute_change_tasks_system
  9. execute_apply_effect_to_target_data_tasks_system
```

**Observer**（触发时立即运行，不在 System 顺序中）：
- `on_try_activate_ability`
- `on_commit_ability`
- `on_end_ability`
- `on_cancel_ability`
- `on_instance_removed`
- 任务相关 Observer（共 12 个）

**跨模块顺序：**
```
GasSystemSet::Input
  ↓
GasSystemSet::Attributes
  ↓
GasSystemSet::Effects
  ↓
GasSystemSet::Abilities  ← Ability System 在此运行
  ↓
GasSystemSet::Cues
  ↓
GasSystemSet::Cleanup
```

---

## Component 生命周期总结

| Component | 创建时机 | 销毁时机 | 所在 Entity |
|-----------|---------|---------|------------|
| `AbilitySpec` | 技能授予给角色 | 技能从角色移除 | Spec Entity |
| `AbilityActiveState` | 技能授予 | 技能移除 | Spec Entity |
| `AbilityOwner` | 技能授予 | 技能移除 | Spec Entity |
| `AbilityActivationHistory` | 技能授予（可选） | 技能移除 | Spec Entity |
| `PendingActivation` | `can_activate` 通过 | 实例生成 | Spec Entity（临时） |
| `ReadyToActivate` | 实例生成 | 激活调用 | Spec Entity（临时） |
| `AbilitySpecInstance` | 激活（实例化策略） | 结束/取消（InstancedPerExecution） | Instance Entity |
| `InstanceControlState` | 激活（实例化策略） | 结束/取消（InstancedPerExecution） | Instance Entity |

---

## 标签管理

标签通过 owner Entity 上的两个 Component 管理：

### OwnedTags
- 包含 Entity 当前的所有标签
- 修改来源：
  - `activation_owned_tags`（在 `pre_activate` 添加，在 `end` 移除）
  - Gameplay Effect（通过 `granted_tags`）
  - 手动标签操作

### BlockedAbilityTags
- 包含阻止其他技能激活的标签
- 修改来源：
  - `block_abilities_with_tags`（在 `pre_activate` 添加，在 `end` 移除）
  - 在 `can_activate` 中检查以阻止激活

**标签计数：**
- 标签通过 `update_tag_container_count(delta)` 使用引用计数
- 多个来源可以添加相同标签
- 仅当计数归零时才移除标签

---

## 错误处理

### 激活失败

**失败原因：**
- `OnCooldown`：冷却效果的 granted tags 存在于 owner 上
- `InsufficientCost`：消耗效果会失败（例如法力不足）
- `MissingRequiredTags`：Owner/source/target 缺少必需标签
- `BlockedByTags`：Owner/source/target 有阻塞标签

**失败事件：**
```rust
AbilityActivationFailedEvent {
    ability_spec: Entity,
    owner: Entity,
    reason: ActivationFailureReason,
}
```

### 提交失败

**提交可能在激活后失败：**
- 激活期间另一个技能应用了冷却
- 激活期间另一个系统消耗了资源

**失败处理：**
- 技能保持激活（不自动结束）
- 发送 `CommitAbilityResultEvent`，`success: false`
- 自定义 behavior 可以决定是否结束技能

---

## 示例：完整激活流程

**场景：** 玩家激活"火球术"技能（InstancedPerExecution 策略）

1. **用户输入：**
   ```rust
   commands.trigger(TryActivateAbilityEvent::new(fireball_spec, player));
   ```

2. **Observer `on_try_activate_ability`：**
   - 检查冷却：✓（玩家上无冷却标签）
   - 检查法力消耗：✓（玩家有 50 法力，消耗 30）
   - 检查标签：✓（玩家有 `State.Alive`，无 `State.Stunned`）
   - **添加：** `PendingActivation` 到 `fireball_spec` Entity

3. **System `spawn_pending_ability_instances_system`：**
   - **生成：** `AbilitySpecInstance` Entity 作为 `fireball_spec` 的子 Entity
   - **添加：** `AbilitySpecInstance`、`InstanceControlState` 到实例
   - **移除：** `fireball_spec` 的 `PendingActivation`
   - **添加：** `ReadyToActivate` 到 `fireball_spec`

4. **System `call_activate_ability_system`：**
   - 调用 `behavior.pre_activate()`：
     - **添加：** `Ability.Casting` 到玩家的 `OwnedTags`
     - **添加：** `Ability.Casting` 到玩家的 `BlockedAbilityTags`
   - 调用 `behavior.activate()`：
     - 生成投射物 Entity
     - 播放施法动画
     - 触发 `CommitAbilityEvent`
   - **递增：** `fireball_spec` 的 `AbilityActiveState`（active_count = 1）
   - **触发：** `AbilityActivatedEvent`
   - **移除：** `fireball_spec` 的 `ReadyToActivate`

5. **Observer `on_commit_ability`：**
   - 调用 `behavior.commit_check()`：✓
   - 调用 `behavior.commit_execute()`：
     - **应用：** 冷却效果（添加 `Cooldown.Fireball` 标签 5 秒）
     - **应用：** 消耗效果（减少 30 法力）
   - **触发：** `CommitAbilityResultEvent`（success: true）

6. **投射物击中目标，技能结束：**
   ```rust
   commands.trigger(EndAbilityEvent {
       instance: Some(instance_entity),
       ability_spec: fireball_spec,
       owner: player,
   });
   ```

7. **Observer `on_end_ability`：**
   - 调用 `behavior.end()`：
     - **触发：** 实例 Entity 上的 `OnGameplayAbilityEnded`
   - **移除：** 玩家 `OwnedTags` 的 `Ability.Casting`
   - **移除：** 玩家 `BlockedAbilityTags` 的 `Ability.Casting`
   - **销毁：** `AbilitySpecInstance` Entity
   - **递减：** `fireball_spec` 的 `AbilityActiveState`（active_count = 0, is_active = false）

---

## 高级特性

### 技能取消

**取消 vs 结束：**
- **结束：** 正常完成（技能完成其逻辑）
- **取消：** 被打断（例如玩家被眩晕，另一个技能取消了它）

**取消匹配：**
- 技能可以指定 `cancel_abilities_with_tags`
- 激活时，取消所有具有匹配 `ability_tags` 的激活技能
- 用于打断机制（例如眩晕取消施法）

### 技能阻塞

**阻塞匹配：**
- 技能可以指定 `block_abilities_with_tags`
- 激活时，将这些标签添加到 owner 的 `BlockedAbilityTags`
- 其他技能在 `can_activate` 中检查 `BlockedAbilityTags`
- 用于防止多次施法（例如施法时不能施法）

### 激活历史

**跟踪的统计信息：**
- `activation_count`：总激活次数
- `successful_activation_count`：成功激活次数
- `failed_activation_count`：失败激活次数
- `last_activation_time`：最后尝试的时间戳
- `last_successful_activation_time`：最后成功的时间戳

**用例：**
- 连击系统（跟踪连续激活）
- 冷却减少（根据使用减少冷却）
- 分析和调试

---

## 测试模式

### 单元测试

**测试激活检查：**
```rust
let mut app = App::new();
app.add_plugins((GameplayTagsPlugin, AttributePlugin, EffectPlugin, AbilityPlugin));

let player = app.world_mut().spawn((
    OwnedTags::default(),
    BlockedAbilityTags::default(),
)).id();

let ability_spec = app.world_mut().spawn((
    AbilitySpec::new("fireball", 1),
    AbilityActiveState::default(),
    AbilityOwner(player),
)).id();

app.world_mut().trigger(TryActivateAbilityEvent::new(ability_spec, player));
app.update();

// 检查激活成功
let state = app.world().get::<AbilityActiveState>(ability_spec).unwrap();
assert!(state.is_active);
```

### 集成测试

**测试完整生命周期：**
```rust
// 使用 Arc<Mutex<Vec<Event>>> 捕获事件
let events = Arc::new(Mutex::new(Vec::new()));
app.insert_resource(TestEvents(events.clone()));

// 触发激活
app.world_mut().trigger(TryActivateAbilityEvent::new(spec, owner));
app.update();

// 验证事件
let captured = events.lock().unwrap();
assert!(captured.iter().any(|e| matches!(e, AbilityActivatedEvent { .. })));
```

---

## 常见陷阱

### 1. 缺少 OwnedTags/BlockedAbilityTags

**问题：** 激活静默失败，因为 owner Entity 没有必需的 Component。

**解决方案：** 始终使用以下方式生成 owner Entity：
```rust
commands.spawn((
    OwnedTags::default(),
    BlockedAbilityTags::default(),
));
```

### 2. 忘记调用 Commit

**问题：** 技能激活但从未应用消耗/冷却。

**解决方案：** 始终从 `behavior.activate()` 触发 `CommitAbilityEvent`：
```rust
fn activate(&self, commands: &mut Commands, instance: Option<Entity>, spec: Entity, _info: &AbilityActivationInfo) {
    // ... 技能逻辑 ...
    commands.trigger(CommitAbilityEvent {
        ability_spec: spec,
        instance,
        owner: self.owner,
    });
}
```

### 3. 不结束技能

**问题：** 技能永远保持激活，阻塞其他技能。

**解决方案：** 技能完成时始终触发 `EndAbilityEvent`：
```rust
commands.trigger(EndAbilityEvent {
    instance: Some(instance_entity),
    ability_spec: spec_entity,
    owner: owner_entity,
});
```

### 4. 实例化策略混淆

**问题：** 期望 NonInstanced 技能有实例 Entity。

**解决方案：** 使用实例 Entity 前始终检查 `instance.is_some()`：
```rust
if let Some(instance) = instance_entity {
    // 安全使用实例
}
```

---

## 性能考虑

### Entity-per-Instance 设计

**优点：**
- Bevy 的查询优化（并行迭代）
- 自动层级清理
- 清晰的所有权和生命周期

**成本：**
- Entity 生成/销毁开销
- 世界中更多 Entity

**优化：**
- 对简单技能使用 `NonInstanced`（无每次激活状态）
- 对频繁激活的技能使用 `InstancedPerActor`（减少生成/销毁）

### 标签计数开销

**标签操作使用引用计数：**
- `update_tag_container_count(+1)` 递增计数
- `update_tag_container_count(-1)` 递减计数
- 仅当计数归零时才移除标签

**优化：**
- 最小化标签变动（避免重复添加/移除相同标签）
- 使用层级标签进行高效匹配（例如 `Ability.Casting.*`）

---

## 未来增强

**计划功能：**
- 技能队列（当前技能结束时激活下一个）
- 技能批处理（同时激活多个技能）
- 技能预测（网络游戏的客户端预测）
- 技能重放（记录和重放激活序列）

**不计划：**
- 网络/复制（单人游戏焦点）
- 可视化脚本（仅基于代码）
