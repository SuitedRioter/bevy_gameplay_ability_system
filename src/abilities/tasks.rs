//! Ability task system.
//!
//! Tasks are ECS entities that represent ongoing operations within an ability.
//! They are spawned as children of the ability instance entity and automatically
//! cleaned up when the instance ends.
//!
//! This is the ECS equivalent of UE's `UAbilityTask`.

use bevy::ecs::relationship::Relationship;
use bevy::prelude::*;
use bevy_gameplay_tag::gameplay_tag::GameplayTag;
use string_cache::DefaultAtom as Atom;

use super::events::GameplayEvent;
use crate::effects::systems::{
    ApplyGameplayEffectEvent, GameplayEffectAppliedEvent, GameplayEffectRemovedEvent,
};

/// Marker component for ability task entities.
///
/// Tasks are spawned as children of ability instance entities.
#[derive(Component, Debug, Clone)]
pub struct AbilityTask {
    /// The ability instance that owns this task.
    pub ability_instance: Option<Entity>,
    /// The ability spec entity.
    pub ability_spec: Entity,
    /// The owner entity (character).
    pub owner: Entity,
}

/// Task state component.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    /// Task is running and waiting for completion.
    Running,
    /// Task has completed successfully.
    Completed,
    /// Task was cancelled or failed.
    Cancelled,
}

impl Default for TaskState {
    fn default() -> Self {
        Self::Running
    }
}

/// WaitDelay task - waits for a specified duration.
///
/// Completes after the specified time has elapsed.
#[derive(Component, Debug, Clone)]
pub struct WaitDelayTask {
    /// Remaining time in seconds.
    pub remaining: f32,
}

impl WaitDelayTask {
    /// Create a new wait delay task.
    pub fn new(duration: f32) -> Self {
        Self {
            remaining: duration,
        }
    }
}

/// WaitGameplayEvent task - waits for a gameplay event with a specific tag.
///
/// Completes when a matching event is received.
#[derive(Component, Debug, Clone)]
pub struct WaitGameplayEventTask {
    /// The event tag to wait for.
    pub event_tag: GameplayTag,
    /// Whether to match only events targeting the owner.
    pub only_trigger_once: bool,
    /// Whether the event has been received.
    pub triggered: bool,
}

impl WaitGameplayEventTask {
    /// Create a new wait gameplay event task.
    pub fn new(event_tag: GameplayTag) -> Self {
        Self {
            event_tag,
            only_trigger_once: true,
            triggered: false,
        }
    }

    /// Set whether to trigger only once.
    pub fn with_only_trigger_once(mut self, only_once: bool) -> Self {
        self.only_trigger_once = only_once;
        self
    }
}

/// Comparison operator for attribute value checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeComparison {
    /// Value is less than threshold.
    LessThan,
    /// Value is less than or equal to threshold.
    LessThanOrEqual,
    /// Value is greater than threshold.
    GreaterThan,
    /// Value is greater than or equal to threshold.
    GreaterThanOrEqual,
    /// Value equals threshold.
    Equal,
    /// Value does not equal threshold.
    NotEqual,
}

impl AttributeComparison {
    /// Check if the value satisfies the comparison with the threshold.
    pub fn check(&self, value: f32, threshold: f32) -> bool {
        match self {
            Self::LessThan => value < threshold,
            Self::LessThanOrEqual => value <= threshold,
            Self::GreaterThan => value > threshold,
            Self::GreaterThanOrEqual => value >= threshold,
            Self::Equal => (value - threshold).abs() < f32::EPSILON,
            Self::NotEqual => (value - threshold).abs() >= f32::EPSILON,
        }
    }
}

/// WaitAttributeChange task - waits for an attribute value to change or reach a threshold.
///
/// Completes when the specified attribute meets the comparison condition.
#[derive(Component, Debug, Clone)]
pub struct WaitAttributeChangeTask {
    /// The attribute name to watch.
    pub attribute_name: String,
    /// The comparison operator.
    pub comparison: AttributeComparison,
    /// The threshold value to compare against.
    pub threshold: f32,
    /// Whether to trigger only once.
    pub only_trigger_once: bool,
    /// Whether the condition has been met.
    pub triggered: bool,
}

impl WaitAttributeChangeTask {
    /// Create a new wait attribute change task.
    pub fn new(attribute_name: impl Into<String>, comparison: AttributeComparison, threshold: f32) -> Self {
        Self {
            attribute_name: attribute_name.into(),
            comparison,
            threshold,
            only_trigger_once: true,
            triggered: false,
        }
    }

