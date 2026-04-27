//! Effect system implementations.
//!
//! This module contains the observer functions and systems that manage gameplay effects.

use super::components::*;
use super::definition::*;
use crate::attributes::{
    AttributeData, AttributeLifecycleHooks, AttributeModifyContext, AttributeName, AttributeSetId,
};
use crate::core::OwnedTags;
use crate::effects::application_requirement::{
    ApplicationAttributeSnapshot, ApplicationContext, ApplicationRequirementRegistry,
};
use bevy::ecs::relationship::Relationship;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_gameplay_tag::GameplayTagsManager;
use string_cache::DefaultAtom as Atom;

/// Bundled query parameters for applying gameplay effects.
#[derive(SystemParam)]
pub struct ApplyEffectParams<'w, 's> {
    pub tag_containers: Query<'w, 's, &'static mut OwnedTags>,
    pub immunity_tags: Query<'w, 's, &'static crate::core::ImmunityTags>,
    pub attributes: Query<
        'w,
        's,
        (
            &'static mut AttributeData,
            &'static AttributeName,
            &'static ChildOf,
        ),
    >,
    pub existing_effects: Query<
        'w,
        's,
        (
            Entity,
            &'static mut ActiveGameplayEffect,
            &'static EffectTarget,
            Option<&'static mut EffectDuration>,
        ),
    >,
}

/// Event for applying a gameplay effect.
#[derive(Event, Debug, Clone)]
pub struct ApplyGameplayEffectEvent {
    /// Complete runtime spec for this application.
    pub spec: GameplayEffectSpec,
}

impl ApplyGameplayEffectEvent {
    /// Creates an effect application event from a spec.
    pub fn from_spec(spec: GameplayEffectSpec) -> Self {
        Self { spec }
    }

    /// Creates an effect application event targeting an entity at level 1.
    pub fn new(effect_id: impl Into<Atom>, target: Entity) -> Self {
        Self::from_spec(GameplayEffectSpec::new(effect_id, target))
    }

    /// Sets the level on the contained spec.
    pub fn with_level(mut self, level: i32) -> Self {
        self.spec.level = level;
        self
    }

    /// Sets the source entity on the contained spec.
    pub fn with_source(mut self, source: Entity) -> Self {
        self.spec.context.source = Some(source);
        self
    }

    /// Sets the instigator entity on the contained spec.
    pub fn with_instigator(mut self, instigator: Entity) -> Self {
        self.spec.context.instigator = Some(instigator);
        self
    }

    /// Adds a SetByCaller magnitude to the contained spec.
    pub fn with_set_by_caller_magnitude(
        mut self,
        tag: bevy_gameplay_tag::gameplay_tag::GameplayTag,
        magnitude: f32,
    ) -> Self {
        self.spec
            .set_by_caller_magnitudes
            .set_magnitude(tag, magnitude);
        self
    }

    /// Returns the effect definition ID.
    pub fn effect_id(&self) -> &Atom {
        &self.spec.effect_id
    }

    /// Returns the target entity.
    pub fn target(&self) -> Entity {
        self.spec.target
    }

    /// Returns the legacy instigator field.
    pub fn instigator(&self) -> Option<Entity> {
        self.spec.instigator()
    }

    /// Returns the application level.
    pub fn level(&self) -> i32 {
        self.spec.level
    }
}

/// Event triggered when an effect is applied.
#[derive(Event, Debug, Clone)]
pub struct GameplayEffectAppliedEvent {
    /// The effect entity (None for instant effects that modify base_value directly).
    pub effect: Entity,
    /// The target entity.
    pub target: Entity,
    /// The effect definition ID.
    pub effect_id: Atom,
}

/// Event triggered when an effect is removed.
#[derive(Event, Debug, Clone)]
pub struct GameplayEffectRemovedEvent {
    /// The effect entity.
    pub effect: Entity,
    /// The target entity.
    pub target: Entity,
    /// The effect definition ID.
    pub effect_id: Atom,
}

/// Event triggered when an effect is blocked by immunity.
#[derive(Event, Debug, Clone)]
pub struct GameplayEffectBlockedByImmunityEvent {
    /// The effect definition ID that was blocked.
    pub effect_id: Atom,
    /// The target entity that has immunity.
    pub target: Entity,
    /// The instigator entity (if any).
    pub instigator: Option<Entity>,
    /// The immunity tag that blocked the effect.
    pub immunity_tag: bevy_gameplay_tag::gameplay_tag::GameplayTag,
}

