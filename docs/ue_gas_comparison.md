# UE GameplayAbilitySystem 对比分析与优化建议

**文档日期**: 2026-04-30  
**分析范围**: Unreal Engine 5.3+ GameplayAbilities 插件 vs 当前 Bevy 实现

## 执行摘要

当前 Bevy GAS 实现已完成核心功能，架构设计优秀，测试覆盖率达 100%（127/127 tests passing）。主要差异在于 UE 5.3+ 引入的**组件化设计**（GameplayEffectComponent 系统）。建议优先实现模块化组件系统以提升可扩展性。

---

## 一、当前实现评估

### ✅ 已实现的核心功能

#### 1. Attribute 系统
- 双值模型（BaseValue/CurrentValue）
- Modifier aggregation（Add → Multiply → Override）
- Entity-per-attribute 设计
- 自定义 AttributeSet trait

#### 2. GameplayEffect 系统
- 三种 duration policy（Instant/HasDuration/Infinite）
- Periodic execution
- Stacking policies（Independent/RefreshDuration/StackCount）
- Evaluation channels（Channel0-Channel9）
- Tag-based requirements
- Attribute-based magnitudes（Snapshot/Dynamic capture）
- Custom application requirements

#### 3. Ability 系统
- 三种 instancing policy（NonInstanced/InstancedPerActor/InstancedPerExecution）
- Activation flow（TryActivate → Commit → End/Cancel）
- Tag-based requirements/blocking/cancellation
- Cost/Cooldown effects
- 12 种 Ability Tasks

#### 4. GameplayCue 系统
- Static handlers（trait-based）
- Actor handlers（spawned entity）
- Hierarchical tag matching

### 架构优势

1. **纯 ECS 设计**
   - Entity-per-thing 模式（attributes、effects、abilities 都是独立 entity）
   - 充分利用 Bevy 的并行查询和 observer 模式
   - 清晰的系统执行顺序（GasSystemSet）

2. **代码质量**
   - 8225 行代码，41 个源文件
   - 21 个 public enums，108 个 public structs
   - 完整的测试覆盖（unit + integration + doc tests）

3. **设计模式**
   - Definition/Registry 模式（模板与实例分离）
   - Builder 模式（流式 API）
   - SystemParam bundles（复杂查询封装）

---

## 二、与 UE GAS 的功能差异

### 1. ⭐ GameplayEffectComponent 系统（最重要）

**UE 5.3+ 的重大改进**：将 monolithic `UGameplayEffect` 拆分为可组合的组件。

#### UE GAS 提供的组件类型

| 组件名称 | 功能 | 当前实现状态 |
|---------|------|------------|
| `TargetTagsGameplayEffectComponent` | 授予 tags 给目标 | ✅ 已实现（granted_tags） |
| `AbilitiesGameplayEffectComponent` | 授予 abilities | ✅ 已实现（granted_abilities） |
| `ImmunityGameplayEffectComponent` | 免疫其他 effects | ❌ 未实现 |
| `ChanceToApplyGameplayEffectComponent` | 概率应用 | ❌ 未实现 |
| `AdditionalEffectsGameplayEffectComponent` | 触发额外 effects | ❌ 未实现 |
| `BlockAbilityTagsGameplayEffectComponent` | 阻止 abilities | ✅ 已实现（block_abilities_with_tags） |
| `RemoveOtherGameplayEffectComponent` | 移除其他 effects | ❌ 未实现 |
| `CustomCanApplyGameplayEffectComponent` | 自定义应用条件 | ✅ 已实现（custom requirements） |

#### 建议实现

```rust
// src/effects/components.rs

/// Trait for modular GameplayEffect components
pub trait GameplayEffectComponent: Send + Sync {
    /// Called when effect is applied to target
    fn on_effect_applied(&self, effect: Entity, target: Entity, world: &mut World);
    
    /// Called when effect is removed from target
    fn on_effect_removed(&self, effect: Entity, target: Entity, world: &mut World);
    
    /// Check if effect can be applied
    fn can_apply(&self, spec: &GameplayEffectSpec, target: Entity, world: &World) -> bool {
        true // Default: allow
    }
}

// 在 GameplayEffectDefinition 中添加
pub struct GameplayEffectDefinition {
    // ... 现有字段
    
    /// Modular components for extensibility
    pub components: Vec<Arc<dyn GameplayEffectComponent>>,
}
```

