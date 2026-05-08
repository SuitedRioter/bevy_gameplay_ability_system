//! Gameplay effect definitions.
//!
//! This module defines the structure of gameplay effects and their properties.

use super::components::{EvaluationChannel, ModifierOperation};
use super::execution::GameplayEffectExecutionCalculation;
use crate::cues::manager::GameplayCueParameters;
use bevy::prelude::*;
use bevy_gameplay_tag::{
    GameplayTagContainer, GameplayTagRequirements, GameplayTagsManager, gameplay_tag::GameplayTag,
};
use std::sync::Arc;
use string_cache::DefaultAtom as Atom;

/// Policy for handling granted abilities when the effect is removed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AbilityRemovalPolicy {
    /// Cancel the ability immediately when the effect is removed.
    CancelAbilityImmediately,
    /// Remove the ability spec but let active instances finish.
    RemoveAbilityOnEnd,
    /// Do nothing - the ability remains granted permanently.
    DoNothing,
}

impl Default for AbilityRemovalPolicy {
    fn default() -> Self {
        Self::CancelAbilityImmediately
    }
}

/// Describes an ability to be granted by an effect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrantedAbilityConfig {
    /// The ability definition ID to grant.
    pub ability_id: Atom,
    /// How to handle the ability when the effect is removed.
    pub removal_policy: AbilityRemovalPolicy,
}

impl GrantedAbilityConfig {
    pub fn new(ability_id: impl Into<Atom>) -> Self {
        Self {
            ability_id: ability_id.into(),
            removal_policy: AbilityRemovalPolicy::default(),
        }
    }

    pub fn with_removal_policy(mut self, policy: AbilityRemovalPolicy) -> Self {
        self.removal_policy = policy;
        self
    }
}

/// Conditional gameplay effect that applies only if source tags match.
///
/// Matches UE GAS's `FConditionalGameplayEffect`.
#[derive(Debug, Clone, PartialEq)]
pub struct ConditionalGameplayEffect {
    /// The effect definition ID to apply.
    pub effect_id: Atom,
    /// Tags that the source must have for this effect to apply.
    /// If empty, the effect always applies.
    pub required_source_tags: GameplayTagContainer,
}

impl ConditionalGameplayEffect {
    /// Creates a new conditional effect.
    pub fn new(effect_id: impl Into<Atom>) -> Self {
        Self {
            effect_id: effect_id.into(),
            required_source_tags: GameplayTagContainer::default(),
        }
    }

    /// Adds a required source tag.
    pub fn require_source_tag(
        mut self,
        tag: GameplayTag,
        tags_manager: &GameplayTagsManager,
    ) -> Self {
        self.required_source_tags.add_tag(tag, tags_manager);
        self
    }

    /// Checks if this conditional effect can apply given the source tags.
    ///
    /// Returns true if the source has all required tags, or if no tags are required.
    pub fn can_apply(&self, source_tags: &GameplayTagContainer) -> bool {
        if self.required_source_tags.is_empty() {
            return true;
        }
        source_tags.has_all(&self.required_source_tags)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DurationPolicy {
    /// Effect applies instantly and is removed immediately.
    Instant,
    /// Effect has a limited duration.
    HasDuration,
    /// Effect lasts forever until explicitly removed.
    Infinite,
}

/// Stacking type for gameplay effects.
///
/// Defines how multiple applications of the same effect aggregate.
/// Matches UE GAS's `EGameplayEffectStackingType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StackingType {
    /// No stacking. Each application is a separate instance.
    None,
    /// Stacks only when the same source reapplies the effect.
    AggregateBySource,
    /// Stacks only when the effect is reapplied to the same target.
    AggregateByTarget,
}

impl Default for StackingType {
    fn default() -> Self {
        Self::None
    }
}

/// Policy for refreshing effect duration while stacking.
///
/// Matches UE GAS's `EGameplayEffectStackingDurationPolicy`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StackingDurationPolicy {
    /// The duration is refreshed from any successful stack application.
    RefreshOnSuccessfulApplication,
    /// The duration is never refreshed.
    NeverRefresh,
    /// New stacks add their duration onto current remaining time.
    ExtendDuration,
}

impl Default for StackingDurationPolicy {
    fn default() -> Self {
        Self::RefreshOnSuccessfulApplication
    }
}

/// Policy for resetting effect period while stacking.
///
/// Matches UE GAS's `EGameplayEffectStackingPeriodPolicy`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StackingPeriodPolicy {
    /// Progress toward next tick is discarded upon successful stack application.
    ResetOnSuccessfulApplication,
    /// Progress toward next tick is never reset.
    NeverReset,
}

impl Default for StackingPeriodPolicy {
    fn default() -> Self {
        Self::NeverReset
    }
}

