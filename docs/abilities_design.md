# Abilities 模块设计文档

## 概述

Abilities 模块提供了一个完整的游戏技能系统，允许实体拥有可激活、提交（消耗资源和触发冷却）、结束和取消的技能。灵感来自 UE 的 GameplayAbilitySystem，采用纯 ECS 架构实现。

## 设计理念

**定义与实例分离，行为可扩展**

- 技能模板（AbilityDefinition）存储在注册表中，运行时实例是独立实体
- 通过 `AbilityBehavior` trait 注入自定义逻辑，框架提供默认实现
- 基于 Tag 的条件系统实现技能间的阻塞、取消和前置条件检查

## ECS 架构设计

### 实体关系

```
Owner Entity (玩家/怪物)
    ├─ OwnedTags (所有者的 Tag 容器)
    ├─ BlockedAbilityTags (阻塞标签容器)
    │
    ├─ AbilitySpec Entity #1 (已授予的技能槽 - 火球术)
    │     ├─ AbilitySpec (definition_id, level, input_id)
    │     ├─ AbilityActiveState (is_active, active_count)
    │     ├─ AbilityOwner (owner Entity)
    │     ├─ AbilityActivationHistory (activation_count, last_activation_time) [可选]
    │     ├─ AbilityTriggers (triggers) [可选]
    │     │
    │     └─ AbilitySpecInstance Entity (激活中的实例, 子实体)
    │           ├─ AbilitySpecInstance (definition_id, level, behavior)
    │           ├─ InstanceControlState (is_active, is_blocking, is_cancelable)
    │           ├─ AbilityActivationInfo (owner, instigator, target_data, level)
    │           ├─ ChildOf → AbilitySpec Entity
    │           │
    │           └─ AbilityTask Entity (任务, 子实体)
    │                 ├─ AbilityTask (ability_instance, ability_spec, owner)
    │                 ├─ TaskState (Running/Completed/Cancelled)
    │                 ├─ WaitDelayTask / WaitGameplayEventTask / ... (具体任务类型)
    │                 └─ ChildOf → AbilitySpecInstance Entity
    │
    └─ AbilitySpec Entity #2 (已授予的技能槽 - 冲刺)
          ├─ AbilitySpec (...)
          └─ ...
```

**重要说明**：
- 一个 Owner Entity 可以拥有**多个** AbilitySpec Entity（多个已授予的技能）
- 根据 `InstancingPolicy`，实例的创建方式不同：
  - **NonInstanced**：不创建实例实体，逻辑直接从定义执行
  - **InstancedPerActor**：每个 AbilitySpec 只有一个实例，跨激活复用
  - **InstancedPerExecution**（默认）：每次激活创建新实例
- 每个实例可以有多个 Task 子实体，用于异步操作（等待、监听事件等）

### 核心组件

#### 1. AbilitySpec — 已授予的技能
```rust
pub struct AbilitySpec {
    pub definition_id: Atom,    // 引用 AbilityRegistry 中的定义
    pub level: i32,             // 授予时的等级
    pub input_id: Option<i32>,  // 可选的输入绑定 ID
}
```

代表角色已获得的技能。每个授予的技能是一个独立实体，挂载在所有者下。

#### 2. AbilityActiveState — 激活状态追踪
```rust
pub struct AbilityActiveState {
    pub is_active: bool,    // 是否有实例在运行
    pub active_count: u8,   // 当前活跃实例数
}
```

与 AbilitySpec 分离，使 Bevy 的变更检测（Change Detection）可以独立追踪激活状态变化。

#### 3. AbilitySpecInstance — 运行中的实例
```rust
pub struct AbilitySpecInstance {
    pub definition_id: Atom,
    pub level: i32,
    pub behavior: Option<Arc<dyn AbilityBehavior>>,
}
```

技能激活时，作为 AbilitySpec 的子实体生成。携带 behavior 的 Arc 克隆，用于执行技能逻辑。当父 AbilitySpec 被销毁时，Bevy 层级系统自动清理子实例。