#### 具体组件实现示例

**ImmunityComponent**（免疫系统）：
```rust
pub struct ImmunityComponent {
    /// Queries to match effects that should be blocked
    pub immunity_queries: Vec<GameplayEffectQuery>,
}

impl GameplayEffectComponent for ImmunityComponent {
    fn on_effect_applied(&self, effect: Entity, target: Entity, world: &mut World) {
        // Register immunity callback on AbilitySystemComponent
    }
    
    fn can_apply(&self, spec: &GameplayEffectSpec, target: Entity, world: &World) -> bool {
        // Check if spec matches any immunity query
        !self.immunity_queries.iter().any(|q| q.matches(spec, world))
    }
}
```

**ChanceToApplyComponent**（概率应用）：
```rust
pub struct ChanceToApplyComponent {
    /// Probability [0.0, 1.0]
    pub chance: f32,
}

impl GameplayEffectComponent for ChanceToApplyComponent {
    fn can_apply(&self, _spec: &GameplayEffectSpec, _target: Entity, _world: &World) -> bool {
        rand::random::<f32>() < self.chance
    }
}
```

**AdditionalEffectsComponent**（触发额外效果）：
```rust
pub struct AdditionalEffectsComponent {
    /// Effects to apply when this effect is applied
    pub on_application: Vec<Atom>,
    /// Effects to apply when this effect completes (any reason)
    pub on_complete_always: Vec<Atom>,
    /// Effects to apply when this effect expires naturally
    pub on_complete_normal: Vec<Atom>,
    /// Effects to apply when this effect is removed prematurely
    pub on_complete_prematurely: Vec<Atom>,
}

impl GameplayEffectComponent for AdditionalEffectsComponent {
    fn on_effect_applied(&self, effect: Entity, target: Entity, world: &mut World) {
        // Apply on_application effects
        for effect_id in &self.on_application {
            // Trigger effect application
        }
    }
    
    fn on_effect_removed(&self, effect: Entity, target: Entity, world: &mut World) {
        // Determine removal reason and apply appropriate effects
    }
}
```

---

### 2. GameplayEffectQuery（效果查询系统）

UE GAS 使用 `FGameplayEffectQuery` 进行复杂的 effect 匹配（用于 immunity、removal、conditional application）。

#### 建议实现

```rust
// src/effects/query.rs

pub struct GameplayEffectQuery {
    /// Match effects with these tags
    pub owning_tags_query: Option<GameplayTagRequirements>,
    
    /// Match effects applied by source with these tags
    pub source_tags_query: Option<GameplayTagRequirements>,
    
    /// Match effects with this definition ID
    pub effect_definition: Option<Atom>,
    
    /// Custom matching function
    pub custom_match: Option<Arc<dyn Fn(Entity, &World) -> bool + Send + Sync>>,
}

impl GameplayEffectQuery {
    pub fn matches(&self, effect: Entity, world: &World) -> bool {
        // Check owning_tags_query
        if let Some(ref query) = self.owning_tags_query {
            if let Some(active_effect) = world.get::<ActiveGameplayEffect>(effect) {
                if !query.is_satisfied(&active_effect.granted_tags, world) {
                    return false;
                }
            }
        }
        
        // Check source_tags_query
        if let Some(ref query) = self.source_tags_query {
            if let Some(source) = world.get::<EffectSource>(effect) {
                if let Some(source_tags) = world.get::<OwnedTags>(source.entity) {
                    if !query.is_satisfied(&source_tags.tags, world) {
                        return false;
                    }
                }
            }
        }
        
        // Check effect_definition
        if let Some(ref def_id) = self.effect_definition {
            if let Some(active_effect) = world.get::<ActiveGameplayEffect>(effect) {
                if &active_effect.definition_id != def_id {
                    return false;
                }
            }
        }
        
        // Check custom_match
        if let Some(ref matcher) = self.custom_match {
            if !matcher(effect, world) {
                return false;
            }
        }
        
        true
    }
}
```

---

### 3. GameplayEffectExecutionCalculation（执行计算）

UE GAS 支持复杂的 `UGameplayEffectExecutionCalculation`，可以捕获多个 attributes 并执行自定义计算逻辑（比当前的 `AttributeBased` magnitude 更强大）。

#### 当前实现的局限