/// Policy for handling stack expiration in duration-based effects.
///
/// Matches UE GAS's `EGameplayEffectStackingExpirationPolicy`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StackingExpirationPolicy {
    /// The entire stack is cleared when the effect expires.
    ClearEntireStack,
    /// Decrement stack count by 1 and refresh duration.
    RemoveSingleStackAndRefreshDuration,
    /// Refresh duration without decrementing (makes effect infinite).
    RefreshDuration,
}

impl Default for StackingExpirationPolicy {
    fn default() -> Self {
        Self::ClearEntireStack
    }
}

/// Complete stacking configuration for gameplay effects.
///
/// This replaces the old `StackingPolicy` enum with a comprehensive
/// three-layer model matching UE GAS.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StackingConfig {
    /// How this effect stacks with other instances.
    pub stacking_type: StackingType,
    /// Stack limit. 0 or negative means no limit.
    pub stack_limit_count: i32,
    /// Policy for refreshing duration while stacking.
    pub duration_refresh_policy: StackingDurationPolicy,
    /// Policy for resetting period while stacking.
    pub period_reset_policy: StackingPeriodPolicy,
    /// Policy for handling expiration.
    pub expiration_policy: StackingExpirationPolicy,
    /// If true, deny applications when at stack limit.
    pub deny_overflow_application: bool,
    /// If true, clear entire stack on overflow.
    pub clear_stack_on_overflow: bool,
    /// If true, include stack count in modifier magnitude calculations.
    pub factor_in_stack_count: bool,
}

impl Default for StackingConfig {
    fn default() -> Self {
        Self {
            stacking_type: StackingType::None,
            stack_limit_count: 0,
            duration_refresh_policy: StackingDurationPolicy::default(),
            period_reset_policy: StackingPeriodPolicy::default(),
            expiration_policy: StackingExpirationPolicy::default(),
            deny_overflow_application: false,
            clear_stack_on_overflow: false,
            factor_in_stack_count: false,
        }
    }
}

impl StackingConfig {
    /// Creates a config with no stacking (each application is independent).
    pub fn none() -> Self {
        Self::default()
    }

    /// Creates a config that stacks by source with default policies.
    pub fn aggregate_by_source(stack_limit: i32) -> Self {
        Self {
            stacking_type: StackingType::AggregateBySource,
            stack_limit_count: stack_limit,
            ..Default::default()
        }
    }

    /// Creates a config that stacks by target with default policies.
    pub fn aggregate_by_target(stack_limit: i32) -> Self {
        Self {
            stacking_type: StackingType::AggregateByTarget,
            stack_limit_count: stack_limit,
            ..Default::default()
        }
    }

    /// Sets the duration refresh policy.
    pub fn with_duration_policy(mut self, policy: StackingDurationPolicy) -> Self {
        self.duration_refresh_policy = policy;
        self
    }

    /// Sets the period reset policy.
    pub fn with_period_policy(mut self, policy: StackingPeriodPolicy) -> Self {
        self.period_reset_policy = policy;
        self
    }

    /// Sets the expiration policy.
    pub fn with_expiration_policy(mut self, policy: StackingExpirationPolicy) -> Self {
        self.expiration_policy = policy;
        self
    }

    /// Sets whether to deny overflow applications.
    pub fn deny_overflow(mut self, deny: bool) -> Self {
        self.deny_overflow_application = deny;
        self
    }

    /// Sets whether to clear stack on overflow.
    pub fn clear_on_overflow(mut self, clear: bool) -> Self {
        self.clear_stack_on_overflow = clear;
        self
    }

    /// Sets whether to factor stack count into magnitude calculations.
    pub fn factor_stack_count(mut self, factor: bool) -> Self {
        self.factor_in_stack_count = factor;
        self
    }

    /// Returns true if this config allows stacking.
    pub fn allows_stacking(&self) -> bool {
        self.stacking_type != StackingType::None
    }

    /// Returns true if the stack limit has been reached.
    pub fn is_at_limit(&self, current_count: i32) -> bool {
        if self.stack_limit_count <= 0 {
            return false; // No limit
        }
        current_count >= self.stack_limit_count
    }
}

/// Deprecated: Old stacking policy enum.
///
/// Use `StackingConfig` instead for full UE GAS compatibility.
#[deprecated(since = "0.2.0", note = "Use StackingConfig instead")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StackingPolicy {
    /// Each application is independent.
    Independent,
    /// Refresh the duration on reapplication.
    RefreshDuration,
    /// Increment stack count up to a maximum.
    StackCount { max_stacks: i32 },
}

/// Attribute calculation type.
///
/// Defines which value to use when capturing an attribute for magnitude calculation.
/// Matches UE GAS's `EAttributeBasedFloatCalculationType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AttributeCalculationType {
    /// Use the final evaluated magnitude (current_value).
    AttributeMagnitude,
    /// Use the base value only.
    AttributeBaseValue,
    /// Use the bonus magnitude: (current_value - base_value).
    AttributeBonusMagnitude,
}