#### 4. InstanceControlState — 实例运行时控制
```rust
pub struct InstanceControlState {
    pub is_active: bool,                    // 是否正在执行
    pub is_blocking_other_abilities: bool,  // 是否阻塞其他技能
    pub is_cancelable: bool,                // 是否可被取消
}
```

#### 5. AbilityActivationInfo — 激活上下文信息
```rust
pub struct AbilityActivationInfo {
    pub owner: Entity,              // 所有者（激活者）
    pub instigator: Entity,         // 触发者（可能与所有者不同）
    pub target_data: GameplayAbilityTargetData,  // 目标数据
    pub level: i32,                 // 激活时的等级
    pub event_payload: Option<GameplayEventData>, // 事件载荷（事件驱动时）
}
```

统一的激活上下文，携带激活时的所有必要信息。类似 UE GAS 的 `FGameplayAbilityActivationInfo` + `FGameplayAbilityActorInfo`。

#### 6. AbilityActivationHistory — 激活历史追踪
```rust
pub struct AbilityActivationHistory {
    pub activation_count: u32,          // 总激活次数
    pub last_activation_time: f64,      // 上次激活时间
    pub last_activation_result: ActivationResult,  // 上次激活结果
}
```

记录技能激活统计，用于调试、分析和游戏逻辑（如连击系统、基于使用次数的冷却减少）。

#### 7. AbilityTriggers — 触发器配置
```rust
pub struct AbilityTriggers {
    pub triggers: Vec<AbilityTriggerData>,
}

pub struct AbilityTriggerData {
    pub trigger_tag: GameplayTag,
    pub trigger_source: AbilityTriggerSource,  // GameplayEvent / OwnedTagAdded / OwnedTagPresent
}
```

定义技能如何被外部事件自动激活。匹配 UE GAS 的 `FAbilityTriggerData`。

#### 8. AbilityTask — 异步任务
```rust
pub struct AbilityTask {
    pub ability_instance: Option<Entity>,  // 所属实例
    pub ability_spec: Entity,              // 所属技能规格
    pub owner: Entity,                     // 所有者
}
```

任务是 ECS 实体，代表技能内的持续操作。作为实例的子实体生成，实例结束时自动清理。类似 UE 的 `UAbilityTask`。

### 为什么采用三层实体结构？

1. **Owner → AbilitySpec**: 同一技能只授予一次，但可以多次激活
2. **AbilitySpec → AbilitySpecInstance**: 支持同一技能的多个并发实例（如持续施法）
3. **AbilitySpecInstance → AbilityTask**: 支持技能内的异步操作（等待、监听事件等）
4. **状态分离**: AbilitySpec 追踪"拥有"，AbilitySpecInstance 追踪"执行"，AbilityTask 追踪"异步操作"
5. **自动清理**: 利用 Bevy 的 ChildOf 层级关系，移除技能自动清理所有实例和任务

## 实例化策略

技能支持三种实例化策略，控制实例的创建和管理方式：

### NonInstanced（无实例化）
- 不创建实例实体
- 逻辑直接从定义执行
- 无法存储每次激活的状态
- 性能最佳，适合简单技能
- 示例：简单增益应用、动画播放

### InstancedPerActor（每角色一个实例）
- 每个 AbilitySpec 只有一个实例
- 实例在激活间复用
- 状态在激活间持久化
- 适合需要跟踪累积状态的技能
- 示例：引导技能、连击计数器

### InstancedPerExecution（每次执行一个实例，默认）
- 每次激活创建新实例
- 状态仅在激活期间存在
- 最常见的模式
- 示例：大多数技能（火球术、冲刺等）

## 定义与注册

### AbilityDefinition — 技能模板

纯配置数据，存储在 `AbilityRegistry` 中：