当前 `MagnitudeCalculation::AttributeBased` 只能捕获**单个** attribute：

```rust
pub enum MagnitudeCalculation {
    AttributeBased {
        attribute_name: Atom,
        capture_source: AttributeCaptureSource,
        calculation_type: AttributeCalculationType,
        capture_mode: AttributeCaptureMode,
        coefficient: f32,
        pre_multiply_additive: f32,
        post_multiply_additive: f32,
    },
}
```

#### 建议实现

```rust
// src/effects/execution.rs

pub struct AttributeCaptureDefinition {
    pub attribute_name: Atom,
    pub capture_source: AttributeCaptureSource,
    pub snapshot: bool,
}

pub struct GameplayModifierEvaluatedData {
    pub attribute: Atom,
    pub modifier_op: ModifierOperation,
    pub magnitude: f32,
}

pub trait GameplayEffectExecutionCalculation: Send + Sync {
    /// 定义需要捕获的 attributes
    fn relevant_attributes_to_capture(&self) -> Vec<AttributeCaptureDefinition>;
    
    /// 执行计算，可以返回多个 modifier
    fn execute(
        &self,
        spec: &GameplayEffectSpec,
        captured_attributes: &HashMap<Atom, f32>,
        world: &World,
    ) -> Vec<GameplayModifierEvaluatedData>;
}

// 在 MagnitudeCalculation 中添加
pub enum MagnitudeCalculation {
    ScalableFloat { /* ... */ },
    AttributeBased { /* ... */ },
    
    /// Custom calculation with multi-attribute capture
    CustomCalculation {
        calculation: Arc<dyn GameplayEffectExecutionCalculation>,
    },
}
```

#### 使用示例

```rust
// 实现一个复杂的伤害计算：Damage = (AttackPower * 2 - Defense) * CritMultiplier
struct DamageCalculation;

impl GameplayEffectExecutionCalculation for DamageCalculation {
    fn relevant_attributes_to_capture(&self) -> Vec<AttributeCaptureDefinition> {
        vec![
            AttributeCaptureDefinition {
                attribute_name: "AttackPower".into(),
                capture_source: AttributeCaptureSource::Source,
                snapshot: true,
            },
            AttributeCaptureDefinition {
                attribute_name: "Defense".into(),
                capture_source: AttributeCaptureSource::Target,
                snapshot: false, // Dynamic
            },
            AttributeCaptureDefinition {
                attribute_name: "CritMultiplier".into(),
                capture_source: AttributeCaptureSource::Source,
                snapshot: true,
            },
        ]
    }
    
    fn execute(
        &self,
        spec: &GameplayEffectSpec,
        captured: &HashMap<Atom, f32>,
        world: &World,
    ) -> Vec<GameplayModifierEvaluatedData> {
        let attack = captured.get(&"AttackPower".into()).copied().unwrap_or(0.0);
        let defense = captured.get(&"Defense".into()).copied().unwrap_or(0.0);
        let crit = captured.get(&"CritMultiplier".into()).copied().unwrap_or(1.0);
        
        let damage = (attack * 2.0 - defense).max(0.0) * crit;
        
        vec![GameplayModifierEvaluatedData {
            attribute: "Health".into(),
            modifier_op: ModifierOperation::Add,
            magnitude: -damage,
        }]
    }
}
```

---

### 4. Ability Triggers（自动触发）

UE GAS 支持 `FAbilityTriggerData`，允许 abilities 响应 gameplay events 或 tags 自动激活。

#### 当前实现状态

`src/abilities/triggers.rs` 已存在，但功能较基础。

#### 建议增强

```rust
// src/abilities/triggers.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbilityTriggerSource {
    /// Triggered by a gameplay event with matching tag
    GameplayEvent,
    /// Triggered when a tag is added to the owner
    OwnedTagAdded,
    /// Triggered when a tag is present on the owner
    OwnedTagPresent,
}

#[derive(Debug, Clone)]
pub struct AbilityTriggerData {
    pub trigger_tag: GameplayTag,
    pub trigger_source: AbilityTriggerSource,
}

// 在 AbilityDefinition 中添加
pub struct AbilityDefinition {
    // ... 现有字段
    
    /// Automatic triggers for this ability
    pub triggers: Vec<AbilityTriggerData>,
}
```

#### 系统实现

