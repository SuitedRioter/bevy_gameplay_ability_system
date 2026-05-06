# Effects 模块设计文档

## 概述

Effects 模块提供了一个全面的游戏效果管理框架，灵感来自 UE 的 GameplayAbilitySystem。它基于纯 ECS 架构，用于对实体属性进行临时或永久的修改、授予标签、触发游戏提示等。

## 设计理念

**实体化效果，模块化扩展**

- 每个活跃效果都是独立实体，利用 Bevy 的查询优化实现并行处理
- 定义/注册表模式：效果模板存储在注册表中，运行时实例从模板生成
- 通过 `GameplayEffectComponent` trait 扩展效果行为，无需修改核心定义
- 基于 Tag 的条件系统实现效果的应用要求、免疫和移除逻辑

## ECS 架构设计

### 实体关系

```
Owner Entity (玩家/怪物)
    ├─ OwnedTags (所有者的 Tag 容器)
    │
    ├─ ActiveGameplayEffect Entity #1 (中毒效果)
    │     ├─ ActiveGameplayEffect (definition_id, source, target, level, granted_tags)
    │     ├─ EffectTarget (target Entity)
    │     ├─ EffectDuration (remaining, initial) [可选]
    │     ├─ PeriodicEffect (period, time_until_next) [可选]
    │     └─ AttributeModifier Entity (修改器, 子实体)
    │           ├─ AttributeModifier (target_entity, target_attribute, operation, magnitude)
    │           └─ ChildOf → ActiveGameplayEffect Entity
    │
    ├─ ActiveGameplayEffect Entity #2 (攻击力增益)
    │     ├─ ActiveGameplayEffect (...)
    │     ├─ EffectTarget (...)
    │     └─ AttributeModifier Entity (...)
    │
    └─ ActiveGameplayEffect Entity #3 (护盾效果)
          ├─ ActiveGameplayEffect (...)
          ├─ EffectTarget (...)
          └─ AttributeModifier Entity (...)
```

**重要说明**：一个 Owner Entity 可以同时拥有**多个** ActiveGameplayEffect Entity。每个效果都是独立的实体，拥有自己的生命周期、持续时间、堆叠计数和修改器。这种设计使得：
- 一个角色可以同时受到多个增益、减益、持续伤害等效果
- 每个效果独立管理，可以单独移除而不影响其他效果
- Bevy 的查询系统可以并行处理多个效果

### 核心组件

#### 1. ActiveGameplayEffect — 活跃效果实例
```rust
pub struct ActiveGameplayEffect {
    pub definition_id: Atom,        // 效果定义 ID
    pub source: Entity,             // 施加者
    pub target: Entity,             // 接收者
    pub level: i32,                 // 效果等级
    pub start_time: f32,            // 应用时间
    pub granted_tags: GameplayTagContainer,  // 授予目标的标签
    pub stack_count: i32,           // 当前堆叠数
}
```

每个活跃效果是一个独立实体，这种设计使得：
- Bevy 的查询系统可以高效并行处理
- 效果生命周期管理清晰独立
- 支持高效的过滤和查询

#### 2. GameplayEffectSpec — 运行时效果规格
```rust
pub struct GameplayEffectSpec {
    pub effect_id: Atom,
    pub target: Entity,
    pub level: i32,
    pub context: GameplayEffectContext,
    pub set_by_caller_magnitudes: SetByCallerMagnitudes,
    pub captured_attributes: HashMap<(Entity, Atom), f32>,
}
```

类似 UE 的 `FGameplayEffectSpec`，携带效果应用时的完整上下文信息。

#### 3. AttributeModifier — 属性修改器
```rust
pub struct AttributeModifier {
    pub target_entity: Entity,
    pub target_attribute: Atom,
    pub channel: EvaluationChannel,
    pub operation: ModifierOperation,
    pub magnitude: f32,
}
```

代表对单个属性的一次修改。修改器是独立实体，作为效果的子实体存在。

## 持续时间策略

三种持续时间策略控制效果的生命周期：

### Instant（即时）
- 立即应用并在同一帧移除
- 直接修改 `base_value`（永久改变）
- **不能授予标签**（没有持久实体）
- 用例：即时伤害/治疗、永久属性变化