fn calculate_modifier_magnitude(
    magnitude: &MagnitudeCalculation,
    level: i32,
    source_entity: Option<Entity>,
    target_entity: Entity,
    set_by_caller: Option<&SetByCallerMagnitudes>,
    custom_calculators: &super::custom_calculation::CustomCalculationRegistry,
    attributes: &[ApplicationAttributeSnapshot],
) -> f32 {
    let source_value = match magnitude {
        MagnitudeCalculation::AttributeBased {
            attribute_name,
            capture_source,
            calculation_type,
            ..
        } => {
            use super::definition::{AttributeCalculationType, AttributeCaptureSource};

            let capture_entity = match capture_source {
                AttributeCaptureSource::Source => source_entity,
                AttributeCaptureSource::Target => Some(target_entity),
            };

            capture_entity.and_then(|entity| {
                attributes
                    .iter()
                    .find(|snapshot| {
                        snapshot.owner == entity && snapshot.attribute_name == *attribute_name
                    })
                    .map(|snapshot| match calculation_type {
                        AttributeCalculationType::AttributeMagnitude => snapshot.current_value,
                        AttributeCalculationType::AttributeBaseValue => snapshot.base_value,
                        AttributeCalculationType::AttributeBonusMagnitude => {
                            snapshot.current_value - snapshot.base_value
                        }
                    })
            })
        }
        MagnitudeCalculation::SetByCaller { data_tag } => {
            set_by_caller.and_then(|magnitudes| magnitudes.get_magnitude(data_tag))
        }
        MagnitudeCalculation::CustomClass { calculator_name } => {
            if let Some(calculator) = custom_calculators.get(calculator_name) {
                use super::custom_calculation::CalculationContext;

                let mut source_attrs = std::collections::HashMap::new();
                let mut target_attrs = std::collections::HashMap::new();

                if let Some(source) = source_entity {
                    for attr_name in calculator.required_source_attributes() {
                        if let Some(value) = attributes
                            .iter()
                            .find(|snapshot| {
                                snapshot.owner == source
                                    && snapshot.attribute_name.as_ref() == *attr_name
                            })
                            .map(|snapshot| snapshot.current_value)
                        {
                            source_attrs.insert((*attr_name).into(), value);
                        }
                    }
                }

                for attr_name in calculator.required_target_attributes() {
                    if let Some(value) = attributes
                        .iter()
                        .find(|snapshot| {
                            snapshot.owner == target_entity
                                && snapshot.attribute_name.as_ref() == *attr_name
                        })
                        .map(|snapshot| snapshot.current_value)
                    {
                        target_attrs.insert((*attr_name).into(), value);
                    }
                }

                let context = CalculationContext {
                    source: source_entity,
                    target: target_entity,
                    level,
                    source_attributes: source_attrs,
                    target_attributes: target_attrs,
                };

                Some(calculator.calculate(&context))
            } else {
                warn!(
                    "Custom calculator '{}' not found in registry",
                    calculator_name
                );
                None
            }
        }
        MagnitudeCalculation::ScalableFloat { .. } => None,
    };

    magnitude.evaluate(level, source_value)
}