```rust
// src/abilities/trigger_systems.rs

pub fn process_ability_triggers_system(
    mut commands: Commands,
    ability_specs: Query<(Entity, &AbilitySpec, &AbilityOwner)>,
    registry: Res<AbilityRegistry>,
    mut gameplay_events: EventReader<GameplayEvent>,
    changed_tags: Query<(Entity, &OwnedTags), Changed<OwnedTags>>,
) {
    // Handle GameplayEvent triggers
    for event in gameplay_events.read() {
        for (spec_entity, spec, owner) in &ability_specs {
            if let Some(def) = registry.get(&spec.definition_id) {
                for trigger in &def.triggers {
                    if trigger.trigger_source == AbilityTriggerSource::GameplayEvent
                        && trigger.trigger_tag == event.event_tag
                    {
                        // Auto-activate ability
                        commands.trigger_targets(
                            TryActivateAbilityEvent {
                                ability_spec: spec_entity,
                                source: owner.entity,
                                target: event.target,
                            },
                            spec_entity,
                        );
                    }
                }
            }
        }
    }
    
    // Handle OwnedTagAdded triggers
    for (entity, tags) in &changed_tags {
        // Check for newly added tags and trigger abilities
    }
}
```

---

### 5. Ability Batching（批量激活）

UE GAS 支持 `FGameplayAbilityActivationInfo` 跟踪激活历史。

#### 建议增强

```rust
// src/abilities/components.rs

#[derive(Component, Debug, Clone)]
pub struct AbilityActivationHistory {
    /// Total number of times this ability has been activated
    pub activation_count: u32,
    /// Timestamp of last activation
    pub last_activation_time: f64,
    /// Result of last activation
    pub last_activation_result: ActivationResult,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivationResult {
    Success,
    Failed(ActivationFailureReason),
    Cancelled,
}
```

---

### 6. Cue Parameters（更丰富的 Cue 参数）

UE GAS 的 `FGameplayCueParameters` 包含更多上下文信息。

#### 建议增强

```rust
// src/cues/manager.rs

pub struct GameplayCueParameters {
    // ... 现有字段
    
    /// Normal vector at impact point
    pub normal_impact_normal: Option<Vec3>,
    /// Physical material at impact point
    pub physical_material: Option<String>,
    /// Level of the gameplay effect
    pub gameplay_effect_level: f32,
    /// Level of the ability
    pub ability_level: f32,
    /// Raw magnitude before clamping
    pub raw_magnitude: f32,
    /// Effect context (instigator, causer, etc.)
    pub effect_context: Option<GameplayEffectContext>,
}

pub struct GameplayEffectContext {
    pub instigator: Entity,
    pub effect_causer: Entity,
    pub ability_instance: Option<Entity>,
}
```

---

### 7. Prediction（客户端预测）

UE GAS 的核心特性之一是客户端预测（`FPredictionKey`），但你明确表示不需要联网功能，所以**可以跳过**。

---

## 三、优化建议

### 架构优化

#### 1. 引入 GameplayEffectComponent trait（高优先级）

**目标**：将 effect 功能模块化，提高可扩展性。

**实现步骤**：
1. 定义 `GameplayEffectComponent` trait
2. 在 `GameplayEffectDefinition` 中添加 `components: Vec<Arc<dyn GameplayEffectComponent>>`
3. 在 effect application/removal 系统中调用 component 回调
4. 实现核心 components：
   - `ImmunityComponent`
   - `ChanceToApplyComponent`
   - `AdditionalEffectsComponent`

**预期收益**：
- 用户可以自定义 components
- 减少 `GameplayEffectDefinition` 的字段数量
- 更符合 UE 5.3+ 的设计理念

#### 2. 添加 GameplayEffectQuery 系统（高优先级）

**目标**：统一 effect 匹配逻辑。

**实现步骤**：
1. 创建 `src/effects/query.rs`
2. 实现 `GameplayEffectQuery` struct 和 `matches()` 方法
3. 在 `ImmunityComponent`、`RemoveOtherEffectsComponent` 中使用

**预期收益**：
- 统一的 effect 查询接口
- 支持复杂的匹配条件（tags + definition + custom）

#### 3. 增强 ExecutionCalculation（中优先级）

**目标**：支持多 attribute 捕获和复杂计算。