    /// Set whether to trigger only once.
    pub fn with_only_trigger_once(mut self, only_once: bool) -> Self {
        self.only_trigger_once = only_once;
        self
    }
}

/// WaitEffectApplied task - waits for a gameplay effect to be applied to the owner.
///
/// Completes when an effect with the specified definition ID is applied.
#[derive(Component, Debug, Clone)]
pub struct WaitEffectAppliedTask {
    /// The effect definition ID to wait for (None = any effect).
    pub effect_definition_id: Option<String>,
    /// Whether to trigger only once.
    pub only_trigger_once: bool,
    /// Whether the effect has been applied.
    pub triggered: bool,
}

impl WaitEffectAppliedTask {
    /// Create a new wait effect applied task for any effect.
    pub fn new() -> Self {
        Self {
            effect_definition_id: None,
            only_trigger_once: true,
            triggered: false,
        }
    }

    /// Create a new wait effect applied task for a specific effect.
    pub fn for_effect(effect_definition_id: impl Into<String>) -> Self {
        Self {
            effect_definition_id: Some(effect_definition_id.into()),
            only_trigger_once: true,
            triggered: false,
        }
    }

    /// Set whether to trigger only once.
    pub fn with_only_trigger_once(mut self, only_once: bool) -> Self {
        self.only_trigger_once = only_once;
        self
    }
}

impl Default for WaitEffectAppliedTask {
    fn default() -> Self {
        Self::new()
    }
}

/// WaitEffectRemoved task - waits for a gameplay effect to be removed from the owner.
///
/// Completes when an effect with the specified definition ID is removed.
#[derive(Component, Debug, Clone)]
pub struct WaitEffectRemovedTask {
    /// The effect definition ID to wait for (None = any effect).
    pub effect_definition_id: Option<String>,
    /// Whether to trigger only once.
    pub only_trigger_once: bool,
    /// Whether the effect has been removed.
    pub triggered: bool,
}

impl WaitEffectRemovedTask {
    /// Create a new wait effect removed task for any effect.
    pub fn new() -> Self {
        Self {
            effect_definition_id: None,
            only_trigger_once: true,
            triggered: false,
        }
    }

    /// Create a new wait effect removed task for a specific effect.
    pub fn for_effect(effect_definition_id: impl Into<String>) -> Self {
        Self {
            effect_definition_id: Some(effect_definition_id.into()),
            only_trigger_once: true,
            triggered: false,
        }
    }

    /// Set whether to trigger only once.
    pub fn with_only_trigger_once(mut self, only_once: bool) -> Self {
        self.only_trigger_once = only_once;
        self
    }
}

impl Default for WaitEffectRemovedTask {
    fn default() -> Self {
        Self::new()
    }
}

/// ApplyEffectToTargetData task - applies a gameplay effect to target data.
///
/// This task applies an effect to all actors in the target data.
#[derive(Component, Debug, Clone)]
pub struct ApplyEffectToTargetDataTask {
    /// The effect definition ID to apply.
    pub effect_definition_id: Atom,
    /// The target data containing actors to apply the effect to.
    pub target_data: crate::abilities::GameplayAbilityTargetData,
    /// The level at which to apply the effect.
    pub level: i32,
    /// Whether the task has been executed.
    pub executed: bool,
}

impl ApplyEffectToTargetDataTask {
    /// Create a new apply effect to target data task.
    pub fn new(
        effect_definition_id: impl Into<Atom>,
        target_data: crate::abilities::GameplayAbilityTargetData,
        level: i32,
    ) -> Self {
        Self {
            effect_definition_id: effect_definition_id.into(),
            target_data,
            level,
            executed: false,
        }
    }
}

/// Event triggered when a task completes.
#[derive(Event, Debug, Clone)]
pub struct TaskCompletedEvent {
    /// The task entity.
    pub task: Entity,
    /// The ability instance that owns the task.
    pub ability_instance: Option<Entity>,
    /// The ability spec entity.
    pub ability_spec: Entity,
    /// The owner entity.
    pub owner: Entity,
}

/// Event triggered when a task is cancelled.
#[derive(Event, Debug, Clone)]
pub struct TaskCancelledEvent {
    /// The task entity.
    pub task: Entity,
    /// The ability instance that owns the task.
    pub ability_instance: Option<Entity>,
    /// The ability spec entity.
    pub ability_spec: Entity,
    /// The owner entity.
    pub owner: Entity,
}