/// Observer for ApplyGameplayEffectEvent.
pub fn on_apply_gameplay_effect(
    ev: On<ApplyGameplayEffectEvent>,
    mut commands: Commands,
    registry: Res<GameplayEffectRegistry>,
    application_requirements: Res<ApplicationRequirementRegistry>,
    custom_calculators: Res<super::custom_calculation::CustomCalculationRegistry>,
    tags_manager: Res<GameplayTagsManager>,
    time: Res<Time>,
    mut params: ApplyEffectParams,
) {
    let event = ev.event();
    let spec = &event.spec;
    let target = spec.target;
    let effect_id = &spec.effect_id;
    let level = spec.level;

    let Some(definition) = registry.get(effect_id) else {
        warn!("Effect definition not found: {}", effect_id);
        return;
    };

    // Check immunity: if target has immunity tags matching effect's immunity_tags, reject
    if let Ok(target_immunity) = params.immunity_tags.get(target) {
        for immunity_tag in definition.immunity_tags.gameplay_tags.iter() {
            if target_immunity
                .0
                .explicit_tags
                .gameplay_tags
                .contains(immunity_tag)
            {
                // Target is immune to this effect
                info!(
                    "Effect '{}' blocked by immunity tag '{:?}' on target {:?}",
                    effect_id, immunity_tag, target
                );

                // Trigger immunity event
                commands.trigger(GameplayEffectBlockedByImmunityEvent {
                    effect_id: effect_id.clone(),
                    target,
                    instigator: spec.instigator(),
                    immunity_tag: immunity_tag.clone(),
                });

                return;
            }
        }
    }

    // Legacy check: if target has any of the effect's asset_tags in their owned tags, reject
    // This is for backwards compatibility with the old immunity system
    if let Ok(owner_tags) = params.tag_containers.get(target) {
        for asset_tag in definition.asset_tags.gameplay_tags.iter() {
            if owner_tags.0.explicit_tags.gameplay_tags.contains(asset_tag) {
                // Target is immune to this effect
                info!(
                    "Effect '{}' blocked by asset tag immunity '{:?}' on target {:?}",
                    effect_id, asset_tag, target
                );
                return;
            }
        }
    }

    // Check application_tag_requirements
    if let Ok(owner_tags) = params.tag_containers.get(target) {
        // OwnedTags wraps GameplayTagCountContainer which has explicit_tags field
        let tag_container = &owner_tags.0;
        if !definition
            .application_tag_requirements
            .requirements_met(&tag_container.explicit_tags)
        {
            return;
        }
    }

    // Check custom application requirements.
    let target_tags = params.tag_containers.get(target).ok();
    let source_tags = spec
        .source_entity()
        .and_then(|source| params.tag_containers.get(source).ok());

    let attribute_snapshots: Vec<_> = params
        .attributes
        .iter()
        .map(|(data, name, child_of)| ApplicationAttributeSnapshot::new(child_of.get(), name, data))
        .collect();

    for requirement_name in &definition.application_requirements {
        let Some(requirement) = application_requirements.get(requirement_name) else {
            warn!(
                "Effect '{}' references unknown application requirement '{}'",
                effect_id, requirement_name
            );
            return;
        };

        let context = ApplicationContext {
            source: spec.source_entity(),
            target,
            level,
            target_tags: target_tags.as_ref().copied(),
            source_tags: source_tags.as_ref().copied(),
            attributes: &attribute_snapshots,
        };

        if !requirement.can_apply(&context) {
            return;
        }
    }

    // Handle stacking
    match definition.stacking_policy {
        StackingPolicy::RefreshDuration => {
            // Find existing effect and refresh its duration
            for (effect_entity, active_effect, effect_target, duration) in
                params.existing_effects.iter_mut()
            {
                if effect_target.0 == target && active_effect.definition_id == *effect_id {
                    if let Some(mut dur) = duration {
                        dur.remaining = definition.duration_magnitude;
                    }
                    // Trigger applied event for the existing effect
                    commands.trigger(GameplayEffectAppliedEvent {
                        effect: effect_entity,
                        target,
                        effect_id: effect_id.clone(),
                    });
                    return;
                }
            }
            // Fall through to spawn new if no existing found
        }
        StackingPolicy::StackCount { max_stacks } => {
            // Find existing effect and increment stack count
            for (effect_entity, mut active_effect, effect_target, _) in
                params.existing_effects.iter_mut()
            {
                if effect_target.0 == target && active_effect.definition_id == *effect_id {
                    if active_effect.stack_count < max_stacks {
                        // Increment stack count
                        active_effect.stack_count += 1;

                        // IMPORTANT: We need to spawn new modifiers for the new stack
                        // The create_effect_modifiers_system will handle this automatically
                        // when it sees the Changed<ActiveGameplayEffect> component

                        commands.trigger(GameplayEffectAppliedEvent {
                            effect: effect_entity,
                            target,
                            effect_id: effect_id.clone(),
                        });
                    }
                    // If at max stacks, do nothing (could optionally refresh duration)
                    return;
                }
            }
            // Fall through to spawn new if no existing found
        }
        StackingPolicy::Independent => {
            // Always spawn a new effect entity
        }
    }

    match definition.duration_policy {
        DurationPolicy::Instant => {
            // Directly modify attribute base_value, no entity spawn
            for modifier in &definition.modifiers {
                let magnitude = calculate_modifier_magnitude(
                    &modifier.magnitude,
                    level,
                    spec.source_entity(),
                    target,
                    Some(&spec.set_by_caller_magnitudes),
                    &custom_calculators,
                    &attribute_snapshots,
                );
                for (mut attr_data, attr_name, attr_owner) in params.attributes.iter_mut() {
                    if attr_owner.0 == target && attr_name.0 == modifier.attribute_name {
                        let old_value = attr_data.base_value;
                        let new_value = match modifier.operation {
                            ModifierOperation::AddBase | ModifierOperation::AddCurrent => {
                                old_value + magnitude
                            }
                            ModifierOperation::MultiplyAdditive
                            | ModifierOperation::MultiplyMultiplicative => {
                                old_value * (1.0 + magnitude)
                            }
                            ModifierOperation::Override => magnitude,
                        };

                        // Call pre_effect_execute hook (allows clamping/rejection)
                        // TODO: Get AttributeSetId to look up hooks
                        // For now, we skip hooks for instant effects
                        // This will be implemented when we add AttributeSnapshot

                        // Apply the modification
                        attr_data.base_value = new_value;
                        // Don't set current_value - let aggregation handle it

                        // Call post_effect_execute hook
                        // TODO: Implement when AttributeSnapshot is added
                    }
                }
            }

            // WARNING: Instant effects with granted_tags cause tag leaks!
            // Tags are added but never removed since no entity persists.
            // Solution: Either forbid granted_tags on instant effects, or
            // spawn a temporary entity that despawns after one frame.
            // For now, we log a warning and skip adding tags.
            if !definition.granted_tags.is_empty() {
                warn!(
                    "Instant effect '{}' has granted_tags, which will leak. \
                     Instant effects should not grant tags. Use HasDuration instead.",
                    effect_id
                );
                // Do NOT add tags - they would never be removed
            }

            // Use PLACEHOLDER since no entity is spawned for instant effects
            commands.trigger(GameplayEffectAppliedEvent {
                effect: Entity::PLACEHOLDER,
                target,
                effect_id: effect_id.clone(),
            });
        }
        DurationPolicy::HasDuration | DurationPolicy::Infinite => {
            // Spawn effect entity with components
            let mut effect_entity_commands = commands.spawn((
                ActiveGameplayEffect::new(effect_id.clone(), level, time.elapsed_secs()),
                EffectTarget(target),
                EffectInstigator(spec.instigator()),
                spec.context.clone(),
            ));

            if !spec.set_by_caller_magnitudes.is_empty() {
                effect_entity_commands.insert(spec.set_by_caller_magnitudes.clone());
            }

            // Add duration component for HasDuration
            if definition.duration_policy == DurationPolicy::HasDuration {
                effect_entity_commands.insert(EffectDuration::new(definition.duration_magnitude));
            }

            // Add periodic component if needed
            if definition.period > 0.0 {
                effect_entity_commands.insert(PeriodicEffect::new(definition.period));
            }

            // Add granted tags component
            if !definition.granted_tags.is_empty() {
                effect_entity_commands.insert(EffectGrantedTags {
                    tags: definition.granted_tags.clone(),
                });
            }

            let effect_entity = effect_entity_commands.id();

            // Add granted_tags to target's OwnedTags
            if !definition.granted_tags.is_empty()
                && let Ok(mut target_tags) = params.tag_containers.get_mut(target)
            {
                target_tags.0.update_tag_container_count(
                    &definition.granted_tags,
                    1,
                    &tags_manager,
                    &mut commands,
                    target,
                );
            }

            commands.trigger(GameplayEffectAppliedEvent {
                effect: effect_entity,
                target,
                effect_id: effect_id.clone(),
            });
        }
    }
}

