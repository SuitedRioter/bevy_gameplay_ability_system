# Attributes 模块设计文档

## 概述

Attributes 模块提供了一个灵活的属性系统，用于管理游戏实体的数值属性（如生命值、法力值等）。采用纯 ECS 架构，每个属性都是独立的实体。

## 设计理念

**最小化框架，最大化自由度**

- 框架只提供核心的属性存储和生命周期钩子机制
- 所有业务逻辑（如值限制、事件触发）由用户在钩子中实现
- 不强制任何特定的行为模式

## ECS 架构设计

### 实体关系

```
Owner Entity (玩家/怪物)
    └─ ChildOf ─> Attribute Entity (Health)
                    ├─ AttributeData (base_value, current_value)
                    ├─ AttributeName ("Health")
                    ├─ AttributeSetId (TypeId)
                    └─ AttributeMetadataComponent (min, max)
```

### 核心组件

#### 1. AttributeData
```rust
pub struct AttributeData {
    pub base_value: f32,      // 基础值（永久）
    pub current_value: f32,   // 当前值（临时修改后）
}
```

- `base_value`: 属性的基础值，不受临时效果影响
- `current_value`: 应用所有修改器后的最终值

#### 2. AttributeName
```rust
pub struct AttributeName(pub Atom);  // 使用 string_cache 优化
```

标识属性名称，使用 interned string 提高性能。

#### 3. AttributeSetId
```rust
pub struct AttributeSetId(pub TypeId);
```

标记属性属于哪个 AttributeSet，用于查找对应的生命周期钩子。

#### 4. ChildOf (Bevy 内置)

使用 Bevy 0.18 的 `ChildOf` 关系组件，通过 `set_parent_in_place()` 建立属性与所有者的父子关系。

### 为什么每个属性是独立实体？

1. **并行查询优化**: Bevy 的 ECS 可以并行处理不同的属性实体
2. **灵活的组件组合**: 可以为不同属性添加不同的组件
3. **高效的修改器系统**: 修改器可以直接查询目标属性实体
4. **符合 ECS 哲学**: 数据驱动，组件化设计

## 核心特性

### 1. AttributeSet 定义

通过实现 `AttributeSetDefinition` trait 定义一组相关属性：

```rust
struct CharacterAttributes;

impl AttributeSetDefinition for CharacterAttributes {
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
            "Health" => 100.0,
            "Mana" => 100.0,
            "Stamina" => 100.0,
            _ => 0.0,
        }
    }
}
```

### 2. 生命周期钩子

每个 AttributeSet 可以实现 4 个生命周期钩子：

```rust
impl AttributeSetDefinition for CharacterAttributes {
    // 在 current_value 修改前调用（可修改 new_value）
    fn pre_attribute_change(context: &mut AttributeModifyContext) {
        // 示例：限制值在范围内
        if let Some(meta) = Self::attribute_metadata(context.attribute_name.as_ref()) {
            context.new_value = meta.clamp(context.new_value);
        }

        // 示例：最小伤害为 1
        if context.new_value < context.old_value {
            let damage = context.old_value - context.new_value;
            if damage < 1.0 {
                context.new_value = context.old_value - 1.0;
            }
        }
    }

    // 在 current_value 修改后调用（只读）
    fn post_attribute_change(context: &AttributeModifyContext) {
        // 示例：检测死亡
        if context.attribute_name.as_ref() == "Health" && context.new_value <= 0.0 {
            // 触发死亡逻辑
        }

        // 示例：触发自定义事件
        // commands.trigger(MyAttributeChangedEvent { ... });
    }

    // 在 base_value 修改前调用（可修改 new_value）
    fn pre_attribute_base_change(context: &mut AttributeModifyContext) {
        // 用户自定义逻辑
    }

    // 在 base_value 修改后调用（只读）
    fn post_attribute_base_change(context: &AttributeModifyContext) {
        // 用户自定义逻辑
    }
}
```

#### AttributeModifyContext

```rust
pub struct AttributeModifyContext {
    pub owner: Entity,              // 属性所有者
    pub attribute: Entity,          // 属性实体
    pub attribute_name: Atom,       // 属性名称
    pub old_value: f32,            // 旧值
    pub new_value: f32,            // 新值（Pre 钩子可修改）
    pub source_effect: Option<Entity>, // 触发修改的效果实体
}
```

### 3. 钩子调用时机

钩子在 `aggregate_attribute_modifiers_system` 中被调用：

```
1. 计算所有修改器的聚合值
2. 创建 AttributeModifyContext
3. 调用 pre_attribute_change (可修改 new_value)
4. 应用修改: attr_data.current_value = context.new_value
5. 调用 post_attribute_change (只读)
```

**关键特性**：
- 同一帧内同步执行
- Pre 钩子可以修改即将应用的值
- Post 钩子用于响应变化（如触发事件、检测死亡）

## 使用指南

### 步骤 1: 添加插件

```rust
App::new()
    .add_plugins(AttributePlugin)
    .run();
```

### 步骤 2: 定义 AttributeSet