/// Attribute capture source.
///
/// Defines whether to capture the attribute from the source (instigator) or target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AttributeCaptureSource {
    /// Capture from the source entity (instigator).
    Source,
    /// Capture from the target entity.
    Target,
}

/// Attribute capture mode.
///
/// Defines when the attribute value is captured for magnitude calculation.
/// Matches UE GAS's snapshot vs dynamic evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AttributeCaptureMode {
    /// Capture attribute value when the effect is created (snapshot).
    /// The captured value never changes, even if the source attribute changes.
    /// Useful for: damage based on caster's attack power at cast time,
    /// DOT effects that scale with spell power at application time.
    Snapshot,

    /// Re-evaluate attribute value each time the magnitude is calculated (dynamic).
    /// The value updates if the source attribute changes.
    /// Useful for: effects that should scale with current stats,
    /// buffs that update when the source's power changes.
    Dynamic,
}

impl Default for AttributeCaptureMode {
    fn default() -> Self {
        Self::Snapshot
    }
}

/// Magnitude calculation type.
///
/// Defines how the magnitude of a modifier is calculated.
/// Follows UE GAS's magnitude calculation system.
#[derive(Clone)]
pub enum MagnitudeCalculation {
    /// A fixed scalar value (optionally scaled by level).
    ///
    /// Formula: `base_value * level_multiplier^(level - 1)`
    ScalableFloat {
        base_value: f32,
        /// Multiplier applied per level (1.0 = no scaling).
        level_multiplier: f32,
    },

    /// Curve-based magnitude using Bevy's Curve system.
    ///
    /// The curve is sampled at the effect level to determine magnitude.
    /// This is equivalent to UE GAS's CurveTable lookup.
    ///
    /// # Example
    /// ```ignore
    /// use bevy::math::curve::{Curve, SampleCurve, Interval};
    ///
    /// // Damage curve: level 1 = 10, level 10 = 150, level 20 = 500
    /// let samples = vec![10.0, 30.0, 60.0, 100.0, 150.0, 220.0, 300.0, 400.0, 500.0];
    /// let curve = SampleCurve::new(interval(1.0, 20.0).unwrap(), samples).unwrap();
    ///
    /// MagnitudeCalculation::CurveBased {
    ///     curve: Arc::new(curve),
    /// }
    /// ```
    CurveBased {
        /// The curve to sample (level -> magnitude).
        /// Must implement `Curve<f32> + Send + Sync`.
        curve: Arc<dyn bevy::math::curve::Curve<f32> + Send + Sync>,
    },

    /// Calculate from an attribute on the source or target entity.
    ///
    /// Formula: `(coefficient * (pre_multiply_additive + [attribute_value])) + post_multiply_additive`
    ///
    /// This allows you to scale damage based on the caster's stats, for example.
    AttributeBased {
        /// Name of the attribute to read.
        attribute_name: Atom,
        /// Which entity to capture from (Source or Target).
        capture_source: AttributeCaptureSource,
        /// Which value to use from the attribute.
        calculation_type: AttributeCalculationType,
        /// When to capture the attribute value (Snapshot or Dynamic).
        capture_mode: AttributeCaptureMode,
        /// Coefficient to multiply the attribute value by.
        coefficient: f32,
        /// Value added before multiplication.
        pre_multiply_additive: f32,
        /// Value added after multiplication.
        post_multiply_additive: f32,
    },

    /// Custom calculation using a registered calculator.
    ///
    /// The calculator is looked up by name from a registry.
    /// This allows complex calculations that capture multiple attributes.
    CustomClass {
        /// Name of the custom calculator to use.
        calculator_name: Atom,
    },

    /// Custom execution calculation.
    ///
    /// Provides the most flexibility by allowing custom logic that can
    /// capture multiple attributes and produce multiple modifiers.
    /// Matches UE GAS's `UGameplayEffectExecutionCalculation`.
    ///
    /// # Example
    /// ```ignore
    /// struct DamageCalculation;
    /// impl GameplayEffectExecutionCalculation for DamageCalculation {
    ///     // ... implementation
    /// }
    ///
    /// MagnitudeCalculation::execution(Arc::new(DamageCalculation))
    /// ```
    CustomExecution {
        /// The execution calculation to use.
        calculation: Arc<dyn GameplayEffectExecutionCalculation>,
    },

    /// Magnitude set at runtime by the caller.
    ///
    /// The caller must provide a value for this tag when applying the effect.
    /// If not provided, defaults to 0.0.
    SetByCaller {
        /// Tag identifying this magnitude value.
        data_tag: GameplayTag,
    },
}

