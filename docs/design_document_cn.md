# Bevy GameplayAbilitySystem 设计文档

## 1. 概述

本文档详细说明了如何使用 Bevy ECS 架构完整实现虚幻引擎的 GameplayAbilitySystem (GAS)。目标是在保持 UE GAS 外部功能一致的前提下，将其 OOP 架构转换为符合 Bevy ECS 习惯的模式。这是一个单机版实现，省略了网络同步/复制/预测功能。

**目标兼容性**: Bevy 0.18, Rust Edition 2024  
**标签系统**: `bevy_gameplay_tag` 0.2.0 (已实现)  
**架构**: 纯 ECS，采用 entity-per-thing 模式

---

## 2. 架构总览

### 2.1 OOP 到 ECS 的转换策略

| UE GAS 概念 | UE 实现方式 | Bevy ECS 实现方式 |
|-------------|-------------|-------------------|
| **AbilitySystemComponent** | Actor 上的单一组件，拥有所有数据 | 拆分为多个组件 + 资源 |
| **AttributeSet** | 带属性的子对象 | 每个属性是独立实体，通过 `ChildOf` 关联 |
| **ActiveGameplayEffect** | 容器数组中的结构体 | 带组件的实体 |
| **GameplayAbilitySpec** | 数组中的结构体 | 带组件的实体 |
| **GameplayEffect (资产)** | UObject 类定义 | 注册表资源中的 `GameplayEffectDefinition` |
| **GameplayAbility (资产)** | UObject 类定义 | 注册表资源中的 `AbilityDefinition` |
| **GameplayCueNotify** | Actor 或静态接口 | 基于 trait 的处理器 + 可选生成实体 |

### 2.2 核心模块

```
bevy_gameplay_ability_system/
├── attributes/      # 属性存储、聚合、钩子
├── effects/         # 效果应用、修改器、堆叠
├── abilities/       # 技能激活、消耗、冷却
├── cues/           # 视觉/音频反馈路由
├── core/           # 共享类型、系统集、事件
└── utils/          # 查询辅助、数学工具
```

---

## 3. 模块设计

### 3.1 属性模块 (Attributes)

**状态**: ✅ 已实现（存在已知问题）

#### 组件定义
```rust
#[derive(Component)]
pub struct AttributeData {
    pub base_value: f32,      // 基础值（永久修改）
    pub current_value: f32,   // 当前值（包含临时修改）
    pub attribute_name: String,
}

#[derive(Component)]
pub struct AttributeOwner(pub Entity);  // 指向拥有者实体
```

#### 架构说明
- 每个属性是一个**独立实体**，带有 `AttributeData` + `ChildOf<owner>`
- 拥有者实体没有特殊组件（通过拥有属性子实体来识别）
- 自定义属性集实现 `AttributeSetDefinition` trait
- 修改器按顺序聚合：`AddBase → MultiplyAdditive → DivideAdditive → MultiplyCompound → AddFinal`（Override 跳过所有）

#### 与 UE 的功能对比

| UE 功能 | 当前状态 | 需要的操作 |
|---------|----------|-----------|
| `FGameplayAttributeData` 双值模型 | ✅ 已实现 | 修复 `set_base_value()` bug（会覆盖 current） |
| 属性钩子 (Pre/PostGameplayEffectExecute) | ❌ 缺失 | 添加 `AttributeSetHooks` trait 和回调 |
| 属性限制 (clamping) | ✅ 已实现 | 正常工作 |
| 属性变化事件 | ✅ 已实现 | 正常工作 |
| `ATTRIBUTE_ACCESSORS` 宏 | ❌ 不需要 | Rust 不需要这种模式 |

**实现计划**:
1. 添加 `AttributeSetHooks` trait，包含 `pre_effect_execute()` 和 `post_effect_execute()` 方法
2. 修复 `set_base_value()` 只修改 base，触发重新聚合
3. 添加 `AttributeSnapshot` 组件用于在效果应用期间捕获值

---

### 3.2 效果模块 (Effects)

**状态**: ✅ 部分实现（缺失功能）

