# Bevy GAS Development 复合技能设计文档

## 概述

创建一个复合技能 `bevy-gas-development`，整合 Bevy ECS、bevy_gameplay_tag 和 gameplay-abilities-system-rust-ecs 三个技能的内容，为 Bevy 游戏玩法开发提供统一的 AI 辅助。

## 目标

- 自动检测项目上下文，无需手动调用多个技能
- 提供连贯的知识体系，涵盖 Bevy ECS 架构、gameplay tags 和完整的 GAS 实现
- 减少用户认知负担，一个技能覆盖所有相关场景

## 设计方案

### 方案选择

采用**单一复合技能文件**方案，将三个技能的内容整合到一个 markdown 文件中。

理由：
- 游戏玩法开发通常同时涉及 ECS、tags 和 abilities，一次性加载所有知识更高效
- 自动触发机制简单可靠
- 维护成本低，只需更新一个文件
- 现代 LLM 上下文窗口足够处理大型技能文件

### 技能元数据

**技能名称**: `bevy-gas-development`

**技能描述**:
```
Comprehensive guide for Bevy game development with GameplayAbilitySystem (GAS). Combines Bevy ECS patterns, bevy_gameplay_tag hierarchical tag system, and complete GAS implementation (Attributes, GameplayEffects, GameplayAbilities, GameplayCues). Use when working on Bevy projects that implement gameplay systems, ability mechanics, tag-based logic, or attribute/effect systems.
```

**自动触发条件**:
1. 检测到 `Cargo.toml` 包含 `bevy` 依赖
2. 且满足以下任一条件：
   - 包含 `bevy_gameplay_tag` 依赖
   - 包含 `bevy_gameplay_ability_system` 依赖
   - 工作目录路径包含 "bevy_gameplay_ability_system"

### 内容组织结构

```
bevy-gas-development.md
├── 技能概述和使用场景
├── 第一模块：Bevy ECS 核心模式
│   ├── ECS 架构原则
│   ├── 组件设计模式
│   ├── 系统执行顺序
│   ├── UI 开发
│   ├── 构建策略
│   └── 常见陷阱
├── 第二模块：Gameplay Tags 系统
│   ├── 标签定义和层级匹配
│   ├── GameplayTagContainer 使用
│   ├── 标签计数机制
│   └── 标签事件系统
├── 第三模块：GameplayAbilitySystem 实现
│   ├── Attributes（属性系统）
│   │   ├── 双值模型（BaseValue/CurrentValue）
│   │   ├── AttributeSetDefinition trait
│   │   └── 修改器应用顺序
│   ├── GameplayEffects（效果系统）
│   │   ├── 定义和注册模式
│   │   ├── 持续时间策略
│   │   ├── 周期执行
│   │   ├── 堆叠机制
│   │   └── 标签需求
│   ├── GameplayAbilities（技能系统）
│   │   ├── 定义和注册模式
│   │   ├── 激活流程
│   │   ├── 消耗和冷却
│   │   └── 标签需求/阻塞/取消
│   └── GameplayCues（表现系统）
│       ├── 静态处理器（trait-based）
│       ├── Actor 处理器（entity-based）
│       └── 层级标签匹配
└── 集成指南和最佳实践
    ├── 系统执行顺序
    ├── Entity-per-thing 模式
    ├── 事件驱动通信
    ├── 测试模式
    └── 代码质量原则
```

### 内容来源

1. **Bevy 模块**：从现有 `bevy` 技能提取
   - ECS 架构和组件设计
   - 系统执行顺序和查询优化
   - UI 开发模式
   - 常见陷阱和最佳实践

2. **Gameplay Tags 模块**：从现有 `bevy-gameplay-tag` 技能提取
   - 层级标签系统概念
   - GameplayTagContainer API
   - 标签匹配规则（Exact/Partial/Any/All）
   - 标签计数和事件

3. **GAS 模块**：从现有 `gameplay-abilities-system-rust-ecs` 技能提取
   - 四大核心模块详细实现
   - ECS 架构翻译模式
   - 数据结构和 builder 模式
   - 系统执行流程

4. **集成指南**：新增内容
   - 如何在 ability 中使用 gameplay tags
   - 如何在 effect 中查询 attributes
   - 跨模块事件通信模式
   - 完整的测试示例

### 技能文件位置

根据 Claude Code 技能系统的约定，技能文件应放置在：
```
~/.claude/skills/bevy-gas-development/SKILL.md
```

或者如果使用插件系统：
```
~/.claude/plugins/<plugin-name>/skills/bevy-gas-development/SKILL.md
```

### 维护策略

- 当三个源技能之一更新时，同步更新复合技能对应部分
- 保持模块边界清晰，便于定位和更新
- 在文件头部注释标注各部分的来源技能

## 实现计划

1. 读取三个现有技能的完整内容
2. 按照设计的结构组织内容
3. 添加技能元数据和触发条件
4. 编写集成指南部分
5. 创建技能文件
6. 测试自动触发机制

## 验收标准

- 在 bevy_gameplay_ability_system 项目中自动触发
- 包含所有三个源技能的核心内容
- 内容组织清晰，易于查找
- 集成指南提供跨模块使用示例
