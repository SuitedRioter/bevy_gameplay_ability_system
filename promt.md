我想的Ability模块设计。
1. Ability定义类，为一个struct。放的是能力的配置数据。
2. 当把一个Ability定义类赋予玩家时，就生成一个AbilitySpec的实体，这个实体代表玩家身上的这个技能，内部包含组件如下：AbilitySpec，AbilityActiveState（内部包含是否激活，激活次数信息），AbilityCooldown，AbilityOwner
3. 当一个AbilitySpec的实体激活时，就生成一个AbilitySpecInstance实体（这个我想作为AbilitySpec的子实体实现，这样我在移除AbilitySpec能力时，就可以把他对应的多个AbilitySpecInstance实体，
但是需要在销毁AbilitySpecInstance实体时调用behavior.end方法），他内部要有包含：bIsActive：标记当前实例是否活跃 bIsBlockingOtherAbilities：控制是否阻止其他能力 bIsCancelable：控制当前实例是否可取消这三个属性的组件，用于激活过程中的逻辑控制。另外AbilitySpecInstance实体本质是Ability定义类的复制。用于在激活过程中调用behavior的逻辑。
4. 方法的参数尽量不要使用world，这个会降低性能，除非只能通过world来实现。


我需要优化现有abilities模块的system和监听system的设计。
1. on_try_activate_ability 这个监听入口需要改一下，生成的AbilitySpec Entity需要是激活者的child entity，这样激活者Entity被移除时，能自动移除子实体。
2.


这个（/Users/zhengwei/GeneralProject/UnrealEngine/Engine/Plugins/Runtime/GameplayAbilities）文件夹下是UnrealEngine的GAS插件的代码，我希望你在当前项目使用bevy来实现UnrealEngine的GAS插件的功能，
对外功能表现必须与原模块一致，你需要把原模块oop思想的代码已bevy的ecs思想实现，现有项目已经实现了一部分代码。你可以参考并优化（当需要优化的时候）。我希望你参考已经完成的中文设计文档（在./docs/design_document_cn.md），然后再进行复刻,注意，我已经自己实现了GameplayTag的功能（源码在/Users/zhengwei/RustProject/bevy_gameplay_tag），相关需要使用GameplayTag，GameplayTagContainer，GameplayTagCountContainer的直接使用。如果需要查看bevy的api，可以分析bevy源码（源码在/Users/zhengwei/RustProject/bevy）


这个（/Users/zhengwei/GeneralProject/UnrealEngine/Engine/Plugins/Runtime/GameplayAbilities）文件夹下是UnrealEngine的GAS插件的代码，我希望你在当前项目使用bevy来实现UnrealEngine的GAS插件的功能，
对外功能表现必须与原模块一致，你需要把原模块oop思想的代码已bevy的ecs思想实现，现有项目已经实现了一部分代码。你可以参考并优化（当需要优化的时候）。注意，我已经自己实现了GameplayTag的功能（源码在/Users/zhengwei/RustProject/bevy_gameplay_tag），相关需要使用GameplayTag，GameplayTagContainer，GameplayTagCountContainer的直接使用。如果需要查看bevy的api，可以分析bevy源码（源码在/Users/zhengwei/RustProject/bevy），我不需要实现联网功能，目标是做单机游戏。
在写代码之前：
1. 先解释架构映射关系
2. 识别等价实现
3. 标出潜在风险点
【转换铁律】
1. 不再有 Actor、Component、UObject。一切都是实体(Entity)加纯数据组件(Component)。
2. 任何有状态的变量都必须成为组件，不可内聚在系统里。
3. 所有函数实现（包括虚函数）都变为系统，通过 Query 组合获取数据依赖。
4. 继承树被扁平化，差异用标记组件或枚举字段代替。