/// System that creates modifier entities for active effects.
///
/// This system runs when:
/// 1. A new effect is added (Added<ActiveGameplayEffect>)
/// 2. An effect's stack count changes (Changed<ActiveGameplayEffect>)
///
/// For stacking effects, we need to create additional modifiers when stack count increases.
/// For AttributeBased and SetByCaller magnitudes, we need to capture/lookup values.
pub fn create_effect_modifiers_system(
    mut commands: Commands,
    registry: Res<GameplayEffectRegistry>,
    custom_calculators: Res<super::custom_calculation::CustomCalculationRegistry>,
    new_or_changed_effects: Query<
        (
            Entity,
            &ActiveGameplayEffect,
            &EffectTarget,
            Option<&EffectInstigator>,
            Option<&SetByCallerMagnitudes>,
            Option<&GameplayEffectContext>,
        ),
        Or<(Added<ActiveGameplayEffect>, Changed<ActiveGameplayEffect>)>,
    >,
    existing_modifiers: Query<(Entity, &ModifierSource)>,
    attributes: Query<(&AttributeData, &AttributeName, &ChildOf)>,
) {
    for (effect_entity, active_effect, target, instigator, set_by_caller, context) in
        new_or_changed_effects.iter()
    {
        let Some(definition) = registry.get(&active_effect.definition_id) else {
            continue;
        };

        // Count existing modifiers for this effect
        let existing_modifier_count = existing_modifiers
            .iter()
            .filter(|(_, source)| source.0 == effect_entity)
            .count();

        // Calculate how many modifier sets we need total
        let needed_modifier_sets = active_effect.stack_count as usize;
        let modifiers_per_set = definition.modifiers.len();
        let needed_total = needed_modifier_sets * modifiers_per_set;

        // If we already have the right number, skip
        if existing_modifier_count == needed_total {
            continue;
        }

        // If we have too many (stack decreased), remove excess
        if existing_modifier_count > needed_total {
            let to_remove = existing_modifier_count - needed_total;
            let mut removed = 0;
            for (modifier_entity, source) in existing_modifiers.iter() {
                if source.0 == effect_entity && removed < to_remove {
                    commands.entity(modifier_entity).despawn();
                    removed += 1;
                }
            }
            continue;
        }

        // We need more modifiers - spawn one complete set per missing stack
        let missing_stacks = (needed_total - existing_modifier_count) / modifiers_per_set;

        let attribute_snapshots: Vec<_> = attributes
            .iter()
            .map(|(data, name, child_of)| {
                ApplicationAttributeSnapshot::new(child_of.get(), name, data)
            })
            .collect();
        let source_entity = context
            .and_then(|context| context.source.or(context.instigator))
            .or_else(|| instigator.and_then(|instigator| instigator.0));

        for _ in 0..missing_stacks {
            for modifier_info in &definition.modifiers {
                let magnitude = calculate_modifier_magnitude(
                    &modifier_info.magnitude,
                    active_effect.level,
                    source_entity,
                    target.0,
                    set_by_caller,
                    &custom_calculators,
                    &attribute_snapshots,
                );

                commands.spawn((
                    AttributeModifier {
                        target_entity: target.0,
                        target_attribute: modifier_info.attribute_name.clone(),
                        operation: modifier_info.operation,
                        magnitude,
                    },
                    ModifierSource(effect_entity),
                ));
            }
        }
    }
}