### HasDuration（有持续时间）
- 持续指定的时间
- 通过修改器修改 `current_value`
- 可以在活跃期间授予标签
- 用例：增益、减益、临时效果

### Infinite（无限）
- 持续到显式移除
- 通过修改器修改 `current_value`
- 可以授予标签
- 用例：被动技能、永久增益

## 堆叠策略

三种堆叠策略控制多次应用的交互方式：

### Independent（独立）
- 每次应用创建独立的效果实体
- 所有效果独立运行
- 用例：多个不同的增益效果

### RefreshDuration（刷新持续时间）
- 重新应用效果，刷新持续时间
- 更新上下文和 SetByCaller 数值
- 维持单个效果实体
- 用例：可刷新的增益（如移动速度提升）

### StackCount（堆叠计数）
- 增加堆叠计数，最多到 `max_stacks`
- 修改器随堆叠数缩放
- 重新应用时刷新持续时间
- 用例：可堆叠的减益（如中毒堆叠）

## 数值计算

五种计算类型决定修改器的数值：

### ScalableFloat（可缩放浮点数）
固定值，可选等级缩放：
```
magnitude = base_value * level_multiplier^(level - 1)
```

### AttributeBased（基于属性）
从源或目标实体捕获属性值：
```
magnitude = (coefficient * (pre_multiply_additive + [attribute_value])) + post_multiply_additive
```

支持：
- **捕获源**：Source（源实体）或 Target（目标实体）
- **计算类型**：AttributeMagnitude（当前值）、AttributeBaseValue（基础值）、AttributeBonusMagnitude（奖励值）
- **捕获模式**：Snapshot（创建时快照）或 Dynamic（动态重新计算）

### CustomClass（自定义类）
通过名称查找注册的 `CustomMagnitudeCalculation`。适用于捕获多个属性的复杂计算。

### CustomExecution（自定义执行）
使用 `GameplayEffectExecutionCalculation` trait 实现，提供最大灵活性。可以捕获多个属性并产生多个修改器。

### SetByCaller（调用者设置）
运行时由调用者通过 `SetByCallerMagnitudes` 组件提供数值。如果未提供则默认为 0.0。

## 修改器操作

五种操作定义修改器如何影响属性：

### Override（覆盖）
完全替换当前值。最高优先级，短路求值。

### AddBase（加到基础值）
加到基础值。在 AddCurrent 之前应用。

### AddCurrent（加到当前值）
加到当前值。在 AddBase 之后应用。

### MultiplyAdditive（加法乘法）
乘以 `(1 + sum(所有加法乘数))`。加法乘数以加法方式堆叠。

### MultiplyMultiplicative（乘法乘法）
乘以 `prod(1 + 每个乘数)`。乘法乘数以复合方式堆叠。

## 评估通道

十个评估通道（Channel0-Channel9）实现复杂的堆叠规则：

- 通道按顺序评估（0 → 9）
- 每个通道的输出成为下一个通道的输入
- 通道内，操作按优先级顺序应用：
  1. Override（短路）
  2. AddBase
  3. AddCurrent
  4. MultiplyAdditive
  5. MultiplyMultiplicative

示例用例：
- Channel0：基础属性修改器
- Channel1：百分比加成
- Channel2：临时增益
- Channel3：减益

## 周期性效果

效果可以按固定间隔周期性执行：

```rust
pub struct PeriodicEffect {
    pub period: f32,              // 执行间隔（秒）
    pub time_until_next: f32,     // 下次执行倒计时
    pub execute_on_application: bool,  // 应用时立即执行
}
```

周期性效果：
- 在每个周期 tick 时执行修改器
- 不创建持久修改器（仅即时修改）
- 可以在应用时立即执行
- 正确处理大 delta 时间（多次执行）

## 标签系统集成

### 授予标签
效果可以在活跃期间授予标签给目标：
- 效果应用时添加标签
- 效果移除时移除标签
- 存储在 `ActiveGameplayEffect::granted_tags`

