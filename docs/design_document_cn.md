# Bevy GameplayAbilitySystem 设计文档

## 1. 项目概述

### 1.1 目标

使用 Bevy ECS 架构完整复刻 Unreal Engine 的 GameplayAbilitySystem (GAS) 插件，实现与原模块功能一致的游戏能力系统。将 UE 的 OOP 设计转换为 Bevy 的纯 ECS 架构。

### 1.2 技术栈

- **引擎**: Bevy 0.18 (Rust Edition 2024)
- **标签系统**: bevy_gameplay_tag 0.2.0 (已实现)
- **字符串优化**: string_cache 0.9
- **目标场景**: 单人游戏 (不包含网络复制)

### 1.3 核心设计原则

1. **Entity-per-thing**: 属性、效果、技能均为独立实体
2. **Observer 模式**: 使用 Bevy 0.18 Observer 处理事件
3. **Definition/Registry 模式**: 定义存储在 Resource，运行时实例为实体
4. **Builder 模式**: 链式调用构建定义
5. **正确性优先**: 非法状态导致崩溃，而非静默继续

---

## 2. UE GAS 核心架构分析

### 2.1 AbilitySystemComponent - 系统中枢

**职责**:
- 管理三大核心系统: GameplayAbilities、GameplayEffects、GameplayAttributes
- 处理属性集的管理和初始化
- 应用、移除和查询 GameplayEffect
- 管理技能的授予、激活和生命周期
- 分发 GameplayCue 事件

**关键枚举**:
- `EGameplayEffectReplicationMode`: Minimal/Mixed/Full
- `EConsiderPending`: PendingAdd/PendingRemove

**核心方法**:
- `ApplyGameplayEffectSpecToTarget/Self()`: 应用效果
- `RemoveActiveGameplayEffect()`: 移除效果
- `GiveAbility()`: 授予技能
- `TryActivateAbility()`: 激活技能
- `GetNumericAttribute()`: 获取属性值

### 2.2 GameplayEffect - 效果系统

**职责**:
- 定义游戏效果的数据结构
- 支持即时、持续、周期性效果
- 提供修饰符系统和属性捕获
- 支持堆叠、免疫和条件应用

**关键枚举**:

```cpp
EGameplayEffectDurationType:
  - Instant: 即时应用
  - Infinite: 永久效果
  - HasDuration: 有持续时间

EGameplayEffectMagnitudeCalculation:
  - ScalableFloat: 简单可缩放浮点数
  - AttributeBased: 基于属性计算
  - CustomCalculationClass: 自定义计算类
  - SetByCaller: 由调用者设置

EGameplayModOp:
  - AddBase: 加到基础值
  - MultiplyAdditive: 乘法(累加)
  - DivideAdditive: 除法(累加)
  - MultiplyCompound: 乘法(复合)
  - AddFinal: 加到最终值
  - Override: 覆盖
```

**核心结构**:
- `FGameplayEffectModifierMagnitude`: 修饰符大小计算
- `FGameplayModifierInfo`: 修饰符信息(属性、操作、大小)
- `FGameplayEffectAttributeCaptureSpec`: 属性捕获规范
- `FGameplayEffectCue`: 效果关联的 Cue

### 2.3 GameplayAbility - 技能系统

**职责**:
- 定义可激活的游戏技能
- 管理激活、执行和结束生命周期
- 支持多种实例化策略
- 提供输入绑定和事件触发

**关键枚举**:

```cpp
EGameplayAbilityInstancingPolicy:
  - NonInstanced: 非实例化(共享CDO)
  - InstancedPerActor: 每个Actor一个实例
  - InstancedPerExecution: 每次执行一个实例

EGameplayAbilityNetExecutionPolicy:
  - LocalPredicted: 本地预测
  - LocalOnly: 仅本地
  - ServerOnly: 仅服务器
  - ServerInitiated: 服务器启动
```