/// System that aggregates attribute modifiers and applies them to attributes.
pub fn aggregate_attribute_modifiers_system(
    mut attributes: Query<(
        Entity,
        &mut AttributeData,
        &AttributeName,
        &ChildOf,
        &AttributeSetId,
    )>,
    modifiers: Query<&AttributeModifier>,
    hooks: Option<Res<AttributeLifecycleHooks>>,
) {
    for (attr_entity, mut attr_data, attr_name, child_of, set_id) in attributes.iter_mut() {
        let owner = child_of.get();
        let mut applicable_modifiers: Vec<_> = modifiers
            .iter()
            .filter(|m| m.target_entity == owner && m.target_attribute == attr_name.0)
            .collect();

        applicable_modifiers.sort_by_key(|m| m.operation.priority());

        // Check for Override first (short-circuit)
        if let Some(override_mod) = applicable_modifiers
            .iter()
            .find(|m| matches!(m.operation, ModifierOperation::Override))
        {
            let old_value = attr_data.current_value;
            let new_value = override_mod.magnitude;
            if (old_value - new_value).abs() > f32::EPSILON {
                let mut context = AttributeModifyContext {
                    owner,
                    attribute: attr_entity,
                    attribute_name: attr_name.0.clone(),
                    old_value,
                    new_value,
                    source_effect: None,
                };

                // Call pre hook for this AttributeSet
                if let Some(hooks_res) = &hooks
                    && let Some(set_hooks) = hooks_res.get(set_id.0)
                {
                    (set_hooks.pre_change)(&mut context);
                }

                attr_data.current_value = context.new_value;

                // Call post hook
                if let Some(hooks_res) = &hooks
                    && let Some(set_hooks) = hooks_res.get(set_id.0)
                {
                    (set_hooks.post_change)(&context);
                }
            }
            continue;
        }

        // UE aggregation formula: ((BaseValue + AddBase) * MultiplyAdditive * MultiplyCompound) + AddFinal
        // Note: We don't have DivideAdditive or AddFinal yet, but the structure is here

        let mut current = attr_data.base_value;

        // Step 1: AddBase - adds to base value (from持续效果的修改器)
        for modifier in applicable_modifiers
            .iter()
            .filter(|m| matches!(m.operation, ModifierOperation::AddBase))
        {
            current += modifier.magnitude;
        }

        // Step 2: AddCurrent - adds to current value
        for modifier in applicable_modifiers
            .iter()
            .filter(|m| matches!(m.operation, ModifierOperation::AddCurrent))
        {
            current += modifier.magnitude;
        }

        // Step 3: MultiplyAdditive - multipliers are summed then applied: (1 + sum)
        // E.g. 50% + 50% = 100% bonus = 1.5 + 1.5 = 2.0 multiplier
        let additive_multiplier: f32 = applicable_modifiers
            .iter()
            .filter(|m| matches!(m.operation, ModifierOperation::MultiplyAdditive))
            .map(|m| m.magnitude)
            .sum();
        current *= 1.0 + additive_multiplier;

        // Step 4: MultiplyMultiplicative (Compound) - each multiplier is applied separately: prod(1 + m)
        // E.g. 50% * 50% = 1.5 * 1.5 = 2.25 multiplier
        for modifier in applicable_modifiers
            .iter()
            .filter(|m| matches!(m.operation, ModifierOperation::MultiplyMultiplicative))
        {
            current *= 1.0 + modifier.magnitude;
        }

        let old_value = attr_data.current_value;
        if (current - old_value).abs() > f32::EPSILON {
            let mut context = AttributeModifyContext {
                owner,
                attribute: attr_entity,
                attribute_name: attr_name.0.clone(),
                old_value,
                new_value: current,
                source_effect: None,
            };

            if let Some(hooks_res) = &hooks
                && let Some(set_hooks) = hooks_res.get(set_id.0)
            {
                (set_hooks.pre_change)(&mut context);
            }

            attr_data.current_value = context.new_value;

            if let Some(hooks_res) = &hooks
                && let Some(set_hooks) = hooks_res.get(set_id.0)
            {
                (set_hooks.post_change)(&context);
            }
        }
    }
}