### 标签要求
效果可以有基于标签的应用要求：
- **Application Tag Requirements**：目标必须有这些标签才能应用
- **Removal Tag Requirements**：目标获得这些标签时移除效果
- **Ongoing Tag Requirements**：目标失去这些标签时移除效果
- **Immunity Tags**：拥有这些标签的目标阻止效果应用

## 应用要求

通过 `ApplicationRequirement` trait 实现条件效果应用的自定义逻辑：

```rust
pub trait ApplicationRequirement: Send + Sync {
    fn can_apply(&self, ctx: &ApplicationContext) -> bool;
}
```

内置要求：
- **AttributePercentAbove/Below**：属性百分比检查
- **AttributeAboveThreshold/BelowThreshold**：绝对值检查
- **SourceAttributeGreaterThanTarget**：源 vs 目标比较
- **LevelRangeRequirement**：基于等级的门槛
- **RequireAllTags/RequireAnyTag/BlockIfHasTag**：基于标签的条件
- **AndRequirement/OrRequirement/NotRequirement**：逻辑组合器

## 自定义计算

### CustomMagnitudeCalculation

用于捕获多个属性的复杂数值计算：

```rust
pub trait CustomMagnitudeCalculation: Send + Sync {
    fn calculate(&self, ctx: &CalculationContext) -> f32;
    fn required_source_attributes(&self) -> &[&'static str];
    fn required_target_attributes(&self) -> &[&'static str];
}
```

### GameplayEffectExecutionCalculation

提供最大灵活性，匹配 UE 的 `UGameplayEffectExecutionCalculation`：

```rust
pub trait GameplayEffectExecutionCalculation: Send + Sync + Debug {
    fn relevant_attributes_to_capture(&self) -> Vec<AttributeCaptureDefinition>;
    fn execute(
        &self,
        spec: &GameplayEffectSpec,
        captured_attributes: &HashMap<Atom, f32>,
        world: &World,
    ) -> Vec<GameplayModifierEvaluatedData>;
}
```

执行计算可以：
- 从源/目标捕获多个属性
- 执行复杂计算
- 产生多个修改器
- 访问 world 状态进行额外查询

## GameplayEffect 组件（UE 5.3+）

模块化组件扩展效果行为，无需修改核心定义：

### GameplayEffectComponent Trait

```rust
pub trait GameplayEffectComponent: Send + Sync {
    fn can_apply(&self, effect_definition_id: &str, source: Entity, target: Entity, world: &World) -> bool;
    fn on_effect_applied(&self, effect: Entity, target: Entity, world: &mut World);
    fn on_effect_removed(&self, effect: Entity, target: Entity, removal_info: &EffectRemovalInfo, world: &mut World);
}
```

### 内置组件

**ChanceToApplyComponent**：基于概率的应用
```rust
let component = ChanceToApplyComponent::new(0.5);  // 50% 概率
```

**ImmunityComponent**：授予对匹配查询的效果的免疫
```rust
let query = GameplayEffectQuery::new()
    .with_owning_tags_any(vec!["Effect.Debuff.Poison"], &manager);
let component = ImmunityComponent::new(vec![query]);
```

**AdditionalEffectsComponent**：在生命周期点应用额外效果
```rust
let component = AdditionalEffectsComponent::new()
    .on_application(vec!["apply_damage".into()])
    .on_complete_normal(vec!["heal_on_expire".into()]);
```

**RemoveOtherEffectsComponent**：移除匹配查询的效果
```rust
let query = GameplayEffectQuery::new()
    .with_owning_tags_any(vec!["Effect.Debuff.Poison"], &manager);
let component = RemoveOtherEffectsComponent::new(vec![query]);
```

## GameplayEffect 查询系统

基于多个条件匹配效果的灵活查询系统：

```rust
let query = GameplayEffectQuery::new()
    .with_definition_id("poison")
    .with_owning_tags_any(vec!["Effect.Debuff"], &manager)
    .with_source_tags_all(vec!["Actor.Enemy"], &manager)
    .with_custom_match(|effect, world| {
        // 自定义匹配逻辑
        true
    });

let matching_effects = query.find_matching_effects(target, world);
```