// --- Systems ---

/// System that ticks WaitDelay tasks.
pub fn tick_wait_delay_tasks_system(
    mut commands: Commands,
    time: Res<Time>,
    mut tasks: Query<(Entity, &AbilityTask, &mut WaitDelayTask, &mut TaskState)>,
) {
    for (task_entity, ability_task, mut wait_delay, mut state) in tasks.iter_mut() {
        if *state != TaskState::Running {
            continue;
        }

        wait_delay.remaining -= time.delta_secs();

        if wait_delay.remaining <= 0.0 {
            *state = TaskState::Completed;
            commands.trigger(TaskCompletedEvent {
                task: task_entity,
                ability_instance: ability_task.ability_instance,
                ability_spec: ability_task.ability_spec,
                owner: ability_task.owner,
            });
        }
    }
}

/// System that handles gameplay events for WaitGameplayEvent tasks.
///
/// Note: This system uses an observer pattern instead of EventReader.
/// Events are handled via the GameplayEvent observer system.
pub fn handle_gameplay_event_for_tasks_system(
    trigger: On<GameplayEvent>,
    mut commands: Commands,
    mut tasks: Query<(
        Entity,
        &AbilityTask,
        &mut WaitGameplayEventTask,
        &mut TaskState,
    )>,
) {
    let event = trigger.event();

    for (task_entity, ability_task, mut wait_event, mut state) in tasks.iter_mut() {
        if *state != TaskState::Running {
            continue;
        }

        // Check if event tag matches
        if event.event_tag != wait_event.event_tag {
            continue;
        }

        // Check if event targets the owner (if target is specified)
        if let Some(target) = event.target {
            if target != ability_task.owner {
                continue;
            }
        }

        // Mark as triggered
        wait_event.triggered = true;

        if wait_event.only_trigger_once {
            *state = TaskState::Completed;
            commands.trigger(TaskCompletedEvent {
                task: task_entity,
                ability_instance: ability_task.ability_instance,
                ability_spec: ability_task.ability_spec,
                owner: ability_task.owner,
            });
        }
    }
}

/// System that cleans up completed or cancelled tasks.
pub fn cleanup_finished_tasks_system(
    mut commands: Commands,
    tasks: Query<(Entity, &TaskState), Changed<TaskState>>,
) {
    for (task_entity, state) in tasks.iter() {
        if *state == TaskState::Completed || *state == TaskState::Cancelled {
            commands.entity(task_entity).despawn();
        }
    }
}

/// System that checks WaitAttributeChange tasks.
///
/// Only runs when attributes are modified (Changed filter), then checks
/// whether the new value meets each task's comparison condition.
pub fn check_wait_attribute_change_tasks_system(
    mut commands: Commands,
    mut tasks: Query<(
        Entity,
        &AbilityTask,
        &mut WaitAttributeChangeTask,
        &mut TaskState,
    )>,
    attributes: Query<
        (Entity, &crate::attributes::AttributeData, &crate::attributes::AttributeName),
        Changed<crate::attributes::AttributeData>,
    >,
    child_of: Query<&ChildOf>,
) {
    for (task_entity, ability_task, mut wait_attr, mut state) in tasks.iter_mut() {
        if *state != TaskState::Running {
            continue;
        }

        // Skip if already triggered and only_trigger_once
        if wait_attr.triggered && wait_attr.only_trigger_once {
            continue;
        }

        // Check each changed attribute — see if it matches this task
        for (attr_entity, attr_data, attr_name) in attributes.iter() {
            if attr_name.0.as_ref() != wait_attr.attribute_name.as_str() {
                continue;
            }

            // Check if this attribute belongs to the owner
            if let Ok(co) = child_of.get(attr_entity) {
                if co.get() != ability_task.owner {
                    continue;
                }
            } else {
                continue;
            }

            // Check if the condition is met
            if wait_attr
                .comparison
                .check(attr_data.current_value, wait_attr.threshold)
            {
                wait_attr.triggered = true;

                if wait_attr.only_trigger_once {
                    *state = TaskState::Completed;
                    commands.trigger(TaskCompletedEvent {
                        task: task_entity,
                        ability_instance: ability_task.ability_instance,
                        ability_spec: ability_task.ability_spec,
                        owner: ability_task.owner,
                    });
                }
            }
        }
    }
}