#### 组件定义
```rust
#[derive(Component)]
pub struct ActiveGameplayEffect {
    pub definition_id: String,    // 效果定义 ID
    pub start_time: f64,          // 开始时间
    pub duration: Option<f32>,    // 持续时间
    pub period: Option<f32>,      // 周期执行间隔
    pub stack_count: u32,         // 堆叠层数
}

#[derive(Component)]
pub struct EffectTarget(pub Entity);  // 效果目标

#[derive(Component)]
pub struct GameplayEffectModifier {
    pub attribute_name: String,
    pub operation: ModifierOperation,
    pub magnitude: f32,
}
```

#### 定义结构
```rust
pub struct GameplayEffectDefinition {
    pub id: String,
    pub duration_policy: DurationPolicy,        // 持续策略
    pub duration: Option<f32>,
    pub period: Option<f32>,
    pub modifiers: Vec<ModifierDefinition>,     // 修改器列表
    pub stacking_policy: StackingPolicy,        // 堆叠策略
    
    // 标签需求（采用 UE 5.3+ 组件化设计）
    pub application_requirement: TagRequirement, // 应用要求
    pub ongoing_requirement: TagRequirement,     // 持续要求
    pub removal_requirement: TagRequirement,     // 移除要求
    pub granted_tags: GameplayTagContainer,      // 授予的标签
    pub blocked_tags: GameplayTagContainer,      // 阻止的标签
    
    // 高级功能（采用 UE 5.3+ 组件化设计）
    pub chance_to_apply: Option<f32>,           // 应用概率（替代旧版 ChanceToApplyToTarget）
    pub granted_abilities: Vec<String>,         // 授予的技能 ID（替代旧版 GrantedAbilitySpecs）
    pub conditional_effects: Vec<ConditionalEffect>, // 条件效果（替代旧版 ConditionalGameplayEffects）
    pub on_expire_effects: Vec<String>,         // 过期时应用的效果（替代旧版 RoutineExpirationEffectClasses）
    pub on_remove_effects: Vec<String>,         // 移除时应用的效果（替代旧版 PrematureExpirationEffectClasses）
}
```

#### 与 UE 的功能对比

| UE 功能 | 当前状态 | 需要的操作 |
|---------|----------|-----------|
| 持续策略 (Instant/HasDuration/Infinite) | ✅ 已实现 | 正常工作 |
| 修改器操作 (Add/Multiply/Override) | ⚠️ 部分 | `AddBase` 在聚合中被跳过，需修复 |
| 周期执行 | ⚠️ 损坏 | `execute_periodic_effects_system` 中有 TODO |
| 堆叠 (Independent/RefreshDuration/StackCount) | ⚠️ 损坏 | `StackCount` 策略生成重复项但不清理 |
| 幅度计算类型 | ❌ 缺失 | 仅实现了 ScalableFloat |
| 自定义计算类 | ❌ 缺失 | 需要基于 trait 的系统 |
| SetByCaller 幅度 | ❌ 缺失 | 需要在 spec 上添加 `HashMap<GameplayTag, f32>` |
| 基于属性的幅度 | ❌ 缺失 | 需要属性捕获系统 |
| 条件效果（免疫） | ❌ 缺失 | 需要免疫标签检查 |
| 效果上下文（来源/目标信息） | ❌ 缺失 | 需要 `GameplayEffectContext` 组件 |
| 授予技能 | ❌ 缺失 | 效果可以授予临时技能 |

**实现计划**:

**阶段 1: 修复现有系统**
- 修复聚合中的 `AddBase` 操作
- 实现周期执行（在 tick 时应用修改器）
- 修复 `StackCount` 策略（堆叠时移除旧修改器）
- 修复瞬时效果的标签泄漏

**阶段 2: 幅度计算**
- 添加 `MagnitudeCalculation` 枚举 (ScalableFloat/AttributeBased/CustomClass/SetByCaller)
- 实现 `AttributeBasedFloat`，包含系数/预乘/后乘
- 添加 `CustomCalculation` trait 用于用户自定义计算
- 在效果 spec 上添加 `SetByCallerMagnitudes` 组件

**阶段 3: 高级功能**
- 添加 `GameplayEffectContext` 组件（来源实体、发起者、命中结果）
- 实现免疫系统（应用前检查阻止标签）
- 添加授予技能（效果应用时生成技能 spec，移除时删除）
- 添加条件应用（自定义需求类）

---

### 3.3 技能模块 (Abilities)

**状态**: ✅ 部分实现（缺失功能）