```rust
pub struct AbilityDefinition {
    pub id: Atom,                              // 唯一标识
    pub instancing_policy: InstancingPolicy,   // 实例化策略
    pub cost_effect: Option<Atom>,             // 消耗效果 ID
    pub cooldown_effect: Option<Atom>,         // 冷却效果 ID
    pub ability_tags: GameplayTagContainer,    // 技能自身的标签
    pub activation_owned_tags: GameplayTagContainer,   // 激活时授予所有者的标签
    pub activation_required_tags: GameplayTagContainer, // 激活所需标签
    pub activation_blocked_tags: GameplayTagContainer,  // 阻止激活的标签
    pub source_required_tags: GameplayTagContainer,     // 来源所需标签
    pub source_blocked_tags: GameplayTagContainer,      // 来源阻止标签
    pub target_required_tags: GameplayTagContainer,     // 目标所需标签
    pub target_blocked_tags: GameplayTagContainer,      // 目标阻止标签
    pub block_abilities_with_tags: GameplayTagContainer, // 激活时阻塞其他技能的标签
    pub cancel_abilities_with_tags: GameplayTagContainer, // 激活时取消其他技能的标签
    pub triggers: Vec<AbilityTriggerData>,     // 触发器配置
    pub behavior: Option<Arc<dyn AbilityBehavior>>,     // 自定义行为
    pub default_blocks_other_abilities: bool,
    pub default_is_cancelable: bool,
}
```

使用 Builder 模式构建：

```rust
let fireball = AbilityDefinition::new("fireball")
    .with_cost_effect("mana_cost_30")
    .with_cooldown_effect("cooldown_5s")
    .with_behavior(Arc::new(FireballBehavior))
    .add_activation_required_tag(GameplayTag::new("State.Alive"), &tags_manager)
    .add_activation_blocked_tag(GameplayTag::new("State.Stunned"), &tags_manager)
    .add_ability_tag(GameplayTag::new("Ability.Casting"), &tags_manager)
    .add_block_abilities_with_tag(GameplayTag::new("Ability.Casting"), &tags_manager);
```

### AbilityRegistry

```rust
#[derive(Resource, Default)]
pub struct AbilityRegistry {
    pub definitions: HashMap<Atom, AbilityDefinition>,
}
```

## 技能生命周期

### 激活流程

```
TryActivateAbilityEvent
    │
    ▼
on_try_activate_ability (Observer)
    ├─ 查找 AbilitySpec + AbilityDefinition
    ├─ 调用 behavior.can_activate() 检查
    │   ├─ 冷却检查 (cooldown_effect 的 granted_tags)
    │   ├─ 来源所需标签检查
    │   └─ 来源阻止标签检查
    ├─ 失败 → 触发 AbilityActivationFailedEvent
    └─ 成功 → 插入 PendingActivation 标记组件
           │
           ▼
execute_pending_activations_system (Exclusive System)
    ├─ 查找所有带 PendingActivation 的 AbilitySpec
    ├─ 生成 AbilitySpecInstance 子实体
    ├─ 递增 AbilityActiveState
    ├─ 调用 behavior.pre_activate()
    ├─ 调用 behavior.activate()
    ├─ 触发 CommitAbilityEvent
    ├─ 触发 AbilityActivatedEvent
    └─ 移除 PendingActivation 标记
           │
           ▼
on_commit_ability (Observer)
    ├─ 调用 behavior.commit_check() (冷却二次检查)
    ├─ 调用 behavior.commit_execute()
    │   ├─ 应用冷却效果 (ApplyGameplayEffectEvent)
    │   └─ 应用消耗效果 (ApplyGameplayEffectEvent)
    └─ 触发 CommitAbilityResultEvent (success/failure)
```

**关键设计**: 激活分为两阶段 — Observer 做轻量检查并标记，Exclusive System 执行实际生成。这是因为 Observer 中无法获得 `&mut World` 来生成实体和调用 behavior 方法。

### 结束流程

```
EndAbilityEvent / CancelAbilityEvent
    │
    ▼
on_end_ability / on_cancel_ability (Observer)
    │
    ▼
end_ability_internal()
    ├─ 定位目标实例（指定实例或所有实例）
    ├─ 取消检查: is_active? (取消时还检查 is_cancelable)
    ├─ 调用 behavior.end()
    │   └─ 触发 OnGameplayAbilityEnded 实体事件
    ├─ 从所有者移除 activation_owned_tags (-1 计数)
    ├─ 从所有者移除 block_abilities_with_tags (-1 计数)
    ├─ 销毁实例实体
    └─ 递减 AbilityActiveState
```