查询条件：
- 效果定义 ID
- 效果拥有的标签（授予的标签）
- 源实体拥有的标签
- 自定义匹配函数

## 技能授予

效果可以授予技能给目标：

```rust
pub struct GrantedAbilityConfig {
    pub ability_id: Atom,
    pub removal_policy: AbilityRemovalPolicy,
}
```

### 移除策略

**CancelAbilityImmediately**：立即取消技能并移除规格

**RemoveAbilityOnEnd**：移除规格但让活跃实例完成

**DoNothing**：技能永久保留

### 实现

- `grant_abilities_from_effects_system`：效果应用时授予技能
- `on_gameplay_effect_removed_remove_granted_abilities`：处理移除的观察者
- `cleanup_remove_on_end_abilities_system`：清理标记为移除的技能

## 批量聚合优化

`batch_aggregation` 模块提供性能优化的修改器处理：

### ModifierBatch

按通道和操作预分组修改器：

```rust
pub struct ModifierBatch {
    pub channels: BTreeMap<EvaluationChannel, ChannelModifiers>,
}
```

### ChannelModifiers

在通道内按操作分组修改器：

```rust
pub struct ChannelModifiers {
    pub overrides: Vec<f32>,
    pub add_base: Vec<f32>,
    pub add_current: Vec<f32>,
    pub multiply_additive: Vec<f32>,
    pub multiply_multiplicative: Vec<f32>,
}
```

### ModifierAggregator

高效收集和处理修改器：

```rust
let mut aggregator = ModifierAggregator::new();
aggregator.add_modifier(&modifier);
let batch = aggregator.get_batch(owner, &attribute_name);
let final_value = batch.evaluate(base_value);
```

优势：
- 减少迭代次数
- 改善缓存局部性
- 预分组修改器以实现高效评估
- 自动处理通道排序

## 事件系统

### ApplyGameplayEffectEvent

触发效果应用：

```rust
commands.trigger(
    ApplyGameplayEffectEvent::from_spec(
        GameplayEffectSpec::new("heal", target)
            .with_level(5)
            .with_source(caster)
    )
);
```

### GameplayEffectAppliedEvent

效果成功应用时触发：

```rust
pub struct GameplayEffectAppliedEvent {
    pub effect: Entity,
    pub target: Entity,
    pub effect_id: Atom,
}
```

### GameplayEffectRemovedEvent

效果移除时触发：

```rust
pub struct GameplayEffectRemovedEvent {
    pub effect: Entity,
    pub target: Entity,
    pub effect_id: Atom,
}
```

### GameplayEffectBlockedByImmunityEvent

效果被免疫阻止时触发：

```rust
pub struct GameplayEffectBlockedByImmunityEvent {
    pub effect_id: Atom,
    pub target: Entity,
    pub instigator: Option<Entity>,
    pub immunity_tag: GameplayTag,
}
```

## GameplayCue 集成

效果可以在各个生命周期点触发游戏提示：

```rust
pub struct GameplayCueInfo {
    pub cue_tag: GameplayTag,
    pub min_level: i32,
    pub max_level: i32,
    pub parameters: GameplayCueParameters,
}
```

提示事件：
- **OnActive**：效果应用时
- **WhileActive**：效果活跃期间持续
- **Executed**：周期性效果执行时
- **Removed**：效果移除时

## 构建器模式

效果定义使用流畅的构建器 API：

```rust
let effect = GameplayEffectDefinition::new("heal_over_time")
    .with_duration(5.0)
    .with_period(1.0)
    .add_modifier(ModifierInfo::new(
        "Health",
        ModifierOperation::AddCurrent,
        MagnitudeCalculation::scalar(10.0),
    ))
    .add_granted_tag("State.Healing", &manager)
    .add_gameplay_cue(GameplayCueInfo::new("GameplayCue.Heal"))
    .with_stacking_policy(StackingPolicy::RefreshDuration);

registry.register(effect);
```

## 系统执行顺序

所有系统在 `Update` 调度中运行，通过 `EffectSystemSet` 链接：

```
Apply → CreateModifiers → Aggregate → UpdateDurations → ExecutePeriodic → RemoveExpired → RemoveInstant
```