#### 组件定义
```rust
#[derive(Component)]
pub struct AbilitySpec {
    pub definition_id: String,
    pub level: u32,
    pub input_id: Option<u32>,
    pub activation_state: ActivationState,
}

#[derive(Component)]
pub struct AbilityOwner(pub Entity);

#[derive(Component)]
pub struct AbilityActivationTags {
    pub activation_owned_tags: GameplayTagContainer,      // 激活时拥有的标签
    pub activation_required_tags: GameplayTagContainer,   // 激活所需标签
    pub activation_blocked_tags: GameplayTagContainer,    // 阻止激活的标签
}

#[derive(Component)]
pub struct AbilityBlockingTags {
    pub block_abilities_with_tag: GameplayTagContainer,   // 阻止带此标签的技能
    pub cancel_abilities_with_tag: GameplayTagContainer,  // 取消带此标签的技能
}
```

#### 定义结构
```rust
pub struct AbilityDefinition {
    pub id: String,
    pub ability_tags: GameplayTagContainer,
    pub activation_owned_tags: GameplayTagContainer,
    pub activation_required_tags: GameplayTagContainer,
    pub activation_blocked_tags: GameplayTagContainer,
    pub block_abilities_with_tag: GameplayTagContainer,
    pub cancel_abilities_with_tag: GameplayTagContainer,
    pub cooldown_effect: Option<String>,  // 冷却效果 ID
    pub cost_effect: Option<String>,      // 消耗效果 ID
}
```

#### 激活流程
```
TryActivate → CheckCanActivate → Commit (消耗/冷却) → Activate → End/Cancel
```

#### 与 UE 的功能对比

| UE 功能 | 当前状态 | 需要的操作 |
|---------|----------|-----------|
| 激活生命周期 | ✅ 已实现 | 正常工作 |
| 基于标签的门控 | ✅ 已实现 | 正常工作 |
| 通过效果实现消耗/冷却 | ✅ 已实现 | 正常工作 |
| 实例化策略 | ❌ 缺失 | 需要 NonInstanced/InstancedPerActor/InstancedPerExecution |
| 技能触发器（自动激活） | ❌ 缺失 | 需要标签事件触发系统 |
| 技能任务 | ❌ 缺失 | 需要异步任务集成 |
| 目标数据 | ❌ 缺失 | 需要目标选择系统 |
| 动画蒙太奇集成 | ❌ 缺失 | 需要 Bevy 特定的动画集成 |
| 技能批处理 | ❌ 缺失 | 一帧内多个技能 |

**实现计划**:

**阶段 1: 实例化策略**
- 在定义中添加 `InstancingPolicy` 枚举
- NonInstanced: 逻辑存储在定义中，无每次激活状态
- InstancedPerActor: 跨激活重用同一 spec 实体
- InstancedPerExecution: 每次激活生成新 spec 实体（当前行为）

**阶段 2: 触发器**
- 添加 `AbilityTriggerData` 组件（触发标签、触发源）
- 实现标签添加/移除时的自动激活
- 实现游戏事件时的自动激活

**阶段 3: 任务与目标选择**
- 添加 `AbilityTask` trait 用于异步操作
- 实现常见任务（WaitDelay, WaitGameplayEvent, WaitTargetData）
- 添加目标选择系统（射线检测、半径、自定义过滤器）

---

### 3.4 提示模块 (Cues)

**状态**: ✅ 已实现（基础功能）

#### 架构说明
- `GameplayCueManager` 资源将事件路由到处理器
- 静态处理器：实现 `GameplayCueHandler` trait
- Actor 处理器：生成带 `GameplayCueNotify` 组件的实体
- 层级标签匹配（例如 `GameplayCue.Damage.Fire` 匹配 `GameplayCue.Damage`）

#### 与 UE 的功能对比

| UE 功能 | 当前状态 | 需要的操作 |
|---------|----------|-----------|
| 事件类型 (OnActive/WhileActive/Executed/Removed) | ✅ 已实现 | 正常工作 |
| 静态处理器 | ✅ 已实现 | 正常工作 |
| Actor 处理器 | ✅ 已实现 | 正常工作 |
| 层级路由 | ✅ 已实现 | 正常工作 |
| 异步加载 | ❌ 缺失 | Bevy 资产加载集成 |
| 提示参数 | ❌ 缺失 | 需要幅度/位置/法线数据 |
| 提示转换 | ❌ 缺失 | 路由前的标签重映射 |