当前实现评估

  你的项目已经实现了 GAS 的核心功能，架构设计优秀：

  ✅ 已实现的核心功能：
  - 完整的 Attribute 系统（双值模型、modifier aggregation）
  - GameplayEffect 系统（三种 duration policy、periodic、stacking、evaluation channels）
  - Ability 系统（三种 instancing policy、activation flow、tag-based requirements）
  - GameplayCue 系统（static/actor handlers、hierarchical matching）
  - Ability Tasks（12 种任务类型）
  - 高级特性（custom application requirements、attribute-based magnitudes、snapshot/dynamic capture）

  架构优势：
  - 纯 ECS 设计，充分利用 Bevy 的并行查询和 observer 模式
  - Entity-per-thing 模式（attributes、effects、abilities 都是独立 entity）
  - 清晰的系统执行顺序（system sets）
  - 良好的测试覆盖率（127/127 tests passing）

  与 UE GAS 的功能差异

  1. GameplayEffect Components（模块化组件系统）

  UE GAS 5.3+ 的重大改进：
  UE 从 5.3 开始将 monolithic UGameplayEffect 拆分为可组合的 UGameplayEffectComponent，包括：

  - TargetTagsGameplayEffectComponent - 授予 tags 给目标
  - AbilitiesGameplayEffectComponent - 授予 abilities（你已实现）
  - ImmunityGameplayEffectComponent - 免疫其他 effects
  - ChanceToApplyGameplayEffectComponent - 概率应用
  - AdditionalEffectsGameplayEffectComponent - 触发额外 effects（OnApplication/OnComplete）
  - BlockAbilityTagsGameplayEffectComponent - 阻止 abilities
  - RemoveOtherGameplayEffectComponent - 移除其他 effects
  - CustomCanApplyGameplayEffectComponent - 自定义应用条件

  建议：
  // 在 src/effects/components.rs 中添加组件化设计
  pub trait GameplayEffectComponent: Send + Sync {
      fn on_effect_applied(&self, effect: Entity, target: Entity, world: &mut World);
      fn on_effect_removed(&self, effect: Entity, target: Entity, world: &mut World);
      fn can_apply(&self, spec: &GameplayEffectSpec, target: Entity, world: &World) -> bool;
  }

  // 在 GameplayEffectDefinition 中添加
  pub struct GameplayEffectDefinition {
      // ... 现有字段
      pub components: Vec<Arc<dyn GameplayEffectComponent>>,
  }

  // 实现具体组件
  pub struct ImmunityComponent {
      pub immunity_queries: Vec<EffectQuery>,
  }

  pub struct ChanceToApplyComponent {
      pub chance: f32,
  }

  pub struct AdditionalEffectsComponent {
      pub on_application: Vec<Atom>,
      pub on_complete_always: Vec<Atom>,
      pub on_complete_normal: Vec<Atom>,
      pub on_complete_prematurely: Vec<Atom>,
  }

  2. GameplayEffectQuery（效果查询系统）

  UE GAS 使用 FGameplayEffectQuery 进行复杂的 effect 匹配（用于 immunity、removal、conditional application）。

  建议：
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
          // 实现匹配逻辑
      }
  }

  3. GameplayEffectExecutionCalculation（执行计算）

  UE GAS 支持复杂的 UGameplayEffectExecutionCalculation，可以捕获多个 attributes 并执行自定义计算逻辑（比你当前的 AttributeBased magnitude 更强大）。

  建议：
  // src/effects/execution.rs
  pub trait GameplayEffectExecutionCalculation: Send + Sync {
      /// 定义需要捕获的 attributes
      fn relevant_attributes_to_capture(&self) -> Vec<AttributeCaptureDefinition>;

      /// 执行计算
      fn execute(
          &self,
          spec: &GameplayEffectSpec,
          captured_attributes: &HashMap<Atom, f32>,
          world: &World,
      ) -> Vec<GameplayModifierEvaluatedData>;
  }

  pub struct AttributeCaptureDefinition {
      pub attribute_name: Atom,
      pub capture_source: AttributeCaptureSource,
      pub snapshot: bool,
  }

  // 在 MagnitudeCalculation 中添加
  pub enum MagnitudeCalculation {
      // ... 现有变体
      CustomCalculation {
          calculation: Arc<dyn GameplayEffectExecutionCalculation>,
      },
  }

  4. Ability Triggers（自动触发）

  UE GAS 支持 FAbilityTriggerData，允许 abilities 响应 gameplay events 或 tags 自动激活。

  建议：
  // src/abilities/triggers.rs 已存在，但需要增强
  pub enum AbilityTriggerSource {
      GameplayEvent,
      OwnedTagAdded,
      OwnedTagPresent,
  }

  pub struct AbilityTriggerData {
      pub trigger_tag: GameplayTag,
      pub trigger_source: AbilityTriggerSource,
  }

  // 在 AbilityDefinition 中添加
  pub struct AbilityDefinition {
      // ... 现有字段
      pub triggers: Vec<AbilityTriggerData>,
  }

  5. Ability Batching（批量激活）

  UE GAS 支持 FGameplayAbilitySpecHandle 批量操作和 FGameplayAbilityActivationInfo 跟踪激活历史。

  当前实现已足够，但可优化：
  // src/abilities/components.rs
  #[derive(Component)]
  pub struct AbilityActivationHistory {
      pub activation_count: u32,
      pub last_activation_time: f64,
      pub last_activation_result: ActivationResult,
  }

  6. Cue Parameters（更丰富的 Cue 参数）

  UE GAS 的 FGameplayCueParameters 包含更多上下文信息（hit result、physical material、magnitude 等）。

  建议增强：
  // src/cues/manager.rs
  pub struct GameplayCueParameters {
      // ... 现有字段
      pub normal_impact_normal: Option<Vec3>,
      pub physical_material: Option<String>,
      pub gameplay_effect_level: f32,
      pub ability_level: f32,
      pub raw_magnitude: f32,
      pub effect_context: Option<GameplayEffectContext>,
  }

  7. Prediction（客户端预测）

  UE GAS 的核心特性之一是客户端预测（FPredictionKey），但你明确表示不需要联网功能，所以可以跳过。

  优化建议

  架构优化

  1. 引入 GameplayEffectComponent trait
    - 将 effect 功能模块化（immunity、chance、additional effects）
    - 提高可扩展性，用户可以自定义 components
  2. 添加 GameplayEffectQuery 系统
    - 统一 effect 匹配逻辑
    - 支持 immunity、conditional removal、effect queries
  3. 增强 ExecutionCalculation
    - 支持多 attribute 捕获
    - 允许复杂的自定义计算逻辑

  性能优化

  当前设计已经很好地利用了 ECS，但可以考虑：

  1. Attribute 查询缓存
  #[derive(Resource)]
  pub struct AttributeCache {
      // Entity -> HashMap<AttributeName, AttributeEntity>
      cache: HashMap<Entity, HashMap<Atom, Entity>>,
  }
  2. Effect Modifier 批处理
    - 当前每个 modifier 单独处理，可以批量聚合同一 attribute 的 modifiers
  3. Tag 查询优化
    - 使用 bevy_gameplay_tag 的 hierarchical matching，但考虑缓存常用查询结果

  代码质量优化

  1. 减少 Arc<dyn Trait> 的使用
    - AbilityBehavior 和未来的 GameplayEffectComponent 可以考虑使用 enum dispatch
    - 或者使用 Bevy 的 TypeId + HashMap 模式
  2. 统一错误处理
  pub enum GasError {
      AttributeNotFound(Atom),
      EffectDefinitionNotFound(Atom),
      AbilityDefinitionNotFound(Atom),
      InvalidState(String),
  }

  pub type GasResult<T> = Result<T, GasError>;
  3. 增强文档
    - 为每个 public API 添加示例代码
    - 创建 architecture decision records (ADRs)

  实现优先级建议

  高优先级（核心功能增强）：
  1. GameplayEffectComponent 系统（模块化设计）
  2. GameplayEffectQuery（effect 匹配）
  3. ExecutionCalculation（复杂计算）

  中优先级（功能完善）：
  4. Ability Triggers 增强（自动激活）
  5. AdditionalEffects Component（OnComplete effects）
  6. Immunity Component（effect 免疫）

  低优先级（锦上添花）：
  7. ChanceToApply Component（概率应用）
  8. RemoveOtherEffects Component（effect 移除）
  9. 性能优化（缓存、批处理）

  总结

  你的实现已经非常接近 UE GAS 的核心功能，主要差异在于 UE 5.3+ 引入的组件化设计。建议优先实现 GameplayEffectComponent trait 和几个关键 components（Immunity、AdditionalEffects、ChanceToApply），这将大幅提升系统的灵活性和可扩展性。

  当前代码质量很高，测试覆盖率完整，ECS 架构设计优秀。继续保持这种设计哲学，逐步添加上述功能即可。