impl std::fmt::Debug for MagnitudeCalculation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ScalableFloat {
                base_value,
                level_multiplier,
            } => f
                .debug_struct("ScalableFloat")
                .field("base_value", base_value)
                .field("level_multiplier", level_multiplier)
                .finish(),
            Self::CurveBased { .. } => f
                .debug_struct("CurveBased")
                .field("curve", &"<dyn Curve<f32>>")
                .finish(),
            Self::AttributeBased {
                attribute_name,
                capture_source,
                calculation_type,
                capture_mode,
                coefficient,
                pre_multiply_additive,
                post_multiply_additive,
            } => f
                .debug_struct("AttributeBased")
                .field("attribute_name", attribute_name)
                .field("capture_source", capture_source)
                .field("calculation_type", calculation_type)
                .field("capture_mode", capture_mode)
                .field("coefficient", coefficient)
                .field("pre_multiply_additive", pre_multiply_additive)
                .field("post_multiply_additive", post_multiply_additive)
                .finish(),
            Self::CustomClass { calculator_name } => f
                .debug_struct("CustomClass")
                .field("calculator_name", calculator_name)
                .finish(),
            Self::CustomExecution { calculation } => f
                .debug_struct("CustomExecution")
                .field("calculation", calculation)
                .finish(),
            Self::SetByCaller { data_tag } => f
                .debug_struct("SetByCaller")
                .field("data_tag", data_tag)
                .finish(),
        }
    }
}

impl PartialEq for MagnitudeCalculation {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::ScalableFloat {
                    base_value: l_base,
                    level_multiplier: l_mult,
                },
                Self::ScalableFloat {
                    base_value: r_base,
                    level_multiplier: r_mult,
                },
            ) => l_base == r_base && l_mult == r_mult,
            (Self::CurveBased { .. }, Self::CurveBased { .. }) => {
                // Cannot compare trait objects, so always return false
                false
            }
            (
                Self::AttributeBased {
                    attribute_name: l_name,
                    capture_source: l_source,
                    calculation_type: l_calc,
                    capture_mode: l_mode,
                    coefficient: l_coef,
                    pre_multiply_additive: l_pre,
                    post_multiply_additive: l_post,
                },
                Self::AttributeBased {
                    attribute_name: r_name,
                    capture_source: r_source,
                    calculation_type: r_calc,
                    capture_mode: r_mode,
                    coefficient: r_coef,
                    pre_multiply_additive: r_pre,
                    post_multiply_additive: r_post,
                },
            ) => {
                l_name == r_name
                    && l_source == r_source
                    && l_calc == r_calc
                    && l_mode == r_mode
                    && l_coef == r_coef
                    && l_pre == r_pre
                    && l_post == r_post
            }
            (
                Self::CustomClass { calculator_name: l },
                Self::CustomClass { calculator_name: r },
            ) => l == r,
            (Self::CustomExecution { .. }, Self::CustomExecution { .. }) => {
                // Cannot compare trait objects, so always return false
                false
            }
            (Self::SetByCaller { data_tag: l }, Self::SetByCaller { data_tag: r }) => l == r,
            _ => false,
        }
    }
}

impl MagnitudeCalculation {
    /// Creates a simple scalar magnitude.
    pub fn scalar(value: f32) -> Self {
        Self::ScalableFloat {
            base_value: value,
            level_multiplier: 1.0,
        }
    }

    /// Creates a level-scaled magnitude.
    ///
    /// # Example
    /// ```ignore
    /// // Damage that scales: 10 at level 1, 20 at level 2, 40 at level 3
    /// MagnitudeCalculation::scaled(10.0, 2.0)
    /// ```
    pub fn scaled(base_value: f32, level_multiplier: f32) -> Self {
        Self::ScalableFloat {
            base_value,
            level_multiplier,
        }
    }

    /// Creates an attribute-based magnitude from the source entity.
    ///
    /// Uses the current value (AttributeMagnitude) and Snapshot mode by default.
    pub fn from_source_attribute(attribute_name: impl Into<Atom>, coefficient: f32) -> Self {
        Self::AttributeBased {
            attribute_name: attribute_name.into(),
            capture_source: AttributeCaptureSource::Source,
            calculation_type: AttributeCalculationType::AttributeMagnitude,
            capture_mode: AttributeCaptureMode::Snapshot,
            coefficient,
            pre_multiply_additive: 0.0,
            post_multiply_additive: 0.0,
        }
    }