**激活流程**:
1. `TryActivateAbility()` - 检查权限
2. `CallActivateAbility()` - 前置处理
3. `ActivateAbility()` - 执行逻辑
4. `CommitAbility()` - 应用成本和冷却
5. `EndAbility()` - 清理

### 2.4 AttributeSet - 属性系统

**职责**:
- 定义游戏属性(生命值、法力等)
- 管理基础值和当前值
- 支持属性修改的前置/后置回调
- 提供属性初始化和访问

**核心结构**:
- `FGameplayAttributeData`: BaseValue + CurrentValue
- `FGameplayAttribute`: 属性引用包装器

**关键回调**:
- `PreGameplayEffectExecute()`: 效果执行前
- `PostGameplayEffectExecute()`: 效果执行后
- `PreAttributeChange()`: 属性改变前
- `PostAttributeChange()`: 属性改变后

### 2.5 GameplayCueManager - Cue 系统

**职责**:
- 管理 GameplayCue 的加载和分发
- 处理 Cue 事件的路由和执行
- 支持异步加载和预分配

**Cue 事件类型**:
- `OnExecute`: 即时执行
- `OnActive`: 添加时触发
- `WhileActive`: 持续活跃
- `OnRemove`: 移除时触发

---

## 3. OOP 到 ECS 转换策略

### 3.1 核心转换表

| UE GAS 概念 | UE 实现 | Bevy ECS 实现 |
|------------|---------|--------------|
| AbilitySystemComponent | Actor 组件 | 拆分为多个组件 + Resource |
| AttributeSet | 子对象 | 独立实体 + ChildOf |
| ActiveGameplayEffect | 容器数组 | 独立实体 |
| GameplayAbilitySpec | 数组结构体 | 独立实体 |
| GameplayEffect (资产) | UObject 类 | Registry 中的 Definition |
| GameplayAbility (资产) | UObject 类 | Registry 中的 Definition |
| GameplayCueNotify | Actor/静态接口 | Trait + 可选实体 |

### 3.2 设计思想

**UE GAS 的分层设计**:
- 数据层: Effect/Ability/Attribute 定义
- 管理层: AbilitySystemComponent
- 执行层: Aggregator/Calculation
- 反馈层: GameplayCueManager

**Bevy ECS 的转换**:
- 数据层: Definition 结构体 + Registry Resource
- 管理层: System + Query
- 执行层: System + Observer
- 反馈层: Event + Handler Trait

---

## 4. 模块设计

### 4.1 属性系统 (Attributes)

#### 4.1.1 实体层级

```
Owner Entity (玩家/NPC)
  └── Attribute Entity (每个属性一个实体)
        ├── AttributeData (BaseValue + CurrentValue)
        ├── AttributeName (字符串标识)
        └── ChildOf<Owner> (Bevy 层级关系)
```

#### 4.1.2 核心组件

```rust
#[derive(Component)]
pub struct AttributeData {
    pub base_value: f32,
    pub current_value: f32,
    pub min_value: Option<f32>,
    pub max_value: Option<f32>,
}

#[derive(Component)]
pub struct AttributeName(pub Atom);
```

#### 4.1.3 属性集定义

```rust
pub trait AttributeSetDefinition: Send + Sync + 'static {
    fn spawn_attributes(
        &self,
        commands: &mut Commands,
        owner: Entity,
    ) -> Vec<Entity>;
}
```

#### 4.1.4 修饰符聚合顺序

1. **AddBase**: 加到基础值
2. **Multiply**: 乘法修饰符(累加后相乘)
3. **Override**: 覆盖值(取最后一个)

公式: `CurrentValue = max(min((BaseValue + AddBase) * Multiply, max_value), min_value) or Override`

#### 4.1.5 生命周期钩子

```rust
pub trait AttributeLifecycleHooks: Send + Sync + 'static {
    fn pre_attribute_change(&self, old_value: f32, new_value: f32) -> f32;
    fn post_attribute_change(&self, old_value: f32, new_value: f32);
}
```

#### 4.1.6 与 UE 的差异

