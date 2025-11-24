use bevy_gameplay_tag::gameplay_tag_requirements::GameplayTagRequirements;

#[derive(Clone, Debug, Default)]
#[expect(dead_code)]
pub struct GameplayEffect {}

// 属性修饰器
#[derive(Debug)]
pub struct GameplayModifierInfo {
    pub attribute_name: String,
    pub modifier_op: ModifierOp,
    pub source_tags: GameplayTagRequirements,
    pub target_tags: GameplayTagRequirements,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DurationPolicy {
    Instant,
    Infinite,
    HasDuration(f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModifierOp {
    AddBase,
    MultiplyAdditive,
    DivideAdditive,
    Override,
    MultiplyCompound,
    AddFinal,
    Max,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameplayEffectStackingType {
    None,
    AggregateBySource,
    AggregateByTarget,
}

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