    /// Creates an attribute-based magnitude from the target entity.
    ///
    /// Uses the current value (AttributeMagnitude) and Snapshot mode by default.
    pub fn from_target_attribute(attribute_name: impl Into<Atom>, coefficient: f32) -> Self {
        Self::AttributeBased {
            attribute_name: attribute_name.into(),
            capture_source: AttributeCaptureSource::Target,
            calculation_type: AttributeCalculationType::AttributeMagnitude,
            capture_mode: AttributeCaptureMode::Snapshot,
            coefficient,
            pre_multiply_additive: 0.0,
            post_multiply_additive: 0.0,
        }
    }

    /// Creates a custom execution calculation magnitude.
    ///
    /// This is the most flexible option, allowing complex calculations
    /// that capture multiple attributes and produce multiple modifiers.
    ///
    /// # Example
    /// ```ignore
    /// let damage_calc = Arc::new(DamageCalculation);
    /// MagnitudeCalculation::execution(damage_calc)
    /// ```
    pub fn execution(calculation: Arc<dyn GameplayEffectExecutionCalculation>) -> Self {
        Self::CustomExecution { calculation }
    }

    /// Builder method to set the calculation type for AttributeBased.
    pub fn with_calculation_type(mut self, calc_type: AttributeCalculationType) -> Self {
        if let Self::AttributeBased {
            calculation_type, ..
        } = &mut self
        {
            *calculation_type = calc_type;
        }
        self
    }

    /// Builder method to set the capture mode for AttributeBased.
    pub fn with_capture_mode(mut self, mode: AttributeCaptureMode) -> Self {
        if let Self::AttributeBased { capture_mode, .. } = &mut self {
            *capture_mode = mode;
        }
        self
    }

    /// Builder method to set pre-multiply additive for AttributeBased.
    pub fn with_pre_multiply_add(mut self, value: f32) -> Self {
        if let Self::AttributeBased {
            pre_multiply_additive,
            ..
        } = &mut self
        {
            *pre_multiply_additive = value;
        }
        self
    }

    /// Builder method to set post-multiply additive for AttributeBased.
    pub fn with_post_multiply_add(mut self, value: f32) -> Self {
        if let Self::AttributeBased {
            post_multiply_additive,
            ..
        } = &mut self
        {
            *post_multiply_additive = value;
        }
        self
    }

    /// Creates a curve-based magnitude using Bevy's Curve system.
    ///
    /// # Example
    /// ```ignore
    /// use bevy::math::curve::{SampleCurve, interval};
    ///
    /// // Damage curve: level 1-5 with values [10, 30, 60, 100, 150]
    /// let samples = vec![10.0, 30.0, 60.0, 100.0, 150.0];
    /// let curve = SampleCurve::new(interval(1.0, 5.0).unwrap(), samples).unwrap();
    ///
    /// MagnitudeCalculation::curve(Arc::new(curve))
    /// ```
    pub fn curve(curve: Arc<dyn bevy::math::curve::Curve<f32> + Send + Sync>) -> Self {
        Self::CurveBased { curve }
    }

    /// Creates a SetByCaller magnitude.
    pub fn set_by_caller(data_tag: GameplayTag) -> Self {
        Self::SetByCaller { data_tag }
    }

    /// Creates a custom calculation magnitude.
    pub fn custom(calculator_name: impl Into<Atom>) -> Self {
        Self::CustomClass {
            calculator_name: calculator_name.into(),
        }
    }

    /// Evaluates the magnitude given a level and optional source value.
    ///
    /// For AttributeBased calculations, pass the captured attribute value as `source_value`.
    /// For SetByCaller, pass the caller-provided value.
    /// For CurveBased, the level is used to sample the curve.
    pub fn evaluate(&self, level: i32, source_value: Option<f32>) -> f32 {
        match self {
            MagnitudeCalculation::ScalableFloat {
                base_value,
                level_multiplier,
            } => {
                if *level_multiplier == 1.0 {
                    *base_value
                } else {
                    base_value * level_multiplier.powi(level - 1)
                }
            }
            MagnitudeCalculation::CurveBased { curve } => {
                use bevy::math::curve::Curve;
                // Sample the curve at the effect level
                curve.sample_clamped(level as f32)
            }
            MagnitudeCalculation::AttributeBased {
                coefficient,
                pre_multiply_additive,
                post_multiply_additive,
                ..
            } => {
                let source = source_value.unwrap_or(0.0);
                (source + pre_multiply_additive) * coefficient + post_multiply_additive
            }
            MagnitudeCalculation::SetByCaller { .. } => {
                // Caller must provide the value
                source_value.unwrap_or(0.0)
            }
            MagnitudeCalculation::CustomClass { .. } => {
                // Custom calculators are looked up from a registry
                // For now, return 0.0 as placeholder
                warn!("Custom calculation not yet implemented");
                0.0
            }
            MagnitudeCalculation::CustomExecution { .. } => {
                // CustomExecution produces multiple modifiers, not a single magnitude
                // This should not be called for execution calculations
                warn!("CustomExecution should not be evaluated as a simple magnitude");
                0.0
            }
        }
    }
}