| 特性 | UE GAS | Bevy 实现 |
|------|--------|-----------|
| 存储方式 | UObject 子对象 | 独立实体 + ChildOf |
| 属性访问 | 宏生成访问器 | Query 查询 |
| 回调机制 | 虚函数重写 | Trait + Observer |
| 属性捕获 | FGameplayAttribute | Entity 引用 |

---

### 4.2 效果系统 (Effects)

#### 4.2.1 实体层级

```
Effect Entity (每个活跃效果一个实体)
  ├── ActiveGameplayEffect (定义ID、等级、开始时间、堆栈数)
  ├── EffectTarget (目标实体)
  ├── EffectInstigator (施法者实体)
  ├── SetByCallerMagnitudes (可选)
  └── Modifier Entity (子实体，每个修饰符一个)
        ├── AttributeModifier (目标属性、操作、大小)
        └── ChildOf<Effect>
```

#### 4.2.2 核心组件

```rust
#[derive(Component)]
pub struct ActiveGameplayEffect {
    pub definition_id: Atom,
    pub level: i32,
    pub start_time: f32,
    pub stack_count: i32,
}

#[derive(Component)]
pub struct AttributeModifier {
    pub target_attribute: Atom,
    pub operation: ModifierOperation,
    pub magnitude: f32,
}

pub enum ModifierOperation {
    AddBase,
    Multiply,
    Override,
}
```

#### 4.2.3 效果定义

```rust
pub struct GameplayEffectDefinition {
    pub id: Atom,
    pub duration_policy: DurationPolicy,
    pub duration: Option<f32>,
    pub period: Option<f32>,
    pub modifiers: Vec<ModifierDefinition>,
    pub granted_tags: GameplayTagContainer,
    pub application_tag_requirements: TagRequirements,
    pub stacking_policy: StackingPolicy,
}

pub enum DurationPolicy {
    Instant,
    HasDuration,
    Infinite,
}

pub enum StackingPolicy {
    Independent,
    RefreshDuration,
    StackCount { max_stacks: i32 },
}
```

#### 4.2.4 修饰符计算

```rust
pub enum MagnitudeCalculation {
    ScalableFloat {
        base: f32,
        per_level: f32,
    },
    AttributeBased {
        attribute: Atom,
        coefficient: f32,
        pre_multiply_add: f32,
        post_multiply_add: f32,
    },
    CustomCalculation {
        calculator: Arc<dyn CustomMagnitudeCalculator>,
    },
    SetByCaller {
        data_tag: GameplayTag,
    },
}
```

#### 4.2.5 效果应用流程

```
1. 检查标签要求 (application_tag_requirements)
2. 检查免疫 (immunity_tags)
3. 检查自定义应用要求
4. 应用效果:
   - Instant: 立即执行修饰符 → 销毁实体
   - HasDuration/Infinite: 生成 ActiveGameplayEffect 实体
5. 应用授予标签 (granted_tags)
6. 触发 GameplayCue
```

#### 4.2.6 与 UE 的差异

| 特性 | UE GAS | Bevy 实现 |
|------|--------|-----------|
| 存储方式 | FActiveGameplayEffectsContainer | 独立实体 |
| 修饰符存储 | 内联 | 子实体 |
| 周期执行 | Timer + Delegate | Bevy Timer 组件 |
| 堆叠策略 | 枚举 + 逻辑 | 枚举 + System |
| 网络复制 | FastArraySerializer | 不支持(单机) |

---

### 4.3 技能系统 (Abilities)

#### 4.3.1 实体层级

```
Owner Entity (玩家/NPC)
  └── AbilitySpec Entity (授予的技能)
        ├── AbilitySpec (定义ID、等级、输入ID)
        ├── AbilityActiveState (激活状态)
        ├── AbilityOwner (所有者引用)
        └── AbilitySpecInstance Entity (激活时生成，子实体)
              ├── AbilitySpecInstance (实例数据)
              ├── InstanceControlState (控制状态)
              └── ChildOf<AbilitySpec>
```

#### 4.3.2 核心组件