**实现步骤**：
1. 创建 `src/effects/execution.rs`
2. 定义 `GameplayEffectExecutionCalculation` trait
3. 在 `MagnitudeCalculation` 中添加 `CustomCalculation` 变体
4. 在 effect application 系统中处理 execution calculation

**预期收益**：
- 支持复杂的伤害/治疗计算
- 可以实现 "攻击力 * 2 - 防御力" 这类公式

---

### 性能优化

#### 1. Attribute 查询缓存（低优先级）

当前每次查询 attribute 都需要遍历子实体。可以添加缓存：

```rust
#[derive(Resource)]
pub struct AttributeCache {
    // Entity -> HashMap<AttributeName, AttributeEntity>
    cache: HashMap<Entity, HashMap<Atom, Entity>>,
}

pub fn update_attribute_cache_system(
    mut cache: ResMut<AttributeCache>,
    attributes: Query<(Entity, &AttributeData, &Parent), Changed<AttributeData>>,
) {
    for (attr_entity, attr_data, parent) in &attributes {
        cache
            .entry(parent.get())
            .or_default()
            .insert(attr_data.name.clone(), attr_entity);
    }
}
```

#### 2. Effect Modifier 批处理（低优先级）

当前每个 modifier 单独处理，可以批量聚合同一 attribute 的 modifiers：

```rust
pub fn batch_aggregate_modifiers_system(
    mut attributes: Query<&mut AttributeData>,
    modifiers: Query<&EffectModifier>,
) {
    // Group modifiers by target attribute
    let mut modifier_groups: HashMap<Entity, Vec<&EffectModifier>> = HashMap::new();
    
    for modifier in &modifiers {
        modifier_groups
            .entry(modifier.target_attribute)
            .or_default()
            .push(modifier);
    }
    
    // Batch process each group
    for (attr_entity, modifiers) in modifier_groups {
        if let Ok(mut attr) = attributes.get_mut(attr_entity) {
            // Apply all modifiers in one pass
        }
    }
}
```

#### 3. Tag 查询优化（低优先级）

使用 `bevy_gameplay_tag` 的 hierarchical matching，但考虑缓存常用查询结果：

```rust
#[derive(Resource)]
pub struct TagQueryCache {
    // (Entity, TagRequirements) -> bool
    cache: HashMap<(Entity, u64), bool>, // u64 = hash of requirements
}
```

---

### 代码质量优化

#### 1. 减少 `Arc<dyn Trait>` 的使用（中优先级）

当前 `AbilityBehavior` 和未来的 `GameplayEffectComponent` 使用 `Arc<dyn Trait>`，可以考虑：

**方案 A：Enum dispatch**
```rust
pub enum GameplayEffectComponent {
    Immunity(ImmunityComponent),
    ChanceToApply(ChanceToApplyComponent),
    AdditionalEffects(AdditionalEffectsComponent),
    Custom(Arc<dyn CustomGameplayEffectComponent>),
}
```

**方案 B：Bevy 的 TypeId + HashMap 模式**
```rust
#[derive(Resource)]
pub struct GameplayEffectComponentRegistry {
    components: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}
```

**权衡**：
- Enum dispatch：性能更好，但扩展性差
- Arc<dyn Trait>：扩展性好，但有虚函数开销
- 建议：保持 `Arc<dyn Trait>`，性能差异在游戏规模下可忽略

#### 2. 统一错误处理（中优先级）

当前使用 `error!` + early return，可以引入统一的错误类型：

```rust
// src/core/error.rs

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GasError {
    AttributeNotFound(Atom),
    EffectDefinitionNotFound(Atom),
    AbilityDefinitionNotFound(Atom),
    InvalidState(String),
    RequirementNotMet(String),
}

pub type GasResult<T> = Result<T, GasError>;

impl std::fmt::Display for GasError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AttributeNotFound(name) => write!(f, "Attribute not found: {}", name),
            Self::EffectDefinitionNotFound(id) => write!(f, "Effect definition not found: {}", id),
            Self::AbilityDefinitionNotFound(id) => write!(f, "Ability definition not found: {}", id),
            Self::InvalidState(msg) => write!(f, "Invalid state: {}", msg),
            Self::RequirementNotMet(msg) => write!(f, "Requirement not met: {}", msg),
        }
    }
}

impl std::error::Error for GasError {}
```

