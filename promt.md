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