/// Observer that handles effect applied events for WaitEffectApplied tasks.
pub fn on_effect_applied_for_tasks(
    trigger: On<GameplayEffectAppliedEvent>,
    mut commands: Commands,
    mut tasks: Query<(
        Entity,
        &AbilityTask,
        &mut WaitEffectAppliedTask,
        &mut TaskState,
    )>,
    effects: Query<&crate::effects::components::ActiveGameplayEffect>,
) {
    let event = trigger.event();

    // Get the effect definition ID
    let effect_def_id = if let Ok(effect) = effects.get(event.effect) {
        effect.definition_id.as_ref()
    } else {
        return;
    };

    for (task_entity, ability_task, mut wait_effect, mut state) in tasks.iter_mut() {
        if *state != TaskState::Running {
            continue;
        }

        // Check if the effect is applied to the owner
        if event.target != ability_task.owner {
            continue;
        }

        // Check if the effect definition ID matches (if specified)
        if let Some(ref expected_id) = wait_effect.effect_definition_id {
            if effect_def_id != expected_id {
                continue;
            }
        }

        // Mark as triggered
        wait_effect.triggered = true;

        if wait_effect.only_trigger_once {
            *state = TaskState::Completed;
            commands.trigger(TaskCompletedEvent {
                task: task_entity,
                ability_instance: ability_task.ability_instance,
                ability_spec: ability_task.ability_spec,
                owner: ability_task.owner,
            });
        }
    }
}

/// Observer that handles effect removed events for WaitEffectRemoved tasks.
pub fn on_effect_removed_for_tasks(
    trigger: On<GameplayEffectRemovedEvent>,
    mut commands: Commands,
    mut tasks: Query<(
        Entity,
        &AbilityTask,
        &mut WaitEffectRemovedTask,
        &mut TaskState,
    )>,
) {
    let event = trigger.event();

    for (task_entity, ability_task, mut wait_effect, mut state) in tasks.iter_mut() {
        if *state != TaskState::Running {
            continue;
        }

        // Check if the effect is removed from the owner
        if event.target != ability_task.owner {
            continue;
        }

        // Check if the effect definition ID matches (if specified)
        if let Some(ref expected_id) = wait_effect.effect_definition_id {
            if event.effect_id.as_ref() != expected_id.as_str() {
                continue;
            }
        }

        // Mark as triggered
        wait_effect.triggered = true;

        if wait_effect.only_trigger_once {
            *state = TaskState::Completed;
            commands.trigger(TaskCompletedEvent {
                task: task_entity,
                ability_instance: ability_task.ability_instance,
                ability_spec: ability_task.ability_spec,
                owner: ability_task.owner,
            });
        }
    }
}

/// System that executes ApplyEffectToTargetData tasks.
pub fn execute_apply_effect_to_target_data_tasks_system(
    mut commands: Commands,
    mut tasks: Query<(
        Entity,
        &AbilityTask,
        &mut ApplyEffectToTargetDataTask,
        &mut TaskState,
    )>,
) {
    for (task_entity, ability_task, mut apply_effect, mut state) in tasks.iter_mut() {
        if *state != TaskState::Running || apply_effect.executed {
            continue;
        }

        // Apply effect to all actors in target data
        for &target in &apply_effect.target_data.actors {
            commands.trigger(
                ApplyGameplayEffectEvent::new(apply_effect.effect_definition_id.clone(), target)
                    .with_instigator(ability_task.owner)
                    .with_level(apply_effect.level),
            );
        }

        apply_effect.executed = true;
        *state = TaskState::Completed;
        commands.trigger(TaskCompletedEvent {
            task: task_entity,
            ability_instance: ability_task.ability_instance,
            ability_spec: ability_task.ability_spec,
            owner: ability_task.owner,
        });
    }
}

/// Observer that cancels all tasks when an ability instance is removed.
pub fn on_ability_instance_removed(
    trigger: On<Remove, super::components::AbilitySpecInstance>,
    mut commands: Commands,
    tasks: Query<(Entity, &AbilityTask, &TaskState)>,
) {
    let instance_entity = trigger.entity;

    for (task_entity, ability_task, state) in tasks.iter() {
        if ability_task.ability_instance == Some(instance_entity) && *state == TaskState::Running {
            commands.entity(task_entity).insert(TaskState::Cancelled);
            commands.trigger(TaskCancelledEvent {
                task: task_entity,
                ability_instance: ability_task.ability_instance,
                ability_spec: ability_task.ability_spec,
                owner: ability_task.owner,
            });
        }
    }
}