这个顺序确保：
1. 效果首先应用
2. 从活跃效果创建修改器
3. 修改器聚合并应用到属性
4. 持续时间更新
5. 周期性效果执行
6. 过期效果移除
7. 即时效果清理

## 安全保证

### 编译时检查

- 带有 `granted_tags` 的即时效果在注册时 panic（非法状态）
- 通过枚举实现类型安全的数值计算
- 对操作和策略进行穷尽模式匹配

### 运行时验证

- 效果定义查找失败记录警告并提前返回
- 属性查询优雅处理缺失实体
- 应用前检查标签要求
- 免疫检查防止不需要的效果

## 性能考虑

### 优化

- **批量聚合**：按通道和操作预分组修改器
- **实体化效果**：通过 Bevy 的查询系统实现并行处理
- **通道的 BTreeMap**：确保有序评估无需排序
- **内联函数**：关键路径函数标记 `#[inline]`
- **提前返回**：Override 操作短路评估

### 可扩展性

当前设计处理：
- <50 个实体，每个 <10 个属性
- 数百个活跃效果
- 复杂的修改器堆叠规则

未来优化延迟：
- 效果查询的空间分区
- 未更改修改器的脏标志优化
- 批量计算的 SIMD

## 测试模式

### 单元测试

隔离测试单个组件：

```rust
#[test]
fn test_channel_modifiers_add_base() {
    let mut channel = ChannelModifiers::default();
    channel.add_modifier(ModifierOperation::AddBase, 10.0);
    channel.add_modifier(ModifierOperation::AddBase, 20.0);
    
    let result = channel.evaluate(100.0);
    assert_eq!(result, 130.0);
}
```

### 集成测试

测试完整的效果生命周期：

```rust
#[test]
fn test_effect_application_flow() {
    let mut app = App::new();
    // 设置插件、注册表、实体
    
    app.world_mut().trigger(
        ApplyGameplayEffectEvent::new("heal", target)
    );
    
    app.update();
    
    // 验证效果已应用
    // 验证修改器已创建
    // 验证属性已修改
}
```

### 测试工具

- `TestEvents` 资源，使用 `Arc<Mutex<Vec<T>>>` 跨观察者捕获事件
- `app.world_mut().run_system_once()` 用于依赖系统参数的测试
- 持续时间测试使用手动 `duration.tick()` 而非 `Time::advance_by()`

## 已知限制

1. **仅单人游戏**：无网络/复制支持
2. **性能**：优化延迟到性能分析显示瓶颈
3. **基准测试套件**：Bevy 0.18 的 criterion 兼容性问题导致损坏
4. **组件执行**：AdditionalEffectsComponent 和 ImmunityComponent 有简化实现

## 未来增强

### 计划功能

- **效果堆叠可视化**：用于查看活跃效果和修改器的调试 UI
- **效果预测**：网络游戏的客户端预测
- **效果池化**：频繁创建/销毁效果的对象池
- **条件修改器**：仅在满足条件时应用的修改器

### 潜在改进

- **属性捕获缓存**：为 Dynamic 模式缓存捕获的属性
- **修改器差异**：仅重新计算更改的修改器
- **效果模板**：带继承的分层效果定义
- **效果组**：对多个效果的批量操作

## 模块结构

```
src/effects/
├── mod.rs                      # 模块导出
├── definition.rs               # GameplayEffectDefinition, MagnitudeCalculation
├── components.rs               # ActiveGameplayEffect, AttributeModifier 等
├── plugin.rs                   # EffectPlugin 注册
├── systems.rs                  # 核心系统和观察者
├── batch_aggregation.rs        # 优化的修改器聚合
├── custom_calculation.rs       # CustomMagnitudeCalculation trait
├── execution.rs                # GameplayEffectExecutionCalculation trait
├── application_requirement.rs  # ApplicationRequirement trait
├── builtin_requirements.rs     # 内置要求实现
├── ge_component.rs             # GameplayEffectComponent trait
├── ge_components.rs            # 内置组件实现
├── ability_granting.rs         # 技能授予系统
└── query.rs                    # GameplayEffectQuery 系统
```

## 使用示例