**实现计划**:
1. 添加 `GameplayCueParameters` 结构体（幅度、位置、法线、发起者）
2. 添加提示转换系统（路由前重映射标签）
3. 与 Bevy 资产加载集成，实现异步处理器生成

---

## 4. 系统执行顺序

### 4.1 当前顺序
```
GasSystemSet::Input
  └─ (用户输入处理)

GasSystemSet::Attributes
  ├─ AttributeSystemSet::Clamp
  └─ AttributeSystemSet::Events

GasSystemSet::Effects
  ├─ EffectSystemSet::Apply
  ├─ EffectSystemSet::CreateModifiers
  ├─ EffectSystemSet::Aggregate
  ├─ EffectSystemSet::UpdateDurations
  ├─ EffectSystemSet::ExecutePeriodic
  ├─ EffectSystemSet::RemoveExpired
  └─ EffectSystemSet::RemoveInstant

GasSystemSet::Abilities
  └─ execute_pending_activations_system (独占)

GasSystemSet::Cues
  ├─ CueSystemSet::Handle
  ├─ CueSystemSet::Route
  ├─ CueSystemSet::ExecuteStatic
  ├─ CueSystemSet::ManageActors
  ├─ CueSystemSet::Cleanup
  └─ CueSystemSet::UpdateWhileActive

GasSystemSet::Cleanup
  └─ (销毁标记的实体)
```

### 4.2 建议的改进
- 添加 `GasSystemSet::PreUpdate` 用于属性快照（效果应用前）
- 添加 `GasSystemSet::PostUpdate` 用于属性钩子（聚合后）
- 将 `Abilities` 拆分为子集（CheckActivation, Commit, Execute, End）

---

## 5. 关键设计模式

### 5.1 Entity-Per-Thing（每个事物一个实体）
- **属性**: 每个属性是独立实体（不是拥有者上的 Vec）
- **效果**: 每个活动效果是独立实体
- **技能**: 每个授予的技能是独立实体
- **理由**: 启用 Bevy 的查询优化和并行执行

### 5.2 定义/注册表模式
- **定义**: 不可变模板（类似 UE 资产）
- **注册表**: 持有 `HashMap<String, Definition>` 的资源
- **Spec**: 作为实体生成的运行时实例
- **理由**: 分离数据（定义）和状态（spec）

### 5.3 观察者模式
- **事件**: `TryActivateAbilityEvent`, `ApplyGameplayEffectEvent` 等
- **观察者**: 在插件中注册，事件触发时执行
- **理由**: 解耦事件发射和处理，支持用户钩子

### 5.4 构建器模式
- **定义**: 流式 API 构造
- **示例**: `GameplayEffectDefinition::new("heal").with_duration(5.0).add_modifier(...)`
- **理由**: 人性化 API，编译时验证

### 5.5 SystemParam 捆绑
- **复杂查询**: 分组到 `#[derive(SystemParam)]` 结构体
- **示例**: `ActivationCheckParams`, `ApplyEffectParams`
- **理由**: 减少样板代码，提高可读性

---

## 6. 实现路线图

### 阶段 1: 修复关键 Bug（第 1 周）
- [ ] 修复 `AttributeData::set_base_value()` 覆盖 current value
- [ ] 修复瞬时效果标签泄漏
- [ ] 修复周期效果执行（实现修改器应用）
- [ ] 修复聚合中的 `AddBase` 操作
- [ ] 修复 `StackCount` 策略（清理旧修改器）

### 阶段 2: 属性钩子（第 2 周）
- [ ] 添加 `AttributeSetHooks` trait
- [ ] 实现 `pre_effect_execute()` 回调
- [ ] 实现 `post_effect_execute()` 回调
- [ ] 添加 `AttributeSnapshot` 组件用于捕获值
- [ ] 添加钩子注册系统

### 阶段 3: 幅度计算（第 3 周）
- [ ] 添加 `MagnitudeCalculation` 枚举
- [ ] 实现 `AttributeBasedFloat` 计算
- [ ] 添加 `CustomCalculation` trait
- [ ] 实现 `SetByCaller` 幅度
- [ ] 添加幅度评估系统
- [ ] **注意**: 不实现 UE 5.3 前的单体设计，直接采用组件化