/// Information about a modifier in an effect.
#[derive(Debug, Clone, PartialEq)]
pub struct ModifierInfo {
    /// The name of the attribute to modify.
    pub attribute_name: Atom,
    /// The operation to perform.
    pub operation: ModifierOperation,
    /// How to calculate the magnitude.
    pub magnitude: MagnitudeCalculation,
    /// The evaluation channel for this modifier.
    pub channel: EvaluationChannel,
}

impl ModifierInfo {
    /// Creates a new modifier info with default channel (Channel0).
    pub fn new(
        attribute_name: impl Into<Atom>,
        operation: ModifierOperation,
        magnitude: MagnitudeCalculation,
    ) -> Self {
        Self {
            attribute_name: attribute_name.into(),
            operation,
            magnitude,
            channel: EvaluationChannel::default(),
        }
    }

    /// Sets the evaluation channel for this modifier.
    pub fn with_channel(mut self, channel: EvaluationChannel) -> Self {
        self.channel = channel;
        self
    }
}

/// GameplayCue configuration attached to an effect definition.
#[derive(Debug, Clone, PartialEq)]
pub struct GameplayEffectCue {
    /// Tag routed through the cue system.
    pub cue_tag: GameplayTag,
    /// Minimum effect level for this cue to fire.
    pub min_level: i32,
    /// Maximum effect level for this cue to fire.
    pub max_level: Option<i32>,
    /// Override parameters merged onto those derived from the effect spec/context.
    pub parameters: GameplayCueParameters,
}

impl GameplayEffectCue {
    pub fn new(cue_tag: GameplayTag) -> Self {
        Self {
            cue_tag,
            min_level: 0,
            max_level: None,
            parameters: GameplayCueParameters::new(),
        }
    }

    pub fn with_level_range(mut self, min_level: i32, max_level: Option<i32>) -> Self {
        self.min_level = min_level;
        self.max_level = max_level;
        self
    }

    pub fn with_parameters(mut self, parameters: GameplayCueParameters) -> Self {
        self.parameters = parameters;
        self
    }

    pub fn applies_to_level(&self, level: i32) -> bool {
        if level < self.min_level {
            return false;
        }
        if let Some(max_level) = self.max_level {
            level <= max_level
        } else {
            true
        }
    }
}

/// Definition of a gameplay effect.
///
/// This is the template for creating active effect instances.
/// Store these in a resource or asset system.
#[derive(Clone)]
pub struct GameplayEffectDefinition {
    /// Unique identifier for this effect.
    pub id: Atom,
    /// Duration policy.
    pub duration_policy: DurationPolicy,
    /// Duration in seconds (if HasDuration).
    pub duration_magnitude: f32,
    /// Period for periodic effects (0.0 = not periodic).
    pub period: f32,
    /// Modifiers applied by this effect.
    pub modifiers: Vec<ModifierInfo>,
    /// Tags granted while this effect is active.
    pub granted_tags: GameplayTagContainer,
    /// Tags that identify this effect (for immunity checks).
    /// If a target has any of these tags in their immunity_tags, the effect is rejected.
    pub asset_tags: GameplayTagContainer,
    /// Tags that grant immunity to effects.
    /// If this effect has any of these tags, targets with matching immunity_tags will reject it.
    pub immunity_tags: GameplayTagContainer,
    /// Tag requirements for applying this effect.
    pub application_tag_requirements: GameplayTagRequirements,
    /// Custom application requirements that must all pass before the effect applies.
    pub application_requirements: Vec<Atom>,
    /// Stacking configuration.
    pub stacking_config: StackingConfig,
    /// Abilities granted while this effect is active.
    pub granted_abilities: Vec<GrantedAbilityConfig>,
    /// Gameplay cues triggered by this effect.
    pub gameplay_cues: Vec<GameplayEffectCue>,
    /// Conditional effects that apply if source tags match.
    ///
    /// These effects are applied to the target when this effect successfully applies.
    /// Each conditional effect checks the source's tags before applying.
    pub conditional_effects: Vec<ConditionalGameplayEffect>,
    /// Modular components that extend effect behavior (UE 5.3+ feature).
    ///
    /// Components are executed at specific lifecycle points:
    /// - `can_apply`: Before application (can block)
    /// - `on_effect_applied`: After successful application
    /// - `on_effect_removed`: When removed from target
    pub components: Vec<crate::effects::ge_component::BoxedGameplayEffectComponent>,
}