/// System that updates effect durations.
pub fn update_effect_durations_system(mut effects: Query<&mut EffectDuration>, time: Res<Time>) {
    for mut duration in effects.iter_mut() {
        duration.tick(time.delta_secs());
    }
}

/// System that removes expired effects and cleans up granted tags.
pub fn remove_expired_effects_system(
    mut commands: Commands,
    tags_manager: Res<GameplayTagsManager>,
    effects: Query<(
        Entity,
        &EffectDuration,
        &ActiveGameplayEffect,
        &EffectTarget,
        Option<&EffectGrantedTags>,
    )>,
    modifiers: Query<(Entity, &ModifierSource)>,
    mut tag_containers: Query<&mut OwnedTags>,
) {
    for (effect_entity, duration, active_effect, target, granted_tags) in effects.iter() {
        if duration.is_expired() {
            // Remove granted_tags from target's OwnedTags
            if let Some(granted) = granted_tags
                && let Ok(mut target_tags) = tag_containers.get_mut(target.0)
            {
                target_tags.0.update_tag_container_count(
                    &granted.tags,
                    -1,
                    &tags_manager,
                    &mut commands,
                    target.0,
                );
            }

            // Remove all modifiers created by this effect
            for (modifier_entity, source) in modifiers.iter() {
                if source.0 == effect_entity {
                    commands.entity(modifier_entity).despawn();
                }
            }

            // Trigger removal event
            commands.trigger(GameplayEffectRemovedEvent {
                effect: effect_entity,
                target: target.0,
                effect_id: active_effect.definition_id.clone(),
            });

            // Remove the effect
            commands.entity(effect_entity).despawn();
        }
    }
}