### 实例清理 (安全网)

```rust
// Observer: 当 AbilitySpecInstance 组件被移除时
fn on_instance_removed(ev: On<Remove, AbilitySpecInstance>, ...) {
    // 调用 behavior.end() 确保清理逻辑执行
}
```

当父 AbilitySpec 被直接销毁时，Bevy 层级系统自动销毁子实例，此 Observer 确保 behavior.end() 仍被调用。

## AbilityBehavior Trait

自定义技能逻辑的扩展点：

```rust
pub trait AbilityBehavior: Send + Sync + 'static {
    /// 激活前检查（冷却、标签条件）
    fn can_activate(&self, world, ability_entity, source, tags_manager) -> ActivationCheckResult;

    /// 激活前准备（&mut World 访问）
    fn pre_activate(&self, world, instance_entity, spec_entity, source);

    /// 主要技能逻辑
    fn activate(&self, world, instance_entity, spec_entity, source, target);

    /// 提交检查（冷却二次验证）
    fn commit_check(&self, world, definition, source, tags_manager) -> ActivationCheckResult;

    /// 提交执行（应用消耗和冷却效果）
    fn commit(&self, world, commands, definition, spec, source, tags_manager) -> ActivationCheckResult;

    /// 提交执行内部逻辑
    fn commit_execute(&self, commands, definition, spec, source);

    /// 结束清理
    fn end(&self, commands, instance_entity, was_cancelled);
}
```

所有方法都有默认实现。`DefaultAbilityBehavior` 是零大小类型，直接使用 trait 默认实现。

### 生命周期顺序

```
can_activate → pre_activate → activate → commit (commit_check + commit_execute) → end
```

## 事件系统

### 输入事件

| 事件 | 用途 |
|------|------|
| `TryActivateAbilityEvent` | 请求激活技能 |
| `CommitAbilityEvent` | 提交技能（应用消耗和冷却） |
| `EndAbilityEvent` | 正常结束技能 |
| `CancelAbilityEvent` | 取消技能 |

### 输出事件

| 事件 | 用途 |
|------|------|
| `AbilityActivatedEvent` | 技能成功激活 |
| `AbilityActivationFailedEvent` | 技能激活失败（附带原因） |
| `CommitAbilityResultEvent` | 提交结果（成功/失败） |
| `OnGameplayAbilityEnded` | 实例结束（EntityEvent） |

### 激活失败原因

```rust
pub enum ActivationFailureReason {
    OnCooldown,          // 冷却中
    InsufficientCost,    // 资源不足
    MissingRequiredTags, // 缺少前置标签
    BlockedByTags,       // 被标签阻止
}
```

## Tag 条件系统

技能系统通过 GameplayTag 实现丰富的条件控制：

| Tag 类型 | 检查时机 | 作用 |
|----------|---------|------|
| `activation_required_tags` | 激活时 | 所有者必须拥有这些标签 |
| `activation_blocked_tags` | 激活时 | 所有者拥有这些标签则阻止激活 |
| `source_required_tags` | 激活时 | 来源必须拥有（can_activate 中检查） |
| `source_blocked_tags` | 激活时 | 来源拥有则阻止 |
| `target_required_tags` | 激活时 | 目标必须拥有 |
| `target_blocked_tags` | 激活时 | 目标拥有则阻止 |
| `activation_owned_tags` | 激活/结束时 | 技能激活时添加到所有者，结束时移除 |
| `block_abilities_with_tags` | 激活/结束时 | 激活时添加阻塞标签，结束时移除 |
| `cancel_abilities_with_tags` | 激活时 | 激活时取消拥有这些标签的其他技能 |

Tag 计数使用 `GameplayTagCountContainer`，支持多个技能同时添加同一标签（引用计数），结束时用 -1 递减而非直接移除。

