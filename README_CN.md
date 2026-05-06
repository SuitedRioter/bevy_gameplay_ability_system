# Bevy 游戏玩法能力系统

一个全面的 Bevy 游戏玩法能力系统，灵感来自虚幻引擎的 GameplayAbilitySystem (GAS)。本库提供了一个灵活且强大的框架，用于实现 RPG 风格的能力、属性和效果，采用纯 ECS 架构。

[![Crates.io](https://img.shields.io/crates/v/bevy_gameplay_ability_system.svg)](https://crates.io/crates/bevy_gameplay_ability_system)
[![Docs](https://docs.rs/bevy_gameplay_ability_system/badge.svg)](https://docs.rs/bevy_gameplay_ability_system)
[![License](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/yourusername/bevy_gameplay_ability_system#license)

## 功能特性

- **属性系统**：定义自定义属性集，包含 6 个生命周期钩子（匹配 UE 的 AttributeSet 回调）
- **游戏玩法效果**：使用即时、持续时间或无限效果修改属性
  - 10 个评估通道用于复杂的堆叠规则
  - 周期性执行用于持续伤害/治疗
  - 自定义数值计算和执行计算
  - GameplayEffect 组件用于模块化行为扩展
- **游戏玩法能力**：实现具有消耗、冷却和激活要求的能力
  - 3 种实例化策略（NonInstanced、InstancedPerActor、InstancedPerExecution）
  - 12+ 种内置能力任务类型用于异步操作
  - 基于标签的激活要求、阻塞和取消
- **游戏玩法提示**：具有层级标签匹配的视觉和音频反馈系统
  - 静态提示（轻量级，无实体）和 Actor 提示（生成的实体）
  - 专用提示类型（Burst、Looping、HitImpact）
- **基于标签的系统**：使用 `bevy_gameplay_tag` 进行灵活的层级标签匹配
- **纯 ECS 架构**：充分利用 Bevy 的 ECS 实现性能和灵活性
- **基于实体的设计**：属性、效果和能力都是独立的实体
- **Bevy 0.18 集成**：使用 ChildOf 关系实现自动清理

## 文档

- [API 文档](https://docs.rs/bevy_gameplay_ability_system)
- [完整 RPG 示例](examples/complete_rpg.rs) - 完整战斗系统演示
- [压力测试示例](examples/stress_test.rs) - 性能测试工具

## Bevy 兼容性

| Bevy 版本 | 插件版本 |
| --------- | -------- |
| 0.18      | 0.1      |

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
bevy = "0.18.1"
bevy_gameplay_ability_system = "0.1"
bevy_gameplay_tag = "0.2.0"
```

## 快速开始

```rust
use bevy::prelude::*;
use bevy_gameplay_ability_system::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GasPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    // 创建一个带属性的实体
    let player = commands.spawn_empty().id();

    // 使用自定义属性集创建属性
    CharacterAttributes::create_attributes(&mut commands, player);
}

// 定义自定义属性集
struct CharacterAttributes;

impl AttributeSetDefinition for CharacterAttributes {
    fn attribute_names() -> &'static [&'static str] {
        &["Health", "Mana", "Stamina"]
    }

    fn attribute_metadata(name: &str) -> Option<AttributeMetadata> {
        match name {
            "Health" => Some(AttributeMetadata {
                name: "Health",
                min_value: Some(0.0),
                max_value: Some(100.0),
            }),
            "Mana" => Some(AttributeMetadata {
                name: "Mana",
                min_value: Some(0.0),
                max_value: Some(100.0),
            }),
            "Stamina" => Some(AttributeMetadata {
                name: "Stamina",
                min_value: Some(0.0),
                max_value: Some(100.0),
            }),
            _ => None,
        }
    }

    fn default_value(name: &str) -> f32 {
        match name {
            "Health" => 100.0,
            "Mana" => 100.0,
            "Stamina" => 100.0,
            _ => 0.0,
        }
    }
}
```

## 架构

系统基于四个核心模块构建：

### 1. 属性

属性系统提供双值模型（BaseValue/CurrentValue），具有自动修改器聚合和 6 个生命周期钩子。

- **BaseValue**：永久值，由即时效果修改
- **CurrentValue**：临时值，由持续时间/无限效果修改
- **修改器**：按顺序应用：Add → Multiply → Override
- **生命周期钩子**：6 个钩子匹配 UE 的 AttributeSet 回调
  - `pre_effect_execute` / `post_effect_execute`：即时效果应用
  - `pre_attribute_change` / `post_attribute_change`：当前值变化
  - `pre_attribute_base_change` / `post_attribute_base_change`：基础值变化

```rust
// 属性是带组件的实体
#[derive(Component)]
pub struct AttributeData {
    pub base_value: f32,
    pub current_value: f32,
}

// 每个属性通过 ChildOf 关系链接到其所有者
commands.spawn((
    AttributeData { base_value: 100.0, current_value: 100.0 },
    AttributeName("Health".into()),
    AttributeSetId(TypeId::of::<CharacterAttributes>()),
)).set_parent_in_place(owner_entity);
```

### 2. 游戏玩法效果

效果修改属性，可以是即时的、基于持续时间的或无限的。

```rust
// 创建效果定义
let damage_effect = GameplayEffectDefinition::new("effect.damage.fire")
    .with_duration_policy(DurationPolicy::Instant)
    .add_modifier(ModifierInfo {
        attribute_name: "Health".to_string(),
        operation: ModifierOperation::AddBase,
        magnitude: MagnitudeCalculation::ScalableFloat { base_value: -20.0 },
    });

// 将效果应用到目标
commands.spawn((
    ActiveGameplayEffect {
        definition_id: "effect.damage.fire".to_string(),
        level: 1,
        start_time: 0.0,
        stack_count: 1,
    },
    EffectTarget(target_entity),
));
```

**效果特性：**

- 持续时间策略：Instant、HasDuration、Infinite
- 周期性执行（持续伤害/治疗）
- 堆叠策略：Independent、RefreshDuration、StackCount
- 应用的标签要求
- 激活时授予的标签
- 10 个评估通道（Channel0-Channel9）用于复杂堆叠规则
- GameplayEffect 组件用于模块化行为扩展
- 自定义数值计算和执行计算

### 3. 游戏玩法能力

能力是玩家激活的动作，具有消耗、冷却和要求。

```rust
// 定义能力（标签方法需要 &Res<GameplayTagsManager>）
let fireball = AbilityDefinition::new("ability.fireball")
    .with_instancing_policy(InstancingPolicy::InstancedPerExecution)
    .add_activation_required_tag(GameplayTag::new("State.Alive"), &tags_manager)
    .add_activation_blocked_tag(GameplayTag::new("State.Stunned"), &tags_manager)
    .with_cost_effect("effect.cost.mana")
    .with_cooldown_effect("effect.cooldown.fireball");

// 将能力授予实体
commands.spawn((
    AbilitySpec {
        definition_id: "ability.fireball".to_string(),
        level: 1,
        input_id: None,
        is_active: false,
    },
    AbilityOwner(player_entity),
));
```

**能力特性：**

- 实例化策略：NonInstanced、InstancedPerActor、InstancedPerExecution
- 消耗效果（法力、耐力等）
- 冷却效果（基于标签）
- 标签要求和阻塞
- 激活事件
- 12+ 种内置能力任务类型：
  - WaitDelayTask、WaitGameplayEventTask、WaitTargetDataTask
  - WaitAttributeChangeTask、WaitGameplayEffectAppliedTask、WaitGameplayEffectRemovedTask
  - ApplyEffectToTargetDataTask、ApplyRootMotionTask、PlayMontageTask
  - SpawnProjectileTask、RepeatTask、WaitInputPressTask、WaitInputReleaseTask

### 4. 游戏玩法提示

提示为游戏玩法事件提供视觉和音频反馈。

```rust
// 注册静态提示
let mut cue_manager = world.resource_mut::<GameplayCueManager>();
cue_manager.register_static_cue(GameplayTag::new("GameplayCue.Damage.Fire"));

// 触发提示
commands.trigger(TriggerGameplayCueEvent {
    cue_tag: GameplayTag::new("GameplayCue.Damage.Fire"),
    event_type: GameplayCueEvent::Executed,
    parameters: GameplayCueParameters::default(),
});
```

**提示特性：**

- 静态提示（轻量级，无实体）
- Actor 提示（生成的实体，具有生命周期）
- 专用提示类型：BurstCue、LoopingCue、HitImpactCue
- 层级标签匹配
- 批处理以提高性能
- 事件类型：OnActive、WhileActive、Executed、Removed

## 核心概念

### 基于实体的设计

与传统的基于组件的方法不同，本系统对属性、能力和效果使用实体：

```rust
// 每个属性是通过 ChildOf 链接的实体
let attribute = commands.spawn((
    AttributeData { base_value: 100.0, current_value: 100.0 },
    AttributeName("Health".into()),
)).set_parent_in_place(owner).id();

// 每个活跃效果是一个实体
let effect_entity = commands.spawn((
    ActiveGameplayEffect { /* ... */ },
    EffectTarget(target),
    EffectDuration { remaining: 5.0, total: 5.0 },
)).id();

// 每个授予的能力是一个实体
let ability_entity = commands.spawn((
    AbilitySpec { /* ... */ },
    AbilityOwner(owner),
)).id();
```

**优势：**

- 更好的 ECS 性能（查询优化）
- 并行系统执行
- 通过 ChildOf 关系自动清理（Bevy 0.18）
- 内存局部性
- 更容易使用自定义组件扩展

### 标签要求

标签要求控制何时可以应用/激活效果和能力：

```rust
use bevy_gameplay_tag::GameplayTagRequirements;

let mut requirements = GameplayTagRequirements::new();

// 必须具有这些标签
requirements.require_tags.add_tag(
    GameplayTag::new("Ability.Skill"),
    &tags_manager
);

// 不能具有这些标签
requirements.ignore_tags.add_tag(
    GameplayTag::new("State.Stunned"),
    &tags_manager
);
```

### 评估通道

10 个评估通道（Channel0-Channel9）实现复杂的堆叠规则：

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

## 使用指南

### 1. 添加插件

```rust
use bevy::prelude::*;
use bevy_gameplay_ability_system::prelude::*;
use bevy_gameplay_tag::GameplayTagsPlugin;

App::new()
    .add_plugins(DefaultPlugins)
    .add_plugins(GameplayTagsPlugin::with_data_path(
        "assets/gameplay_tags.json".to_string()
    ))
    .add_plugins(GasPlugin)
    .run();
```

### 2. 定义属性集

```rust
struct PlayerAttributes;

impl AttributeSetDefinition for PlayerAttributes {
    fn attribute_names() -> &'static [&'static str] {
        &["Health", "Mana", "Stamina"]
    }

    fn attribute_metadata(name: &str) -> Option<AttributeMetadata> {
        match name {
            "Health" => Some(
                AttributeMetadata::new("Health")
                    .with_min(0.0)
                    .with_max(100.0)
            ),
            _ => None,
        }
    }

    fn default_value(name: &str) -> f32 {
        match name {
            "Health" | "Mana" | "Stamina" => 100.0,
            _ => 0.0,
        }
    }

    // 可选：实现生命周期钩子
    fn pre_attribute_change(context: &mut AttributeModifyContext) {
        // 自动限制到元数据定义的范围
        if let Some(meta) = Self::attribute_metadata(context.attribute_name.as_ref()) {
            context.new_value = meta.clamp(context.new_value);
        }
    }

    fn post_attribute_change(context: &AttributeModifyContext) {
        // 检测死亡
        if context.attribute_name.as_ref() == "Health" && context.new_value <= 0.0 {
            info!("玩家死亡！");
        }
    }
}
```

### 3. 创建效果

```rust
fn setup(mut commands: Commands, mut registry: ResMut<GameplayEffectRegistry>) {
    // 即时伤害效果
    let damage = GameplayEffectDefinition::new("damage")
        .with_duration_policy(DurationPolicy::Instant)
        .add_modifier(ModifierInfo::new(
            "Health",
            ModifierOperation::AddCurrent,
            MagnitudeCalculation::scalar(-20.0),
        ));
    
    registry.register(damage);

    // 持续治疗效果
    let heal_over_time = GameplayEffectDefinition::new("heal_over_time")
        .with_duration(10.0)
        .with_period(1.0)
        .add_modifier(ModifierInfo::new(
            "Health",
            ModifierOperation::AddCurrent,
            MagnitudeCalculation::scalar(5.0),
        ));
    
    registry.register(heal_over_time);
}
```

### 4. 定义能力

```rust
fn setup_abilities(
    mut commands: Commands,
    mut registry: ResMut<AbilityRegistry>,
    tags_manager: Res<GameplayTagsManager>,
) {
    let fireball = AbilityDefinition::new("fireball")
        .with_instancing_policy(InstancingPolicy::InstancedPerExecution)
        .add_activation_required_tag(
            GameplayTag::new("State.Alive"),
            &tags_manager
        )
        .add_activation_blocked_tag(
            GameplayTag::new("State.Stunned"),
            &tags_manager
        )
        .with_cost_effect("mana_cost")
        .with_cooldown_effect("fireball_cooldown");
    
    registry.register(fireball);
}
```

### 5. 利用系统集

将自定义系统添加到适当的集合：

```rust
app.add_systems(
    Update,
    my_custom_ability_logic.in_set(GasSystemSet::Abilities)
);
```

## 性能考虑

- **基于实体的设计**实现并行系统执行
- 通过 Bevy 的原型系统进行**查询优化**
- **变更检测**最小化不必要的更新
- **提示批处理**减少视觉效果的开销
- **ChildOf 关系**提供自动清理（Bevy 0.18）
- **内部字符串**（string_cache::Atom）用于高效查找

## 项目状态

✅ **单人游戏生产就绪** - 所有核心系统完成，具有全面的测试覆盖。

### 测试覆盖

**总计：127/127 测试通过（100% 通过率）✅**

- 单元测试：41/41 通过 ✅
- 集成测试：81/81 通过 ✅
  - `ability_granting_lifecycle_test`：1 个测试
  - `ability_task_test`：12 个测试（所有任务类型）
  - `application_requirement_test`：2 个测试（自定义要求）
  - `attribute_aggregation_test`：2 个测试
  - `enhanced_requirements_test`：4 个测试（基于百分比、源 vs 目标、标签、等级范围）
  - `evaluation_channel_test`：3 个测试（通道评估顺序、同通道组合、复杂堆叠）
  - `gameplay_effect_spec_test`：2 个测试
  - `instancing_policy_test`：3 个测试（NonInstanced、InstancedPerActor、InstancedPerExecution）
  - `periodic_effect_spec_test`：2 个测试
  - `stack_count_test`：2 个测试
  - `stacking_reapply_spec_test`：2 个测试
  - 专用提示、输入任务、动态数值等的其他测试
- 文档测试：5/5 通过 ✅

### 已完成功能

- ✅ 具有 6 个生命周期钩子的属性系统（匹配 UE 的 AttributeSet 回调）
- ✅ 游戏玩法效果（即时、持续时间、无限、周期性）
- ✅ 效果堆叠策略（Independent、RefreshDuration、StackCount）
- ✅ 10 个评估通道用于复杂堆叠规则
- ✅ GameplayEffect 组件用于模块化行为扩展
- ✅ 具有 3 种实例化策略的能力激活
- ✅ 12+ 种能力任务类型用于异步操作
- ✅ 具有专用类型的游戏玩法提示（Burst、Looping、HitImpact）
- ✅ 基于标签的要求和阻塞
- ✅ 自定义应用要求和数值计算
- ✅ 具有规格持久化的 SetByCaller 数值

### 已知限制

- 仅限单人游戏（无网络/复制）
- 性能优化延后（当前设计处理 <50 个实体，每个实体 <10 个属性）
- Bevy 0.18 的基准测试套件损坏（criterion 兼容性问题）

## 示例

查看 `examples/` 目录以获取完整示例：

- `basic_attributes.rs` - 基本属性系统使用
- `ability_activation.rs` - 能力激活流程
- `gameplay_effects.rs` - 效果应用和堆叠
- `complete_rpg.rs` - 完整的战斗系统演示
- `stress_test.rs` - 性能测试工具

运行示例：

```bash
cargo run --example complete_rpg
```

## 贡献

欢迎贡献！请随时提交 Pull Request。

## 许可证

根据以下任一许可证授权：

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) 或 http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) 或 http://opensource.org/licenses/MIT)

由您选择。

## 致谢

本库受虚幻引擎的 GameplayAbilitySystem (GAS) 启发，并针对 Bevy 的 ECS 架构进行了改编。特别感谢：

- Epic Games 的原始 GAS 设计
- Bevy 社区提供的优秀游戏引擎
- `bevy_gameplay_tag` 的贡献者

## 资源

- [Bevy 引擎](https://bevyengine.org/)
- [虚幻引擎 GAS 文档](https://docs.unrealengine.com/en-US/gameplay-ability-system-for-unreal-engine/)
- [bevy_gameplay_tag](https://github.com/SuitedRioter/bevy_gameplay_tag)