```rust
#[derive(Component)]
pub struct AbilitySpec {
    pub definition_id: Atom,
    pub level: i32,
    pub input_id: Option<i32>,
}

#[derive(Component)]
pub struct AbilityActiveState {
    pub is_active: bool,
    pub active_count: u8,
}

#[derive(Component)]
pub struct AbilitySpecInstance {
    pub definition_id: Atom,
    pub behavior: Arc<dyn AbilityBehavior>,
}
```

#### 4.3.3 技能定义

```rust
pub struct AbilityDefinition {
    pub id: Atom,
    pub instancing_policy: InstancingPolicy,
    pub ability_tags: GameplayTagContainer,
    pub cancel_abilities_with_tag: GameplayTagContainer,
    pub block_abilities_with_tag: GameplayTagContainer,
    pub activation_required_tags: GameplayTagContainer,
    pub activation_blocked_tags: GameplayTagContainer,
    pub cost_effect: Option<Atom>,
    pub cooldown_effect: Option<Atom>,
    pub behavior: Arc<dyn AbilityBehavior>,
}

pub enum InstancingPolicy {
    NonInstanced,
    InstancedPerActor,
    InstancedPerExecution,
}
```

#### 4.3.4 技能行为 Trait

```rust
pub trait AbilityBehavior: Send + Sync + 'static {
    fn can_activate(&self, context: &ActivationContext) -> bool;
    fn activate(&self, context: &mut ActivationContext);
    fn end(&self, context: &mut EndContext);
    fn cancel(&self, context: &mut CancelContext);
}
```

#### 4.3.5 激活流程

```
1. 触发 TryActivateAbility 事件
2. Observer: on_try_activate_ability
   - 检查标签要求
   - 检查冷却和成本
   - 检查 can_activate()
3. 生成 AbilitySpecInstance 实体
4. 调用 behavior.activate()
5. 触发 CommitAbility 事件
   - 应用成本效果
   - 应用冷却效果
6. 触发 EndAbility 事件
   - 调用 behavior.end()
   - 销毁 AbilitySpecInstance 实体
```

#### 4.3.6 与 UE 的差异

| 特性 | UE GAS | Bevy 实现 |
|------|--------|-----------|
| 实例化 | UObject 实例 | Entity + Trait |
| 激活流程 | 虚函数调用 | Observer + Event |
| 成本/冷却 | GameplayEffect | GameplayEffect (相同) |
| 输入绑定 | InputComponent | 手动映射 InputID |
| 网络预测 | FPredictionKey | 不支持(单机) |

---

### 4.4 Cue 系统 (Cues)

#### 4.4.1 实体层级

```
GameplayCueManager (Resource)
  ├── static_handlers: HashMap<GameplayTag, Arc<dyn StaticCueHandler>>
  └── actor_handlers: HashMap<GameplayTag, Box<dyn ActorCueHandler>>

Actor Cue Entity (动态生成)
  ├── ActiveGameplayCue (标签、目标)
  ├── CueActor (处理器实例)
  └── 自定义组件 (如 ParticleSystem, AudioSource)
```

#### 4.4.2 核心组件

```rust
#[derive(Resource)]
pub struct GameplayCueManager {
    static_handlers: HashMap<GameplayTag, Arc<dyn StaticCueHandler>>,
    actor_handlers: HashMap<GameplayTag, Box<dyn ActorCueHandler>>,
}

#[derive(Component)]
pub struct ActiveGameplayCue {
    pub cue_tag: GameplayTag,
    pub target: Entity,
}
```

#### 4.4.3 处理器 Trait

```rust
pub trait StaticCueHandler: Send + Sync + 'static {
    fn on_execute(&self, context: &CueContext);
    fn on_active(&self, context: &CueContext);
    fn on_remove(&self, context: &CueContext);
}

pub trait ActorCueHandler: Send + Sync + 'static {
    fn spawn(&self, commands: &mut Commands, context: &CueContext) -> Entity;
    fn while_active(&self, entity: Entity, context: &CueContext);
    fn on_remove(&self, entity: Entity, commands: &mut Commands);
}
```