impl std::fmt::Debug for GameplayEffectDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GameplayEffectDefinition")
            .field("id", &self.id)
            .field("duration_policy", &self.duration_policy)
            .field("duration_magnitude", &self.duration_magnitude)
            .field("period", &self.period)
            .field("modifiers", &self.modifiers)
            .field("granted_tags", &self.granted_tags)
            .field("asset_tags", &self.asset_tags)
            .field("immunity_tags", &self.immunity_tags)
            .field(
                "application_tag_requirements",
                &self.application_tag_requirements,
            )
            .field("application_requirements", &self.application_requirements)
            .field("stacking_config", &self.stacking_config)
            .field("granted_abilities", &self.granted_abilities)
            .field("gameplay_cues", &self.gameplay_cues)
            .field("conditional_effects", &self.conditional_effects)
            .field(
                "components",
                &format!("{} components", self.components.len()),
            )
            .finish()
    }
}

impl PartialEq for GameplayEffectDefinition {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.duration_policy == other.duration_policy
            && self.duration_magnitude == other.duration_magnitude
            && self.period == other.period
            && self.modifiers == other.modifiers
            && self.granted_tags == other.granted_tags
            && self.asset_tags == other.asset_tags
            && self.immunity_tags == other.immunity_tags
            && self.application_tag_requirements == other.application_tag_requirements
            && self.application_requirements == other.application_requirements
            && self.stacking_config == other.stacking_config
            && self.granted_abilities == other.granted_abilities
            && self.gameplay_cues == other.gameplay_cues
            && self.conditional_effects == other.conditional_effects
            && self.components.len() == other.components.len()
    }
}

impl GameplayEffectDefinition {
    /// Creates a new gameplay effect definition.
    pub fn new(id: impl Into<Atom>) -> Self {
        Self {
            id: id.into(),
            duration_policy: DurationPolicy::Instant,
            duration_magnitude: 0.0,
            period: 0.0,
            modifiers: Vec::new(),
            granted_tags: GameplayTagContainer::default(),
            asset_tags: GameplayTagContainer::default(),
            immunity_tags: GameplayTagContainer::default(),
            application_tag_requirements: GameplayTagRequirements::default(),
            application_requirements: Vec::new(),
            stacking_config: StackingConfig::default(),
            granted_abilities: Vec::new(),
            gameplay_cues: Vec::new(),
            conditional_effects: Vec::new(),
            components: Vec::new(),
        }
    }

    /// Sets the duration policy.
    pub fn with_duration_policy(mut self, policy: DurationPolicy) -> Self {
        self.duration_policy = policy;
        self
    }

    /// Sets the duration magnitude.
    pub fn with_duration(mut self, duration: f32) -> Self {
        self.duration_magnitude = duration;
        self.duration_policy = DurationPolicy::HasDuration;
        self
    }

    /// Sets the period for periodic effects.
    pub fn with_period(mut self, period: f32) -> Self {
        self.period = period;
        self
    }

    /// Adds a modifier to this effect.
    pub fn add_modifier(mut self, modifier: ModifierInfo) -> Self {
        self.modifiers.push(modifier);
        self
    }

    /// Adds a granted tag.
    pub fn grant_tag(mut self, tag: GameplayTag, tags_manager: &Res<GameplayTagsManager>) -> Self {
        self.granted_tags.add_tag(tag, tags_manager);
        self
    }

    /// Adds an asset tag (for immunity checks).
    pub fn with_asset_tag(
        mut self,
        tag: GameplayTag,
        tags_manager: &Res<GameplayTagsManager>,
    ) -> Self {
        self.asset_tags.add_tag(tag, tags_manager);
        self
    }

    /// Adds an immunity tag.
    ///
    /// Effects with these tags can be blocked by targets that have matching immunity.
    pub fn with_immunity_tag(
        mut self,
        tag: GameplayTag,
        tags_manager: &Res<GameplayTagsManager>,
    ) -> Self {
        self.immunity_tags.add_tag(tag, tags_manager);
        self
    }

    /// Sets the tag requirements.
    pub fn with_tag_requirements(mut self, requirements: GameplayTagRequirements) -> Self {
        self.application_tag_requirements = requirements;
        self
    }

    /// Adds a custom application requirement.
    pub fn add_application_requirement(mut self, requirement_name: impl Into<Atom>) -> Self {
        self.application_requirements.push(requirement_name.into());
        self
    }

    /// Sets the stacking configuration.
    pub fn with_stacking_config(mut self, config: StackingConfig) -> Self {
        self.stacking_config = config;
        self
    }

    /// Deprecated: Sets the stacking policy using old enum.
    #[deprecated(since = "0.2.0", note = "Use with_stacking_config instead")]
    #[allow(deprecated)]
    pub fn with_stacking_policy(mut self, policy: StackingPolicy) -> Self {
        self.stacking_config = match policy {
            StackingPolicy::Independent => StackingConfig::none(),
            StackingPolicy::RefreshDuration => StackingConfig::aggregate_by_target(0)
                .with_duration_policy(StackingDurationPolicy::RefreshOnSuccessfulApplication),
            StackingPolicy::StackCount { max_stacks } => {
                StackingConfig::aggregate_by_target(max_stacks)
            }
        };
        self
    }