/// System that executes periodic effects.
///
/// Periodic effects apply their modifiers at regular intervals. This system:
/// 1. Ticks the periodic timer
/// 2. For each execution, re-applies the effect's modifiers to the target's base value
///
/// Note: Periodic effects modify the base value permanently on each tick.
/// This is consistent with UE GAS behavior where periodic damage/healing
/// permanently changes the attribute.
pub fn execute_periodic_effects_system(
    mut effects: Query<(&mut PeriodicEffect, &ActiveGameplayEffect, &EffectTarget)>,
    registry: Res<GameplayEffectRegistry>,
    mut attributes: Query<(&mut AttributeData, &AttributeName, &ChildOf)>,
    time: Res<Time>,
) {
    for (mut periodic, active_effect, target) in effects.iter_mut() {
        let executions = periodic.tick(time.delta_secs());

        if executions == 0 {
            continue;
        }

        // Get the effect definition
        let Some(definition) = registry.get(&active_effect.definition_id) else {
            warn!(
                "Periodic effect references unknown definition: {}",
                active_effect.definition_id
            );
            continue;
        };

        // Apply modifiers for each execution
        for _ in 0..executions {
            for modifier in &definition.modifiers {
                let magnitude = modifier.magnitude.evaluate(active_effect.level, None);

                // Find and modify the target attribute
                for (mut attr_data, attr_name, child_of) in attributes.iter_mut() {
                    let owner = child_of.get();
                    if owner == target.0 && attr_name.0 == modifier.attribute_name {
                        match modifier.operation {
                            ModifierOperation::AddBase | ModifierOperation::AddCurrent => {
                                attr_data.base_value += magnitude;
                            }
                            ModifierOperation::MultiplyAdditive
                            | ModifierOperation::MultiplyMultiplicative => {
                                attr_data.base_value *= 1.0 + magnitude;
                            }
                            ModifierOperation::Override => {
                                attr_data.base_value = magnitude;
                            }
                        }
                        // Aggregation system will recalculate current_value
                    }
                }
            }
        }
    }
}

/// System that removes instant effects after they've been applied.
pub fn remove_instant_effects_system(
    mut commands: Commands,
    registry: Res<GameplayEffectRegistry>,
    instant_effects: Query<(Entity, &ActiveGameplayEffect), Added<ActiveGameplayEffect>>,
) {
    for (effect_entity, active_effect) in instant_effects.iter() {
        if let Some(definition) = registry.get(&active_effect.definition_id)
            && definition.duration_policy == DurationPolicy::Instant
        {
            commands.entity(effect_entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Resource, Default)]
    struct ReceivedApplyEvents(Vec<ApplyGameplayEffectEvent>);

    #[derive(Resource, Default)]
    struct ReceivedAppliedEvents(Vec<GameplayEffectAppliedEvent>);

    #[test]
    fn test_apply_effect_event() {
        let mut app = App::new();
        app.add_plugins(bevy_gameplay_tag::GameplayTagsPlugin::with_data_path(
            "assets/gameplay_tags.json".to_string(),
        ));
        app.init_resource::<ReceivedApplyEvents>();
        app.init_resource::<ReceivedAppliedEvents>();
        app.init_resource::<GameplayEffectRegistry>();
        app.init_resource::<ApplicationRequirementRegistry>();
        app.init_resource::<crate::effects::custom_calculation::CustomCalculationRegistry>();
        app.init_resource::<Time>();
        app.add_observer(on_apply_gameplay_effect);
        app.update();

        app.add_observer(
            |ev: On<ApplyGameplayEffectEvent>, mut received: ResMut<ReceivedApplyEvents>| {
                received.0.push(ev.event().clone());
            },
        );
        app.add_observer(
            |ev: On<GameplayEffectAppliedEvent>, mut received: ResMut<ReceivedAppliedEvents>| {
                received.0.push(ev.event().clone());
            },
        );

        let effect = GameplayEffectDefinition::new("test_effect").with_duration(5.0);
        app.world_mut()
            .resource_mut::<GameplayEffectRegistry>()
            .register(effect);

        let target = app.world_mut().spawn(OwnedTags::default()).id();

        app.world_mut().trigger(
            ApplyGameplayEffectEvent::new(Atom::from("test_effect"), target).with_level(1),
        );

        app.update();

        let apply_events = app.world().resource::<ReceivedApplyEvents>();
        assert_eq!(apply_events.0.len(), 1);
        assert_eq!(apply_events.0[0].effect_id(), &Atom::from("test_effect"));
        assert_eq!(apply_events.0[0].target(), target);
    }
}