#### 4.4.4 Cue 路由

```
1. 接收 GameplayCueEvent
2. 层级匹配 GameplayTag
3. 路由到对应处理器:
   - Static: 直接调用 trait 方法
   - Actor: 生成/更新/销毁实体
4. 触发视觉/音效
```

#### 4.4.5 与 UE 的差异

| 特性 | UE GAS | Bevy 实现 |
|------|--------|-----------|
| 静态处理器 | UGameplayCueNotify_Static | Trait 实现 |
| Actor 处理器 | AGameplayCueNotify_Actor | Trait + Entity |
| 异步加载 | FStreamableManager | 不支持 |
| 预分配 | Object Pool | 不支持 |

---

## 5. 系统执行顺序

### 5.1 系统集

```rust
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GasSystemSet {
    Input,
    Attributes,
    Effects,
    Abilities,
    Cues,
    Cleanup,
}
```

### 5.2 执行链

```
Input → Attributes → Effects → Abilities → Cues → Cleanup
```

子系统集:
- `AttributeSystemSet`: Clamp → Events
- `EffectSystemSet`: Apply → CreateModifiers → Aggregate → UpdateDurations → ExecutePeriodic → RemoveExpired → RemoveInstant
- `AbilitySystemSet`: 独占系统 + Observer
- `CueSystemSet`: Handle → Route → ExecuteStatic → ManageActors → Cleanup → UpdateWhileActive

---

## 6. 核心事件

### 6.1 属性事件

```rust
#[derive(Event)]
pub struct AttributeChangedEvent {
    pub owner: Entity,
    pub attribute: Entity,
    pub old_value: f32,
    pub new_value: f32,
}
```

### 6.2 效果事件

```rust
#[derive(Event)]
pub struct ApplyGameplayEffectEvent {
    pub target: Entity,
    pub definition_id: Atom,
    pub level: i32,
    pub instigator: Option<Entity>,
}

#[derive(Event)]
pub struct GameplayEffectAppliedEvent {
    pub target: Entity,
    pub effect: Entity,
}
```

### 6.3 技能事件

```rust
#[derive(Event)]
pub struct TryActivateAbilityEvent {
    pub owner: Entity,
    pub ability_spec: Entity,
}

#[derive(Event)]
pub struct CommitAbilityEvent {
    pub owner: Entity,
    pub ability_spec: Entity,
}

#[derive(Event)]
pub struct EndAbilityEvent {
    pub owner: Entity,
    pub ability_spec: Entity,
    pub was_cancelled: bool,
}
```

### 6.4 Cue 事件

```rust
#[derive(Event)]
pub struct GameplayCueEvent {
    pub cue_tag: GameplayTag,
    pub target: Entity,
    pub event_type: CueEventType,
}

pub enum CueEventType {
    OnExecute,
    OnActive,
    WhileActive,
    OnRemove,
}
```

---

## 7. 工具函数

### 7.1 数学工具

```rust
pub fn clamp_optional(value: f32, min: Option<f32>, max: Option<f32>) -> f32;
pub fn lerp(a: f32, b: f32, t: f32) -> f32;
pub fn remap(value: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32;
pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32;
```

### 7.2 查询工具

```rust
pub fn find_attribute_by_name(
    owner: Entity,
    name: &str,
    attributes: &Query<(Entity, &AttributeName, &Parent)>,
) -> Option<Entity>;

pub fn get_owner_attributes(
    owner: Entity,
    attributes: &Query<(Entity, &AttributeName, &AttributeData, &Parent)>,
) -> Vec<(Entity, String, f32, f32)>;

pub fn get_active_effects_on_target(
    target: Entity,
    effects: &Query<(Entity, &ActiveGameplayEffect, &EffectTarget)>,
) -> Vec<Entity>;

pub fn find_ability_by_definition(
    owner: Entity,
    definition_id: &str,
    abilities: &Query<(Entity, &AbilitySpec, &Parent)>,
) -> Option<Entity>;
```