## 系统执行顺序

所有系统在 `Update` 阶段运行，使用 SystemSet 排序：

```
GasSystemSet::Abilities
    └─ execute_pending_activations_system (Exclusive)
```

Observer（on_try_activate_ability, on_commit_ability, on_end_ability, on_cancel_ability）由事件触发，不受 SystemSet 排序约束。技能模块只有一个 Exclusive System 需要调度，不需要额外的子集划分。

## 使用指南

### 步骤 1: 添加插件

```rust
App::new()
    .add_plugins(AbilityPlugin)
    .run();
```

### 步骤 2: 定义并注册技能

```rust
fn setup(
    mut ability_registry: ResMut<AbilityRegistry>,
    tags_manager: Res<GameplayTagsManager>,
) {
    let fireball = AbilityDefinition::new("fireball")
        .with_cost_effect("mana_cost_30")
        .with_cooldown_effect("cooldown_fireball")
        .with_behavior(Arc::new(FireballBehavior))
        .add_activation_required_tag(GameplayTag::new("State.Alive"), &tags_manager)
        .add_activation_blocked_tag(GameplayTag::new("State.Stunned"), &tags_manager);

    ability_registry.register(fireball);
}
```

### 步骤 3: 授予技能

```rust
fn grant_ability(mut commands: Commands, player: Entity) {
    let spec_entity = commands
        .spawn((
            AbilitySpec::new("fireball", 1),
            AbilityActiveState::default(),
            AbilityOwner(player),
        ))
        .set_parent_in_place(player)
        .id();
}
```

### 步骤 4: 激活技能

```rust
fn activate_ability(mut commands: Commands, player: Entity, spec_entity: Entity) {
    commands.trigger(TryActivateAbilityEvent {
        ability_spec: spec_entity,
        owner: player,
    });
}
```

### 步骤 5: 结束/取消技能

```rust
// 正常结束（所有实例）
commands.trigger(EndAbilityEvent {
    instance: None,
    ability_spec: spec_entity,
    owner: player,
});

// 取消特定实例
commands.trigger(CancelAbilityEvent {
    instance: Some(instance_entity),
    ability_spec: spec_entity,
    owner: player,
});
```

## 自定义 Behavior 示例

```rust
struct FireballBehavior;

impl AbilityBehavior for FireballBehavior {
    fn activate(
        &self,
        world: &mut World,
        instance_entity: Entity,
        spec_entity: Entity,
        source: Entity,
        target: Option<Entity>,
    ) {
        // 生成火球投射物、播放动画等
    }

    fn end(&self, commands: &mut Commands, instance_entity: Entity, was_cancelled: bool) {
        if was_cancelled {
            // 取消特殊处理
        }
        // 调用默认的 end 触发 OnGameplayAbilityEnded
        commands.trigger(OnGameplayAbilityEnded {
            ability_instance: instance_entity,
            was_cancelled,
        });
    }
}
```

## 与其他模块的集成

### 与 Effects 模块

技能通过 `ApplyGameplayEffectEvent` 与效果系统交互：
- **冷却**: `cooldown_effect` 指定一个 HasDuration 效果，其 `granted_tags` 在持续期间存在，阻止技能再次激活
- **消耗**: `cost_effect` 指定一个 Instant 效果，通过修改器扣除属性值

### 与 Attributes 模块

技能系统通过效果系统间接操作属性：

**架构概览**:
- 每个属性是独立实体，通过 `ChildOf` 关系链接到所有者
- 属性有双值模型: `base_value`(永久值) 和 `current_value`(应用修改器后的值)
- 属性集通过 `AttributeSetDefinition` trait 定义，支持自定义生命周期钩子

**技能如何消耗资源**:
1. 技能定义中指定 `cost_effect`(Instant 效果)
2. Commit 阶段应用该效果，修改器扣除属性的 `base_value`
3. Aggregation 系统自动重新计算 `current_value`