---
  🔴 未实现的核心功能

  1. GameplayEffect Components (UE 5.3+ 模块化系统)

  UE GAS:
  // UE 5.3 引入的模块化 GE 系统
  UGameplayEffectComponent (基类)
    ├─ AbilitiesGameplayEffectComponent        // 授予 Ability
    ├─ AdditionalEffectsGameplayEffectComponent // 触发额外 Effect
    ├─ AssetTagsGameplayEffectComponent        // 资产标签
    ├─ BlockAbilityTagsGameplayEffectComponent // 阻止 Ability
    ├─ CancelAbilityTagsGameplayEffectComponent // 取消 Ability
    ├─ ChanceToApplyGameplayEffectComponent    // 概率应用
    ├─ CustomCanApplyGameplayEffectComponent   // 自定义应用条件
    ├─ ImmunityGameplayEffectComponent         // 免疫系统
    ├─ RemoveOtherGameplayEffectComponent      // 移除其他 Effect
    ├─ TargetTagRequirementsGameplayEffectComponent // 目标标签需求
    └─ TargetTagsGameplayEffectComponent       // 目标标签

  Bevy GAS 现状:
  - ❌ 完全未实现 模块化组件系统
  - ✅ 部分功能已内置在 GameplayEffectDefinition 中（如 granted_abilities, granted_tags）
  - ⚠️ 缺少：概率应用、免疫系统、条件触发额外 Effect

  影响:
  - 无法实现复杂的 Effect 组合逻辑（如"50% 概率触发额外伤害"）
  - 无法实现免疫系统（如"免疫所有 Stun 效果"）

  ---
  2. Ability Task 系统（部分缺失）

  UE GAS 有 42 个 Task，Bevy GAS 实现了 12 个:

  ┌───────────────────────────┬───────────┬──────────────────────────┐
  │              UE Task              │ Bevy 实现 │           说明           │
  ├───────────────────────────────────┼───────────┼──────────────────────────┤
  │ WaitDelay                         │ ✅        │ 等待延迟                 │
  ├───────────────────────────────────┼───────────┼──────────────────────────┤
  │ WaitGameplayEvent                 │ ✅        │ 等待游戏事件             │
  ├───────────────────────────────────┼───────────┼──────────────────────────┤
  │ WaitAttributeChange               │ ✅        │ 等待属性变化             │
  ├───────────────────────────────────┼───────────┼──────────────────────────┤
  │ WaitGameplayEffectApplied         │ ✅        │ 等待 Effect 应用         │
  ├───────────────────────────────────┼───────────┼──────────────────────────┤
  │ WaitGameplayEffectRemoved         │ ✅        │ 等待 Effect 移除         │
  ├───────────────────────────────────┼───────────┼──────────────────────────┤
  │ WaitGameplayTag                   │ ✅        │ 等待标签添加/移除        │
  ├───────────────────────────────────┼───────────┼──────────────────────────┤
  │ WaitInputPress                    │ ✅        │ 等待输入按下             │
  ├───────────────────────────────────┼───────────┼──────────────────────────┤
  │ WaitInputRelease                  │ ✅        │ 等待输入释放             │
  ├───────────────────────────────────┼───────────┼──────────────────────────┤
  │ WaitConfirm                       │ ✅        │ 等待确认                 │
  ├───────────────────────────────────┼───────────┼──────────────────────────┤
  │ WaitCancel                        │ ✅        │ 等待取消                 │
  ├───────────────────────────────────┼───────────┼──────────────────────────┤
  │ WaitAbilityActivate               │ ✅        │ 等待其他 Ability 激活    │
  ├───────────────────────────────────┼───────────┼──────────────────────────┤
  │ WaitAbilityCommit                 │ ✅        │ 等待 Ability 提交        │
  ├───────────────────────────────────┼───────────┼──────────────────────────┤
  │ WaitTargetData                    │ ❌        │ 等待目标数据（重要）     │
  ├───────────────────────────────────┼───────────┼──────────────────────────┤
  │ PlayMontageAndWait                │ ❌        │ 播放动画并等待（重要）   │
  ├───────────────────────────────────┼───────────┼──────────────────────────┤
  │ PlayAnimAndWait                   │ ❌        │ 播放动画并等待           │
  ├───────────────────────────────────┼───────────┼──────────────────────────┤
  │ ApplyRootMotion*                  │ ❌        │ 根运动相关（7 个 Task）  │
  ├───────────────────────────────────┼───────────┼──────────────────────────┤
  │ MoveToLocation                    │ ❌        │ 移动到位置               │
  ├───────────────────────────────────┼───────────┼──────────────────────────┤
  │ NetworkSyncPoint                  │ ❌        │ 网络同步点（单机不需要） │
  ├───────────────────────────────────┼───────────┼──────────────────────────┤
  │ Repeat                            │ ❌        │ 重复执行                 │
  ├───────────────────────────────────┼───────────┼──────────────────────────┤
  │ SpawnActor                        │ ❌        │ 生成 Actor               │
  ├───────────────────────────────────┼───────────┼──────────────────────────┤
  │ StartAbilityState                 │ ❌        │ 启动 Ability 状态        │
  ├───────────────────────────────────┼───────────┼──────────────────────────┤
  │ VisualizeTargeting                │ ❌        │ 可视化目标               │
  ├───────────────────────────────────┼───────────┼──────────────────────────┤
  │ WaitAttributeChangeRatioThreshold │ ❌        │ 等待属性比例阈值         │
  ├───────────────────────────────────┼───────────┼──────────────────────────┤
  │ WaitAttributeChangeThreshold      │ ❌        │ 等待属性阈值             │
  ├───────────────────────────────────┼───────────┼──────────────────────────┤
  │ WaitConfirmCancel                 │ ❌        │ 等待确认或取消           │
  ├───────────────────────────────────┼───────────┼──────────────────────────┤
  │ WaitGameplayEffectBlockedImmunity │ ❌        │ 等待免疫阻止             │
  ├───────────────────────────────────┼───────────┼──────────────────────────┤
  │ WaitGameplayEffectStackChange     │ ❌        │ 等待 Effect 堆叠变化     │
  ├───────────────────────────────────┼───────────┼──────────────────────────┤
  │ WaitGameplayTagCountChanged       │ ❌        │ 等待标签计数变化         │
  ├───────────────────────────────────┼───────────┼──────────────────────────┤
  │ WaitGameplayTagQuery              │ ❌        │ 等待标签查询             │
  ├───────────────────────────────────┼───────────┼──────────────────────────┤
  │ WaitMovementModeChange            │ ❌        │ 等待移动模式变化         │
  ├───────────────────────────────────┼───────────┼──────────────────────────┤
  │ WaitOverlap                       │ ❌        │ 等待重叠                 │
  ├───────────────────────────────────┼───────────┼──────────────────────────┤
  │ WaitVelocityChange                │ ❌        │ 等待速度变化             │
  └───────────────────────────────────┴───────────┴──────────────────────────┘

  关键缺失:
  - ❌ WaitTargetData: 等待玩家选择目标（技能指示器、瞄准）
  - ❌ PlayMontageAndWait: 播放动画并等待完成（技能动画）
  - ❌ SpawnActor: 生成实体（召唤物、投射物）

  ---
  3. Attribute 预计算/缓存系统

  UE GAS:
  // FAggregator 缓存计算结果
  struct FAggregator {
      float CachedValue;
      bool bIsDirty;

      float Evaluate() {
          if (bIsDirty) {
              CachedValue = RecalculateValue();
              bIsDirty = false;
          }
          return CachedValue;
      }
  };

  Bevy GAS 现状:
  - ❌ 每次查询都重新计算（aggregate_modifiers_system）
  - ⚠️ 性能问题：大量 Attribute 查询时会重复计算

  解决方案:
  // 添加缓存组件
  #[derive(Component)]
  struct AttributeCache {
      cached_value: f32,
      is_dirty: bool,
  }

  ---
  4. Prediction System（网络预测）

  UE GAS:
  // 客户端预测 + 服务器校验
  struct FPredictionKey {
      int16 Current;
      int16 Base;
  };

  // 预测流程
  Client: TryActivateAbility() → 预测激活 → 等待服务器确认
  Server: 校验 → 确认/拒绝 → 回滚客户端状态

  Bevy GAS 现状:
  - ❌ 完全未实现（单机游戏不需要）
  - ⚠️ 如果未来需要联网，需要重构整个架构

  ---
  5. GameplayCue 高级功能

  UE GAS 有，Bevy GAS 缺失:

  ┌────────────────────────────┬────────┬──────────┐
  │            功能            │ UE GAS │ Bevy GAS │
  ├────────────────────────────┼────────┼──────────┤
  │ Burst Cue（爆发式）        │ ✅     │ ✅       │
  ├────────────────────────────┼────────┼──────────┤
  │ Looping Cue（循环式）      │ ✅     │ ✅       │
  ├────────────────────────────┼────────┼──────────┤
  │ Latent Cue（延迟式）       │ ✅     │ ❌       │
  ├────────────────────────────┼────────┼──────────┤
  │ HitImpact Cue（命中反馈）  │ ✅     │ ❌       │
  ├────────────────────────────┼────────┼──────────┤
  │ Cue Translator（标签转换） │ ✅     │ ❌       │
  ├────────────────────────────┼────────┼──────────┤
  │ Cue Set（批量管理）        │ ✅     │ ❌       │
  └────────────────────────────┴────────┴──────────┘

  缺失功能:
  - ❌ Latent Cue: 延迟触发（如"3 秒后爆炸"）
  - ❌ HitImpact Cue: 命中反馈（物理参数：法线、命中点）
  - ❌ Cue Translator: 标签转换（如 Damage.Fire → GameplayCue.Fire.Impact）

  ---
  6. AttributeSet 高级功能

  UE GAS:
  class UAttributeSet {
      // 属性变化前回调
      virtual void PreAttributeChange(const FGameplayAttribute& Attribute, float& NewValue);

      // 属性变化后回调
      virtual void PostGameplayEffectExecute(const FGameplayEffectModCallbackData& Data);

      // 属性基础值变化回调
      virtual void PreAttributeBaseChange(const FGameplayAttribute& Attribute, float& NewValue);

      // 属性聚合器初始化
      virtual void InitFromMetaDataTable(const UDataTable* DataTable);
  };

  Bevy GAS 现状:
  - ✅ 有 AttributeHooks trait（pre_modify, post_modify）
  - ❌ 缺少 PreAttributeBaseChange（基础值变化前回调）
  - ❌ 缺少 InitFromMetaDataTable（从数据表初始化）

  ---
  7. GameplayEffect 高级计算

  UE GAS 有，Bevy GAS 部分实现:

  ┌────────────────────┬────────┬──────────┐
  │        功能        │ UE GAS │ Bevy GAS │
  ├────────────────────┼────────┼──────────┤
  │ ScalableFloat      │ ✅     │ ✅       │
  ├────────────────────┼────────┼──────────┤
  │ AttributeBased     │ ✅     │ ✅       │
  ├────────────────────┼────────┼──────────┤
  │ CustomCalculation  │ ✅     │ ✅       │
  ├────────────────────┼────────┼──────────┤
  │ SetByCaller        │ ✅     │ ❌       │
  ├────────────────────┼────────┼──────────┤
  │ CurveTable         │ ✅     │ ❌       │
  ├────────────────────┼────────┼──────────┤
  │ ConditionalEffects │ ✅     │ ❌       │
  └────────────────────┴────────┴──────────┘

  缺失功能:
  - ❌ SetByCaller: 运行时动态设置 Magnitude（如"伤害 = 攻击力 * 技能等级"）
  - ❌ CurveTable: 曲线表查找（如"等级 1-100 的伤害曲线"）
  - ❌ ConditionalEffects: 条件触发 Effect（如"生命值 < 50% 时触发额外效果"）

  ---
  8. Ability 高级功能

  UE GAS 有，Bevy GAS 缺失:

  ┌──────────────────────┬────────┬────────────┐
  │         功能         │ UE GAS │  Bevy GAS  │
  ├──────────────────────┼────────┼────────────┤
  │ Instancing Policy    │ ✅     │ ✅         │
  ├──────────────────────┼────────┼────────────┤
  │ Replication Policy   │ ✅     │ ❌（单机） │
  ├──────────────────────┼────────┼────────────┤
  │ Net Execution Policy │ ✅     │ ❌（单机） │
  ├──────────────────────┼────────┼────────────┤
  │ Ability Level        │ ✅     │ ❌         │
  ├──────────────────────┼────────┼────────────┤
  │ Input Binding        │ ✅     │ ❌         │
  ├──────────────────────┼────────┼────────────┤
  │ Ability Set          │ ✅     │ ❌         │
  ├──────────────────────┼────────┼────────────┤
  │ Ability State        │ ✅     │ ❌         │
  └──────────────────────┴────────┴────────────┘

  缺失功能:
  - ❌ Ability Level: 技能等级系统（影响伤害、冷却等）
  - ❌ Input Binding: 输入绑定（如"按 Q 键激活技能"）
  - ❌ Ability Set: 批量授予技能（如"职业技能包"）
  - ❌ Ability State: 技能状态机（如"蓄力 → 释放 → 冷却"）

  ---
  🟡 部分实现的功能

  9. GameplayEffect Execution Calculation

  现状:
  - ✅ 有 GameplayEffectExecutionCalculation trait
  - ⚠️ 缺少示例和文档
  - ⚠️ 缺少 AttributeCaptureDefinition（属性捕获定义）

  ---
  10. Custom Application Requirements

  现状:
  - ✅ 有 CustomApplicationRequirement trait
  - ✅ 有 4 种内置实现（百分比、等级范围、标签、源 vs 目标）
  - ⚠️ 缺少更多示例（如"暴击率"、"格挡率"）

  ---
  📊 功能覆盖率统计

  ┌─────────────────┬───────────────┬─────────────────┬────────┐
  │      模块       │ UE GAS 功能数 │ Bevy GAS 实现数 │ 覆盖率 │
  ├─────────────────┼───────────────┼─────────────────┼────────┤
  │ Attributes      │ 10            │ 8               │ 80%    │
  ├─────────────────┼───────────────┼─────────────────┼────────┤
  │ GameplayEffects │ 25            │ 18              │ 72%    │
  ├─────────────────┼───────────────┼─────────────────┼────────┤
  │ Abilities       │ 20            │ 14              │ 70%    │
  ├─────────────────┼───────────────┼─────────────────┼────────┤
  │ GameplayCues    │ 10            │ 6               │ 60%    │
  ├─────────────────┼───────────────┼─────────────────┼────────┤
  │ Ability Tasks   │ 42            │ 12              │ 29%    │
  ├─────────────────┼───────────────┼─────────────────┼────────┤
  │ 总计            │ 107           │ 58              │ 54%    │
  └─────────────────┴───────────────┴─────────────────┴────────┘

  ---
  🎯 建议的实现优先级

  P0 (Critical - 必须实现):

  1. ✅ 核心 Attribute/Effect/Ability/Cue 系统（已完成）
  2. ❌ WaitTargetData Task（技能指示器）
  3. ❌ PlayMontageAndWait Task（技能动画）
  4. ❌ SetByCaller Magnitude（动态伤害计算）

  P1 (High - 强烈建议):

  5. ❌ Ability Level 系统（技能等级）
  6. ❌ Input Binding（输入绑定）
  7. ❌ Attribute Cache（性能优化）
  8. ❌ ChanceToApplyGameplayEffectComponent（概率应用）
  9. ❌ ImmunityGameplayEffectComponent（免疫系统）

  P2 (Medium - 可选):

  10. ❌ CurveTable Magnitude（曲线表）
  11. ❌ ConditionalEffects（条件触发）
  12. ❌ Ability State Machine（状态机）
  13. ❌ GameplayCue Translator（标签转换）
  14. ❌ 更多 Ability Tasks（SpawnActor, Repeat, WaitOverlap 等）

  P3 (Low - 未来扩展):

  15. ❌ Prediction System（网络预测，如果需要联网）
  16. ❌ Replication（网络同步）
  17. ❌ Blueprint 可视化编辑器（Bevy 不支持）

  ---
  总结

  当前 Bevy GAS 实现了 UE GAS 约 54% 的功能，核心系统（Attributes、Effects、Abilities、Cues）已完成，但缺少：

  1. 模块化 GameplayEffect Components（UE 5.3+ 新特性）
  2. 30 个 Ability Tasks（特别是 WaitTargetData 和 PlayMontageAndWait）
  3. Ability Level 和 Input Binding
  4. SetByCaller 和 CurveTable（动态计算）
  5. 免疫系统和概率应用

  如果你的目标是单机 RPG 游戏，当前实现已经足够。如果需要复杂的技能系统（如 MOBA、MMORPG），建议优先实现 P0 和 P1 的功能。