**使用示例**：
```rust
pub fn apply_effect(
    effect_id: &Atom,
    target: Entity,
    registry: &GameplayEffectRegistry,
) -> GasResult<Entity> {
    let definition = registry
        .get(effect_id)
        .ok_or_else(|| GasError::EffectDefinitionNotFound(effect_id.clone()))?;
    
    // ...
    Ok(effect_entity)
}
```

#### 3. 增强文档（低优先级）

为每个 public API 添加示例代码：

```rust
/// Applies a gameplay effect to a target entity.
///
/// # Example
///
/// ```
/// use bevy_gameplay_ability_system::prelude::*;
///
/// fn apply_damage(
///     mut commands: Commands,
///     registry: Res<GameplayEffectRegistry>,
/// ) {
///     commands.trigger(ApplyGameplayEffectEvent {
///         effect_id: "damage_over_time".into(),
///         source: source_entity,
///         target: target_entity,
///         level: 1.0,
///     });
/// }
/// ```
pub struct ApplyGameplayEffectEvent {
    // ...
}
```

---

## 四、实现优先级建议

### 高优先级（核心功能增强）

1. **GameplayEffectComponent 系统**
   - 定义 trait 和基础架构
   - 实现 `ImmunityComponent`
   - 实现 `AdditionalEffectsComponent`
   - 预计工作量：3-5 天

2. **GameplayEffectQuery**
   - 实现查询匹配逻辑
   - 集成到 immunity 和 removal 系统
   - 预计工作量：1-2 天

3. **ExecutionCalculation**
   - 定义 trait 和 capture 机制
   - 实现多 attribute 捕获
   - 添加到 magnitude calculation
   - 预计工作量：2-3 天

### 中优先级（功能完善）

4. **Ability Triggers 增强**
   - 实现 `OwnedTagAdded` 和 `OwnedTagPresent` 触发
   - 添加自动激活系统
   - 预计工作量：1-2 天

5. **ChanceToApply Component**
   - 实现概率应用逻辑
   - 添加测试
   - 预计工作量：0.5-1 天

6. **RemoveOtherEffects Component**
   - 实现 effect 移除逻辑
   - 使用 `GameplayEffectQuery`
   - 预计工作量：1 天

### 低优先级（锦上添花）

7. **性能优化**
   - Attribute 查询缓存
   - Modifier 批处理
   - Tag 查询缓存
   - 预计工作量：2-3 天

8. **代码质量优化**
   - 统一错误处理
   - 增强文档
   - 减少 `Arc<dyn Trait>`（可选）
   - 预计工作量：2-3 天

---

## 五、总结

### 当前实现的优势

1. **架构设计优秀**：纯 ECS 设计，充分利用 Bevy 的特性
2. **测试覆盖完整**：127/127 tests passing（100% pass rate）
3. **代码质量高**：清晰的模块划分，良好的命名规范
4. **核心功能完整**：Attributes、Effects、Abilities、Cues 四大模块全部实现

### 主要差异

1. **组件化设计**：UE 5.3+ 的 `GameplayEffectComponent` 系统
2. **查询系统**：`GameplayEffectQuery` 用于复杂匹配
3. **执行计算**：`ExecutionCalculation` 支持多 attribute 捕获

### 建议行动

**短期（1-2 周）**：
- 实现 `GameplayEffectComponent` trait 和基础架构
- 实现 `ImmunityComponent` 和 `AdditionalEffectsComponent`
- 实现 `GameplayEffectQuery`

**中期（1 个月）**：
- 实现 `ExecutionCalculation`
- 增强 Ability Triggers
- 实现 `ChanceToApplyComponent` 和 `RemoveOtherEffectsComponent`

**长期（2-3 个月）**：
- 性能优化（缓存、批处理）
- 代码质量优化（错误处理、文档）
- 添加更多示例和教程

### 最终评价

当前实现已经非常接近 UE GAS 的核心功能，主要差异在于 UE 5.3+ 引入的**组件化设计**。建议优先实现 `GameplayEffectComponent` trait 和几个关键 components（Immunity、AdditionalEffects、ChanceToApply），这将大幅提升系统的灵活性和可扩展性。

继续保持当前的设计哲学（correctness over convenience、make illegal states unrepresentable），逐步添加上述功能即可。

---

**文档维护者**: Claude Code  
**最后更新**: 2026-04-30
