use bevy_gameplay_tag::gameplay_tag_requirements::GameplayTagRequirements;

use crate::gameplay_effect::components::{
    ActiveEffectsContainer, ActiveGameplayEffect, GameplayEffectSpec,
};

#[derive(Clone, Debug, Default)]
pub struct GameplayEffect {
    pub stacking_type: GameplayEffectStackingType,
    pub stack_limit_count: i32,
    pub stack_duration_refresh_policy: GameplayEffectStackingDurationPolicy,
    pub stack_period_reset_policy: EGameplayEffectStackingPeriodPolicy,
    pub stack_expiration_policy: GameplayEffectStackingExpirationPolicy,
    pub overflow_effects: Vec<GameplayEffect>,
    /** 如果为true，当堆叠计数达到上限时，堆叠尝试将失败，导致持续时间和上下文不会被刷新 */
    pub deny_overflow_application: bool,
    /** 如果为true，当效果溢出时，整个堆叠将被清除 */
    pub clear_stack_on_overflow: bool,
    /** 如果为true，GameplayCues 将仅在堆叠 GameplayEffect 的第一个实例中触发 */
    pub suppress_stacking_cues: bool,

    /** 如果为true，效果在应用时执行一次，然后在每个周期间隔执行。如果为false，则在第一个周期过去之前不会执行 */
    pub execute_periodic_effect_on_application: bool,

    /** These Gameplay Effect Components define how this Gameplay Effect behaves when applied */
    pub effect_behaviors: Vec<Box<dyn GameplayEffectBehavior>>,
}

pub trait GameplayEffectBehavior: Clone + Send + Sync + 'static {
    fn can_gameplay_effect_apply(
        &self,
        active_effect_container: &ActiveEffectsContainer,
        effect_spec: GameplayEffectSpec,
    ) -> bool {
        true
    }

    fn on_active_gameplay_effect_added(
        &self,
        active_effect_container: &ActiveEffectsContainer,
        active_effect: &ActiveGameplayEffect,
    ) -> bool {
        true
    }

    fn on_gameplay_effect_executed(
        &self,
        active_effect_container: &ActiveEffectsContainer,
        effect_spec: GameplayEffectSpec,
    ) {
    }

    fn on_gameplay_effect_applied(
        &self,
        active_effect_container: &ActiveEffectsContainer,
        effect_spec: GameplayEffectSpec,
    ) {
    }

    fn on_gameplay_effect_changed(&self) {}
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DurationPolicy {
    Instant,
    Infinite,
    HasDuration(f32),
}

// =============================================================================
// Stack堆叠相关
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum GameplayEffectStackingType {
    #[default]
    None,
    AggregateBySource,
    AggregateByTarget,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum GameplayEffectStackingDurationPolicy {
    #[default]
    /** 效果的持续时间将在任何成功的堆叠应用时刷新 */
    RefreshOnSuccessfulApplication,
    /** 效果的持续时间永远不会刷新 */
    NeverRefresh,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum EGameplayEffectStackingPeriodPolicy {
    #[default]
    /** 任何成功的堆叠应用都会丢弃周期性效果的下一次触发的进度 */
    ResetOnSuccessfulApplication,
    /** 无论堆叠应用如何，周期性效果的下一次触发的进度永远不会重置 */
    NeverReset,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum GameplayEffectStackingExpirationPolicy {
    #[default]
    /** 任何成功的堆叠应用都会丢弃周期性效果的下一次触发的进度 */
    ResetOnSuccessfulApplication,
    /** 无论堆叠应用如何，周期性效果的下一次触发的进度永远不会重置 */
    NeverReset,
}

// =============================================================================
// Modifier相关
// =============================================================================、

// 属性修饰器
#[derive(Debug)]
pub struct GameplayModifierInfo {
    pub attribute_name: String,
    pub modifier_op: GameplayModOp,
    pub source_tags: GameplayTagRequirements,
    pub target_tags: GameplayTagRequirements,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameplayModOp {
    AddBase,
    MultiplyAdditive,
    DivideAdditive,
    Override,
    MultiplyCompound,
    AddFinal,
    Max,
}

// =============================================================================
// Modifier计算相关
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameplayModEvaluationChannel {
    Channel0,
    Channel1,
    Channel2,
    Channel3,
    Channel4,
    Channel5,
    Channel6,
    Channel7,
    Channel8,
    Channel9,
    ChannelMax,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameplayEffectMagnitudeCalculation {
    /** 使用一个简单的、可扩展的浮点数进行计算 */
    ScalableFloat,
    /** 根据属性执行计算 */
    AttributeBased,
    /** 执行自定义计算，能够在BP或原生中捕获并处理多个属性 */
    CustomCalculationClass,
    /** 这个修改数值幅度将由创建规范的调用者明确设置 */
    SetByCaller,
}