### 简单治疗效果

```rust
let heal = GameplayEffectDefinition::new("instant_heal")
    .with_duration_policy(DurationPolicy::Instant)
    .add_modifier(ModifierInfo::new(
        "Health",
        ModifierOperation::AddBase,
        MagnitudeCalculation::scalar(50.0),
    ));

registry.register(heal);

// 应用
commands.trigger(ApplyGameplayEffectEvent::new("instant_heal", target));
```

### 持续伤害

```rust
let dot = GameplayEffectDefinition::new("poison")
    .with_duration(10.0)
    .with_period(1.0)
    .add_modifier(ModifierInfo::new(
        "Health",
        ModifierOperation::AddCurrent,
        MagnitudeCalculation::scalar(-5.0),
    ))
    .add_granted_tag("State.Poisoned", &manager)
    .add_gameplay_cue(GameplayCueInfo::new("GameplayCue.Poison"));

registry.register(dot);
```

### 基于属性的伤害

```rust
let damage = GameplayEffectDefinition::new("attack_damage")
    .with_duration_policy(DurationPolicy::Instant)
    .add_modifier(ModifierInfo::new(
        "Health",
        ModifierOperation::AddCurrent,
        MagnitudeCalculation::from_source_attribute("AttackPower", 1.5)
            .with_pre_multiply_additive(10.0),  // 基础伤害
    ));

registry.register(damage);
```

### 可堆叠增益

```rust
let buff = GameplayEffectDefinition::new("attack_buff")
    .with_duration(30.0)
    .with_stacking_policy(StackingPolicy::StackCount { max_stacks: 5 })
    .add_modifier(ModifierInfo::new(
        "AttackPower",
        ModifierOperation::AddCurrent,
        MagnitudeCalculation::scalar(10.0),
    ));

registry.register(buff);
```

### 复杂执行计算

```rust
#[derive(Debug)]
struct CriticalDamageCalculation;

impl GameplayEffectExecutionCalculation for CriticalDamageCalculation {
    fn relevant_attributes_to_capture(&self) -> Vec<AttributeCaptureDefinition> {
        vec![
            AttributeCaptureDefinition::snapshot_source("AttackPower"),
            AttributeCaptureDefinition::snapshot_source("CritChance"),
            AttributeCaptureDefinition::dynamic_target("Defense"),
        ]
    }

    fn execute(
        &self,
        spec: &GameplayEffectSpec,
        captured_attributes: &HashMap<Atom, f32>,
        _world: &World,
    ) -> Vec<GameplayModifierEvaluatedData> {
        let attack = captured_attributes.get(&"AttackPower".into()).copied().unwrap_or(0.0);
        let crit_chance = captured_attributes.get(&"CritChance".into()).copied().unwrap_or(0.0);
        let defense = captured_attributes.get(&"Defense".into()).copied().unwrap_or(0.0);
        
        let is_crit = rand::random::<f32>() < crit_chance;
        let multiplier = if is_crit { 2.0 } else { 1.0 };
        let damage = ((attack * multiplier) - defense).max(0.0);
        
        vec![GameplayModifierEvaluatedData::new(
            "Health",
            ModifierOperation::AddCurrent,
            -damage,
        )]
    }
}

let effect = GameplayEffectDefinition::new("critical_attack")
    .with_duration_policy(DurationPolicy::Instant)
    .add_modifier(ModifierInfo::new(
        "Health",
        ModifierOperation::AddCurrent,
        MagnitudeCalculation::execution(Arc::new(CriticalDamageCalculation)),
    ));
```

## 总结

Effects 模块提供了一个健壮、灵活且高性能的游戏效果管理框架。其基于 ECS 的架构、全面的功能集和可扩展性使其适用于各种游戏类型和机制。

关键优势：
- **纯 ECS 设计**：利用 Bevy 的优势实现并行处理
- **全面的功能集**：匹配 UE GAS 的能力
- **类型安全**：编译时保证防止常见错误
- **可扩展性**：自定义计算、要求和组件
- **性能**：优化的批量聚合和高效查询

该系统已为单人游戏做好生产准备，并为未来增强提供了坚实的基础。