```rust
struct PlayerAttributes;

impl AttributeSetDefinition for PlayerAttributes {
    fn attribute_names() -> &'static [&'static str] {
        &["Health", "Mana"]
    }

    fn attribute_metadata(name: &str) -> Option<AttributeMetadata> {
        match name {
            "Health" => Some(
                AttributeMetadata::new("Health")
                    .with_min(0.0)
                    .with_max(100.0)
            ),
            "Mana" => Some(
                AttributeMetadata::new("Mana")
                    .with_min(0.0)
                    .with_max(100.0)
            ),
            _ => None,
        }
    }

    fn default_value(name: &str) -> f32 {
        match name {
            "Health" | "Mana" => 100.0,
            _ => 0.0,
        }
    }

    fn pre_attribute_change(context: &mut AttributeModifyContext) {
        // 自动 clamp 到 metadata 定义的范围
        if let Some(meta) = Self::attribute_metadata(context.attribute_name.as_ref()) {
            context.new_value = meta.clamp(context.new_value);
        }
    }

    fn post_attribute_change(context: &AttributeModifyContext) {
        // 检测死亡
        if context.attribute_name.as_ref() == "Health" && context.new_value <= 0.0 {
            info!("Player died!");
        }
    }
}
```

### 步骤 3: 注册钩子并创建属性

```rust
fn setup(mut commands: Commands, world: &mut World) {
    // 1. 注册钩子（启动时执行一次）
    PlayerAttributes::register_hooks(world);

    // 2. 创建玩家实体
    let player = commands.spawn_empty().id();

    // 3. 创建属性
    PlayerAttributes::create_attributes(&mut commands, player);
}
```

### 步骤 4: 查询和修改属性

```rust
// 查询属性
fn query_health(
    attributes: Query<(&AttributeData, &AttributeName, &ChildOf)>
) {
    for (data, name, child_of) in attributes.iter() {
        if name.as_str() == "Health" {
            info!("Health: {}/{}", data.current_value, data.base_value);
        }
    }
}

// 修改属性（直接修改会触发钩子）
fn damage_player(
    mut attributes: Query<(&mut AttributeData, &AttributeName)>
) {
    for (mut data, name) in attributes.iter_mut() {
        if name.as_str() == "Health" {
            data.current_value -= 10.0;
            // Pre/Post 钩子会在 aggregate_attribute_modifiers_system 中被调用
        }
    }
}
```

## 高级用法

### 多个 AttributeSet

不同实体可以使用不同的 AttributeSet：

```rust
struct PlayerAttributes;
struct MonsterAttributes;

impl AttributeSetDefinition for PlayerAttributes {
    // ... 玩家特定的属性和钩子
}

impl AttributeSetDefinition for MonsterAttributes {
    // ... 怪物特定的属性和钩子
}

fn setup(world: &mut World) {
    PlayerAttributes::register_hooks(world);
    MonsterAttributes::register_hooks(world);
}
```

每个 AttributeSet 的钩子互不干扰，通过 `AttributeSetId(TypeId)` 区分。

### 自定义事件

框架不提供内置事件，用户在钩子中触发自己的事件：

```rust
#[derive(Event)]
struct HealthChangedEvent {
    entity: Entity,
    old_health: f32,
    new_health: f32,
}

impl AttributeSetDefinition for MyAttributes {
    fn post_attribute_change(context: &AttributeModifyContext) {
        if context.attribute_name.as_ref() == "Health" {
            // 触发自定义事件
            // 注意：钩子中无法直接访问 Commands
            // 需要通过其他方式（如 EventWriter）触发
        }
    }
}
```

### 条件修改

在 Pre 钩子中实现复杂的条件逻辑：

```rust
fn pre_attribute_change(context: &mut AttributeModifyContext) {
    // 示例：护盾吸收伤害
    if context.attribute_name.as_ref() == "Health"
        && context.new_value < context.old_value
    {
        let damage = context.old_value - context.new_value;
        // 假设有护盾系统
        let absorbed = absorb_damage_with_shield(context.owner, damage);
        context.new_value = context.old_value - (damage - absorbed);
    }
}
```

## 设计决策

### 为什么使用 TypeId 而不是字符串标识 AttributeSet？

**原因**：类型安全 + 性能

- `TypeId` 是编译期确定的，零运行时开销
- 避免字符串比较
- 类型安全，不会拼错名字

## 性能考虑

1. **Interned Strings**: `AttributeName` 使用 `string_cache::Atom`，字符串比较是 O(1)
2. **并行查询**: 每个属性是独立实体，Bevy 可以并行处理
3. **零成本钩子**: 钩子是函数指针，调用开销极小
4. **按需执行**: 只有被修改的属性才会触发钩子

## 与其他模块的集成

### 与 Effects 模块

Effects 模块通过 `aggregate_attribute_modifiers_system` 修改属性：

```
GameplayEffect -> AttributeModifier -> aggregate_attribute_modifiers_system
                                       -> Pre Hook -> 修改值 -> Post Hook
```

### 与 Abilities 模块

Abilities 可以查询属性来检查消耗：

```rust
fn can_afford_cost(
    attributes: Query<(&AttributeData, &AttributeName, &ChildOf)>
) -> bool {
    // 检查是否有足够的 Mana
}
```

## 示例代码

完整示例见 `examples/attribute_lifecycle.rs`

## 总结

Attributes 模块提供了：
- ✅ 纯 ECS 架构的属性系统
- ✅ 灵活的生命周期钩子
- ✅ 最小化框架约束
- ✅ 高性能设计
- ✅ 类型安全的 AttributeSet 系统

用户可以在钩子中实现任何自定义逻辑，框架不做任何假设。