### 阶段 4: 效果上下文与免疫（第 4 周）
- [ ] 添加 `GameplayEffectContext` 组件
- [ ] 实现免疫检查（阻止标签）
- [ ] 添加条件应用（使用 `ApplicationRequirement` trait，不实现 UE 旧版 `ApplicationRequirement` 属性）
- [ ] 添加效果授予技能（使用 `granted_abilities` 字段，不实现 UE 旧版 `GrantedAbilitySpecs`）
- [ ] 添加 `ChanceToApply` 组件（替代 UE 旧版 `ChanceToApplyToTarget` 属性）

### 阶段 5: 技能实例化（第 5 周）
- [ ] 添加 `InstancingPolicy` 枚举到定义
- [ ] 实现 NonInstanced 技能
- [ ] 实现 InstancedPerActor 技能
- [ ] 重构 InstancedPerExecution（当前默认）

### 阶段 6: 技能触发器（第 6 周）
- [ ] 添加 `AbilityTriggerData` 组件
- [ ] 实现标签事件的自动激活
- [ ] 实现游戏事件的自动激活
- [ ] 添加触发器优先级系统

### 阶段 7: 任务与目标选择（第 7-8 周）
- [ ] 添加 `AbilityTask` trait
- [ ] 实现 WaitDelay 任务
- [ ] 实现 WaitGameplayEvent 任务
- [ ] 实现 WaitTargetData 任务
- [ ] 添加目标选择系统（射线检测、半径、过滤器）

### 阶段 8: 提示增强（第 9 周）
- [ ] 添加 `GameplayCueParameters` 结构体
- [ ] 实现提示转换系统
- [ ] 集成 Bevy 资产加载用于处理器
- [ ] 添加提示批处理以提高性能

### 阶段 9: 优化与完善（第 10 周）
- [ ] 用 `Atom` (string_cache) 替换字符串 ID
- [ ] 实现句柄生成跟踪
- [ ] 添加全面的示例
- [ ] 编写集成测试
- [ ] 性能基准测试

---

## 7. API 示例

### 7.1 定义属性集
```rust
#[derive(Component)]
pub struct CharacterAttributes;

impl AttributeSetDefinition for CharacterAttributes {
    fn attributes() -> Vec<AttributeDefinition> {
        vec![
            AttributeDefinition::new("Health", 100.0, Some(0.0), Some(100.0)),
            AttributeDefinition::new("Mana", 50.0, Some(0.0), Some(50.0)),
            AttributeDefinition::new("Stamina", 100.0, Some(0.0), Some(100.0)),
        ]
    }
}

// 生成带属性的角色
commands.spawn_empty().with_children(|parent| {
    CharacterAttributes::spawn_attributes(parent);
});
```

### 7.2 定义游戏效果
```rust
// 采用 UE 5.3+ 组件化设计
let heal_effect = GameplayEffectDefinition::new("heal_over_time")
    .with_duration(5.0)
    .with_period(1.0)
    .add_modifier(
        "Health",
        ModifierOperation::Add,
        MagnitudeCalculation::ScalableFloat(10.0),
    )
    .with_granted_tags(&tags_manager, &["Effect.HealOverTime"])
    .with_chance_to_apply(0.8)  // 80% 概率应用（替代旧版 ChanceToApplyToTarget）
    .build();

effect_registry.register(heal_effect);

// 带条件效果的示例（替代旧版 ConditionalGameplayEffects）
let buff_effect = GameplayEffectDefinition::new("conditional_buff")
    .with_duration(10.0)
    .add_modifier("Attack", ModifierOperation::Multiply, MagnitudeCalculation::ScalableFloat(1.5))
    .add_conditional_effect(
        ConditionalEffect::new("extra_buff")
            .when_target_has_tags(&["State.Enraged"])
    )
    .with_on_expire_effects(&["debuff_after_buff"])  // 过期时应用虚弱效果
    .build();
```