    /// Adds a gameplay cue triggered by this effect.
    pub fn add_gameplay_cue(mut self, cue: GameplayEffectCue) -> Self {
        self.gameplay_cues.push(cue);
        self
    }

    /// Grants an ability while this effect is active.
    ///
    /// The ability will be granted when the effect is applied and removed when the effect ends.
    /// This is useful for temporary abilities (e.g., equipment abilities, buff abilities).
    pub fn grant_ability(mut self, config: GrantedAbilityConfig) -> Self {
        self.granted_abilities.push(config);
        self
    }

    /// Convenience method to grant an ability with default removal policy.
    pub fn grant_ability_simple(mut self, ability_id: impl Into<Atom>) -> Self {
        self.granted_abilities
            .push(GrantedAbilityConfig::new(ability_id));
        self
    }

    /// Adds a modular component to this effect.
    ///
    /// Components extend effect behavior at specific lifecycle points.
    /// See `GameplayEffectComponent` trait for details.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use std::sync::Arc;
    ///
    /// let effect = GameplayEffectDefinition::new("buff")
    ///     .add_component(Arc::new(ChanceToApplyComponent::new(0.5)))
    ///     .add_component(Arc::new(AdditionalEffectsComponent::new()
    ///         .on_application(vec!["apply_damage".into()])));
    /// ```
    pub fn add_component(
        mut self,
        component: crate::effects::ge_component::BoxedGameplayEffectComponent,
    ) -> Self {
        self.components.push(component);
        self
    }

    /// Adds a conditional effect that applies if source tags match.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let effect = GameplayEffectDefinition::new("primary_damage")
    ///     .add_conditional_effect(
    ///         ConditionalGameplayEffect::new("bonus_damage")
    ///             .require_source_tag(tag!("Buff.CriticalHit"), &tags_manager)
    ///     );
    /// ```
    pub fn add_conditional_effect(mut self, conditional: ConditionalGameplayEffect) -> Self {
        self.conditional_effects.push(conditional);
        self
    }
}

/// Resource that stores all gameplay effect definitions.
#[derive(Resource, Default)]
pub struct GameplayEffectRegistry {
    pub definitions: std::collections::HashMap<Atom, GameplayEffectDefinition>,
}

impl GameplayEffectRegistry {
    /// Creates a new empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers an effect definition.
    ///
    /// # Panics
    ///
    /// Panics if an Instant effect has `granted_tags`, since there is no persistent
    /// entity to hold them and remove them later. Use `HasDuration` or `Infinite` instead.
    pub fn register(&mut self, definition: GameplayEffectDefinition) {
        if definition.duration_policy == DurationPolicy::Instant
            && !definition.granted_tags.is_empty()
        {
            panic!(
                "Instant effect '{}' has granted_tags, which cannot be cleaned up. \
                 Use DurationPolicy::HasDuration or DurationPolicy::Infinite instead.",
                definition.id
            );
        }
        self.definitions.insert(definition.id.clone(), definition);
    }

    /// Gets an effect definition by ID.
    pub fn get(&self, id: impl Into<Atom>) -> Option<&GameplayEffectDefinition> {
        self.definitions.get(&id.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magnitude_calculation_scalar() {
        let mag = MagnitudeCalculation::scalar(10.0);
        assert_eq!(mag.evaluate(1, None), 10.0);
    }

    #[test]
    fn test_magnitude_calculation_attribute_based() {
        let mag = MagnitudeCalculation::from_source_attribute("Strength", 2.0);
        assert_eq!(mag.evaluate(1, Some(5.0)), 10.0);
    }

    #[test]
    fn test_effect_definition_builder() {
        let effect = GameplayEffectDefinition::new("test_effect")
            .with_duration(5.0)
            .with_period(1.0)
            .add_modifier(ModifierInfo::new(
                "Health",
                ModifierOperation::AddCurrent,
                MagnitudeCalculation::scalar(10.0),
            ));

        assert_eq!(effect.id, Atom::from("test_effect"));
        assert_eq!(effect.duration_policy, DurationPolicy::HasDuration);
        assert_eq!(effect.duration_magnitude, 5.0);
        assert_eq!(effect.period, 1.0);
        assert_eq!(effect.modifiers.len(), 1);
    }

    #[test]
    fn test_registry() {
        let mut registry = GameplayEffectRegistry::new();
        let effect = GameplayEffectDefinition::new("test");
        registry.register(effect);

        assert!(registry.get("test").is_some());
        assert!(registry.get("nonexistent").is_none());
    }
}