**生命周期钩子集成**:
- `pre_effect_execute`: 在 Instant 效果修改 base_value 前调用，可拒绝修改
- `post_effect_execute`: 在修改后调用，可触发事件(如资源不足时的反馈)
- `pre_change`/`post_change`: 在 current_value 变化时调用(包括 aggregation)
- `pre_base_change`/`post_base_change`: 在 base_value 永久变化时调用(如升级)

技能系统本身不直接操作属性，所有修改都通过 `ApplyGameplayEffectEvent` 路由到效果系统。

### 与 Tags 系统

技能系统是 Tag 系统的重度用户。`GameplayTagCountContainer` 上的引用计数确保多个技能同时添加/移除同一标签不会冲突。

## 触发器系统

技能可以通过触发器自动激活，无需手动调用 `TryActivateAbilityEvent`。

### 触发源类型

```rust
pub enum AbilityTriggerSource {
    /// 由外部游戏事件触发
    GameplayEvent,
    
    /// 所有者获得指定标签时触发
    /// 标签移除时不会取消技能
    OwnedTagAdded,
    
    /// 所有者拥有指定标签时触发
    /// 标签移除时会取消技能
    OwnedTagPresent,
}
```

### 配置触发器

```rust
let passive_ability = AbilityDefinition::new("passive_regen")
    .add_trigger(AbilityTriggerData::owned_tag_present(
        GameplayTag::new("State.InCombat")
    ))
    .with_behavior(Arc::new(RegenBehavior));

// 授予技能时，触发器会自动监听
commands.spawn((
    AbilitySpec::new("passive_regen", 1),
    AbilityActiveState::default(),
    AbilityOwner(player),
    AbilityTriggers {
        triggers: passive_ability.triggers.clone(),
    },
));
```

### 触发器系统工作流程

1. **OwnedTagAdded**: 监听 `OwnedTags` 的变化，标签添加时触发激活
2. **OwnedTagPresent**: 标签存在时激活，标签移除时取消
3. **GameplayEvent**: 监听 `GameplayEvent`，匹配 `trigger_tag` 时激活

## 任务系统

任务（Tasks）是技能内的异步操作，作为实例的子实体存在。

### 任务生命周期

```
AbilitySpecInstance Entity
    └─ AbilityTask Entity (子实体)
          ├─ AbilityTask (ability_instance, ability_spec, owner)
          ├─ TaskState (Running/Completed/Cancelled)
          └─ 具体任务类型组件
```

任务在实例结束时自动清理（通过 Bevy 的层级系统）。

### 内置任务类型

#### 1. WaitDelayTask — 等待延迟
```rust
commands.spawn((
    AbilityTask {
        ability_instance: Some(instance),
        ability_spec: spec,
        owner: player,
    },
    TaskState::Running,
    WaitDelayTask::new(2.0),  // 等待 2 秒
)).set_parent_in_place(instance);
```

#### 2. WaitGameplayEventTask — 等待游戏事件
```rust
commands.spawn((
    AbilityTask { /* ... */ },
    TaskState::Running,
    WaitGameplayEventTask::new(GameplayTag::new("Event.Hit"))
        .with_only_trigger_once(true),
)).set_parent_in_place(instance);
```

#### 3. WaitAttributeChangeTask — 等待属性变化
```rust
commands.spawn((
    AbilityTask { /* ... */ },
    TaskState::Running,
    WaitAttributeChangeTask::new(
        "Health",
        AttributeComparison::LessThan,
        50.0,
    ),
)).set_parent_in_place(instance);
```

#### 4. WaitGameplayTagAddedTask — 等待标签添加
```rust
commands.spawn((
    AbilityTask { /* ... */ },
    TaskState::Running,
    WaitGameplayTagAddedTask::new(GameplayTag::new("State.Stunned")),
)).set_parent_in_place(instance);
```

#### 5. WaitGameplayTagRemovedTask — 等待标签移除
```rust
commands.spawn((
    AbilityTask { /* ... */ },
    TaskState::Running,
    WaitGameplayTagRemovedTask::new(GameplayTag::new("State.Invincible")),
)).set_parent_in_place(instance);
```