### 7.3 定义技能
```rust
let fireball = AbilityDefinition::new("fireball")
    .with_ability_tags(&tags_manager, &["Ability.Fireball"])
    .with_activation_required_tags(&tags_manager, &["State.Alive"])
    .with_activation_blocked_tags(&tags_manager, &["State.Stunned"])
    .with_cooldown_effect("cooldown_fireball")
    .with_cost_effect("cost_fireball")
    .build();

ability_registry.register(fireball);
```

### 7.4 激活技能
```rust
// 授予技能给实体
commands.trigger_targets(
    GrantAbilityEvent {
        definition_id: "fireball".to_string(),
        level: 1,
    },
    player_entity,
);

// 激活技能
commands.trigger_targets(
    TryActivateAbilityEvent {
        definition_id: "fireball".to_string(),
    },
    player_entity,
);
```

---

## 8. 测试策略

### 8.1 单元测试
- 独立测试每个模块
- 使用 `App::new()` 配合最小插件
- 使用 `run_system_once` 进行依赖系统的测试

### 8.2 集成测试
- 测试完整生命周期（技能激活、效果应用）
- 使用 `TestEvents` 资源配合 `Arc<Mutex<Vec<T>>>` 捕获事件
- 测试跨模块交互

### 8.3 示例
- `basic_attributes`: 属性创建和修改
- `gameplay_effects`: 效果应用和堆叠
- `ability_activation`: 技能生命周期
- `complete_rpg`: 完整战斗模拟
- `stress_test`: 性能基准测试

---

## 9. 已知限制

### 9.1 有意省略的功能（UE 功能未实现）
- **网络/复制**: 仅单机
- **预测**: 无客户端预测
- **蓝图集成**: 仅 Rust API
- **编辑器工具**: 无可视化编辑器
- **动画蒙太奇**: 需要 Bevy 特定的动画集成

### 9.2 UE 已弃用功能（不复刻）

以下是 UE 5.3+ 已弃用的功能，我们将直接采用新的设计，不实现旧版本：

#### 9.2.1 GameplayEffect 组件化（UE 5.3）
**弃用内容**: UE 5.3 前的单体 `UGameplayEffect` 设计  
**新设计**: 模块化 `UGameplayEffectComponent` 系统  
**我们的做法**: 直接采用组件化设计，使用 Bevy 的组件系统

| UE 弃用功能 | 替代方案 | Bevy 实现 |
|------------|---------|----------|
| `ChanceToApplyToTarget` 属性 | `UChanceToApplyGameplayEffectComponent` | `ChanceToApply` 组件 |
| `ApplicationRequirement` 属性 | `UCustomCanApplyGameplayEffectComponent` | `ApplicationRequirement` trait |
| `ConditionalGameplayEffects` | `UAdditionalEffectsGameplayEffectComponent` | `ConditionalEffects` 组件 |
| `PrematureExpirationEffectClasses` | `UAdditionalEffectsGameplayEffectComponent` | `OnRemoveEffects` 组件 |
| `RoutineExpirationEffectClasses` | `UAdditionalEffectsGameplayEffectComponent` | `OnExpireEffects` 组件 |
| `InheritableGameplayEffectTags` | `UAssetTagsGameplayEffectComponent` | 直接在定义中使用 `asset_tags` |
| `InheritableOwnedTagsContainer` | `UTargetTagsGameplayEffectComponent` | 直接在定义中使用 `granted_tags` |
| `InheritableBlockedAbilityTagsContainer` | `UTargetTagsGameplayEffectComponent` | 直接在定义中使用 `blocked_tags` |
| `OngoingTagRequirements` | `UTargetTagRequirementsGameplayEffectComponent` | `ongoing_requirement` |
| `ApplicationTagRequirements` | `UTargetTagRequirementsGameplayEffectComponent` | `application_requirement` |
| `RemovalTagRequirements` | `URemoveOtherGameplayEffectComponent` | `removal_requirement` |
| `RemoveGameplayEffectsWithTags` | `UTargetTagRequirementsGameplayEffectComponent` | `remove_effects_with_tags` |

#### 9.2.2 链接的 GameplayEffect（UE 5.3）
**弃用原因**: 不复制，仅服务器端有效  
**我们的做法**: 不实现 `TargetEffectSpecs`，使用 `ConditionalEffects` 组件替代