---

## 8. 插件组合

### 8.1 主插件

```rust
pub struct GasPlugin;

impl Plugin for GasPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            AttributePlugin,
            EffectPlugin,
            AbilityPlugin,
            CuePlugin,
        ));
    }
}
```

### 8.2 独立插件

```rust
app.add_plugins(AttributePlugin);
app.add_plugins(EffectPlugin);
app.add_plugins(AbilityPlugin);
app.add_plugins(CuePlugin);
```

---

## 9. 已知问题与技术债务

### 9.1 关键问题

1. **AttributeData::set_base_value()** 覆盖 current_value
   - 应仅设置 base_value，让聚合重新计算

2. **即时效果的 granted_tags** 导致标签泄漏
   - 标签被添加但永不移除

3. **周期性效果** 未执行修饰符
   - execute_periodic_effects_system 有 TODO

4. **ModifierOperation::AddBase** 在聚合中被跳过
   - 语义不清晰

### 9.2 设计问题

5. **StackCount 策略** 生成重复修饰符无清理

6. **Handle 类型** 定义但未使用

7. **字符串 ID** 到处使用，未用 Atom

### 9.3 代码质量

8. **Changed<AttributeData>** 在多个系统中使用
   - 可能导致重复事件

9. **测试路径硬编码**
   - CI/不同环境失败

10. **Registry 查找失败** 使用 warn!
    - 调用者无法检测失败

---

## 10. 与 UE GAS 的核心差异总结

| 方面 | UE GAS | Bevy 实现 |
|------|--------|-----------|
| 架构 | OOP + 组件 | 纯 ECS |
| 存储 | 容器 + 子对象 | 独立实体 + 层级 |
| 事件 | 委托 + 虚函数 | Observer + Event |
| 网络 | 复制 + 预测 | 不支持(单机) |
| 实例化 | UObject 实例 | Entity + Trait |
| 修饰符 | 内联存储 | 子实体 |
| 标签 | FGameplayTagContainer | bevy_gameplay_tag |
| 性能 | 对象池 + 缓存 | 需手动优化 |

---

## 11. 实现路线图

### 阶段 1: 修复关键 Bug (1 周)
- 修复 AttributeData::set_base_value()
- 修复即时效果标签泄漏
- 实现周期性效果执行
- 修复 AddBase 操作

### 阶段 2: 效果系统增强 (2 周)
- 实现 AttributeBased 计算
- 实现 CustomCalculation
- 实现 SetByCaller
- 修复 StackCount 策略

### 阶段 3: 技能系统增强 (2 周)
- 实现实例化策略
- 实现技能触发器
- 实现技能任务

### 阶段 4: Cue 系统增强 (1 周)
- 添加 GameplayCueParameters
- 实现 Cue 转换
- Bevy 资产加载集成

### 阶段 5: 高级功能 (2 周)
- 实现效果免疫
- 实现授予技能
- 实现目标选择

### 阶段 6: 优化和测试 (2 周)
- 性能优化
- 完善测试
- 编写示例

**总计**: 10 周

---

## 12. 参考资料

### 12.1 UE GAS 源码
- AbilitySystemComponent.h
- GameplayEffect.h
- GameplayAbility.h
- AttributeSet.h
- GameplayCueManager.h

### 12.2 Bevy 文档
- Bevy ECS 指南
- Bevy Observer 系统
- Bevy 层级系统

### 12.3 项目文档
- CLAUDE.md
- bevy_gameplay_tag 文档

---

## 13. 结语

本设计文档提供了将 UE GameplayAbilitySystem 从 OOP 转换为 Bevy ECS 的完整路线图。核心思想:

1. **实体化**: UObject → Entity
2. **组件化**: 属性 → Component
3. **系统化**: 方法 → System
4. **事件化**: 委托 → Event + Observer

通过这种转换，保持 GAS 的核心功能，同时充分利用 Bevy ECS 的性能优势。