#### 6. ApplyRootMotionTask — 应用根运动
```rust
commands.spawn((
    AbilityTask { /* ... */ },
    TaskState::Running,
    ApplyRootMotionTask::new(Vec3::new(0.0, 0.0, 10.0), 1.0),
)).set_parent_in_place(instance);
```

#### 7. PlayMontageTask — 播放动画蒙太奇
```rust
commands.spawn((
    AbilityTask { /* ... */ },
    TaskState::Running,
    PlayMontageTask::new("attack_montage".into()),
)).set_parent_in_place(instance);
```

#### 8. WaitGameplayEffectAppliedTask — 等待效果应用
```rust
commands.spawn((
    AbilityTask { /* ... */ },
    TaskState::Running,
    WaitGameplayEffectAppliedTask::new("poison".into()),
)).set_parent_in_place(instance);
```

#### 9. WaitGameplayEffectRemovedTask — 等待效果移除
```rust
commands.spawn((
    AbilityTask { /* ... */ },
    TaskState::Running,
    WaitGameplayEffectRemovedTask::new("shield".into()),
)).set_parent_in_place(instance);
```

#### 10. SpawnProjectileTask — 生成投射物
```rust
commands.spawn((
    AbilityTask { /* ... */ },
    TaskState::Running,
    SpawnProjectileTask::new(
        Vec3::ZERO,
        Vec3::new(0.0, 0.0, 1.0),
        10.0,
    ),
)).set_parent_in_place(instance);
```

#### 11. RepeatTask — 重复执行
```rust
commands.spawn((
    AbilityTask { /* ... */ },
    TaskState::Running,
    RepeatTask::new(5, 1.0),  // 重复 5 次，间隔 1 秒
)).set_parent_in_place(instance);
```

#### 12. WaitInputPressTask — 等待输入按下
```rust
commands.spawn((
    AbilityTask { /* ... */ },
    TaskState::Running,
    WaitInputPressTask::new(0),  // 等待输入 ID 0
)).set_parent_in_place(instance);
```

### 任务完成事件

任务完成时触发 `TaskCompletedEvent`：

```rust
#[derive(Event)]
pub struct TaskCompletedEvent {
    pub task_entity: Entity,
    pub ability_instance: Option<Entity>,
    pub owner: Entity,
}
```

### 任务系统的优势

1. **异步操作**: 技能可以等待外部事件而不阻塞主逻辑
2. **组合性**: 多个任务可以并行或串行执行
3. **自动清理**: 实例结束时任务自动清理
4. **类型安全**: 每种任务是独立的组件类型
5. **ECS 原生**: 利用 Bevy 的查询和系统处理任务

## 设计决策

### 为什么用 Exclusive System 执行激活？

Observer 只能获得 `&World`（只读）或有限的可变访问。生成实例实体和调用 `behavior.activate(&mut World, ...)` 需要完整的 `&mut World` 访问，因此用 Exclusive System 处理实际激活。Observer 负责轻量检查并标记 `PendingActivation`。

### 为什么 Behavior 用 `Arc<dyn AbilityBehavior>`？

- 定义存储在 Registry（Resource）中，实例需要 behavior 的引用
- `Arc` 允许多个实例共享同一 behavior 而不需要 Clone trait bound
- `dyn` 允许不同技能有不同的 behavior 实现
- `Send + Sync` bound 确保跨线程安全

### 为什么 AbilityActiveState 与 AbilitySpec 分离？

Bevy 的变更检测按组件粒度工作。分离后，`Changed<AbilityActiveState>` 只在激活状态变化时触发，不会因为 spec 的其他字段变化而误触。

## 性能考虑

1. **Interned Strings**: `definition_id` 使用 `string_cache::Atom`，HashMap 查找高效
2. **实体化设计**: 每个实例是独立实体，Bevy 可以并行处理
3. **Observer 模式**: 事件驱动，不需要每帧轮询所有技能
4. **延迟执行**: PendingActivation 标记 + Exclusive System 避免 Observer 中的 World 借用冲突