#### 9.2.3 效果授予技能（UE 5.3）
**弃用内容**: `FGameplayEffectSpec::GrantedAbilitySpecs`  
**新设计**: 不可变的 `GASpecs` 存储在 `AbilitiesGameplayEffectComponent` 中  
**我们的做法**: 在 `GameplayEffectDefinition` 中添加 `granted_abilities: Vec<String>`，应用时生成技能实体

#### 9.2.4 AbilitySystemComponent 内部实现细节（UE 5.1+）
**弃用内容**: 直接访问内部数组和属性  
**我们的做法**: 从一开始就使用查询和事件，不暴露内部数据结构

| UE 弃用功能 | 我们的做法 |
|------------|----------|
| 直接访问 `SpawnedAttributes` | 使用查询 `Query<&AttributeData, With<ChildOf<owner>>>` |
| 直接访问 `ReplicatedInstancedAbilities` | 使用查询 `Query<&AbilitySpec, With<AbilityOwner>>` |
| 直接访问 `RepAnimMontageInfo` | 不实现（Bevy 动画系统不同） |
| `FindAbilitySpecFromGEHandle` | 不实现（一个效果可授予多个技能，语义不明确） |
| `ReinvokeActiveGameplayCues` | 不实现（逻辑不一致） |

#### 9.2.5 调试命令（UE 5.3+）
**弃用内容**: 旧的调试命令名称  
**我们的做法**: 不实现调试命令（使用 Bevy 的 inspector 和日志系统）

#### 9.2.6 全局配置（UE 5.5+）
**弃用内容**: `AbilitySystemGlobalsClassName` 等全局变量  
**新设计**: 通过项目设置配置  
**我们的做法**: 使用 Bevy 的 `Resource` 系统配置全局设置

### 9.3 技术债务
- ~~使用字符串 ID 而非 interned `Atom`（性能）~~ **已澄清**: `bevy_gameplay_tag` 已使用 `string_cache::Atom`，但项目中的 `definition_id` 和 `attribute_name` 仍使用 `String`，应考虑统一使用 `Atom`
- 句柄类型已定义但未使用（生成跟踪）
- 测试中硬编码资产路径（CI 失败）
- 注册表查找失败使用 `warn!` 而非错误事件

---

## 10. 性能考虑

### 10.1 Entity-Per-Thing 开销
- **优点**: 并行查询执行，缓存友好
- **缺点**: 更多实体 = 更多内存，更多原型碎片
- **缓解**: 对罕见组件使用 `SparseSet` 存储

### 10.2 标签匹配
- **优点**: `bevy_gameplay_tag` 使用 `string_cache::Atom` (interned strings)，标签比较是 O(1) 指针比较
- **优点**: 层级匹配通过预构建的父标签容器实现，查询高效
- **潜在开销**: 层级匹配需要查询 `GameplayTagsManager` 获取完整标签容器（涉及 HashMap 查找）
- **缓解**: 
  - 标签本身已经是 interned，无需额外缓存
  - 对于频繁的层级匹配，可以缓存 `GameplayTagContainer` 而非单个标签
  - 使用 `matches_tag_exact()` 代替 `matches_tag()` 可跳过层级查询（仅指针比较）

### 10.3 修改器聚合
- **优点**: 仅在修改器变化时重新计算
- **缺点**: 许多效果 = 许多修改器 = 慢聚合
- **缓解**: 批量修改器变化，使用脏标志

---

## 11. 未来增强

### 11.1 多人游戏支持
- 添加复制组件
- 实现预测键
- 添加客户端-服务器 RPC 事件

### 11.2 可视化脚本
- Bevy 基于图的技能编辑器
- 可视化效果组合
- 拖放式技能创建

### 11.3 高级功能
- 技能队列系统
- 连招系统（链式技能）
- 技能打断（优先级系统）
- 动态属性集（运行时属性创建）

---

## 12. 结论

本设计文档为使用 Bevy ECS 实现虚幻引擎 GameplayAbilitySystem 的功能完整版本提供了全面的路线图。实现优先考虑正确性、性能和符合 Bevy 习惯的模式，同时保持与 UE GAS 在单机游戏中的功能对等。

**下一步**:
1. 审查并批准本设计文档
2. 开始阶段 1（修复关键 bug）
3. 根据用户反馈迭代每个阶段
4. 在整个过程中维护全面的测试和示例

**预计时间**: 10 周完整实现（假设 1 名开发者，兼职）
