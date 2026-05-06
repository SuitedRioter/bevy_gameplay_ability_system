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
    pub fn new(
        attribute_name: impl Into<String>,
        comparison: AttributeComparison,
        threshold: f32,
    ) -> Self {
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

/// WaitTargetData task - waits for target data to be provided.
///
/// This task is used for abilities that require player input to select targets,
/// such as skill shots, area-of-effect abilities, or targeted spells.
///
/// # Example
/// ```ignore
/// // Spawn a WaitTargetData task
/// commands.spawn((
///     AbilityTask { ability_instance, ability_spec, owner },
///     WaitTargetDataTask::new(),
///     TaskState::Running,
/// ));
///
/// // Later, provide target data via event
/// commands.trigger(ProvideTargetDataEvent {
///     task: task_entity,
///     target_data: GameplayAbilityTargetData::from_actor(target_entity),
/// });
/// ```
#[derive(Component, Debug, Clone)]
pub struct WaitTargetDataTask {
    /// The target data once provided (None = waiting).
    pub target_data: Option<crate::abilities::GameplayAbilityTargetData>,
    /// Whether to trigger only once.
    pub only_trigger_once: bool,
    /// Whether the target data has been received.
    pub data_received: bool,
}

impl WaitTargetDataTask {
    /// Create a new wait target data task.
    pub fn new() -> Self {
        Self {
            target_data: None,
            only_trigger_once: true,
            data_received: false,
        }
    }

    /// Set whether to trigger only once.
    pub fn with_only_trigger_once(mut self, only_once: bool) -> Self {
        self.only_trigger_once = only_once;
        self
    }

    /// Provide target data to complete the task (legacy method).
    pub fn provide_target_data(
        &mut self,
        target_data: crate::abilities::GameplayAbilityTargetData,
    ) {
        self.target_data = Some(target_data);
        self.data_received = true;
    }
}

impl Default for WaitTargetDataTask {
    fn default() -> Self {
        Self::new()
    }
}

/// Event for providing target data to a WaitTargetData task.
///
/// This event is triggered by external systems (e.g., UI, input handlers)
/// when the player has selected a target.
#[derive(Event, Debug, Clone)]
pub struct ProvideTargetDataEvent {
    /// The task entity waiting for target data.
    pub task: Entity,
    /// The target data to provide.
    pub target_data: super::target_data::GameplayAbilityTargetData,
}

/// Event for cancelling a WaitTargetData task.
///
/// This event is triggered when the player cancels target selection
/// (e.g., pressing ESC or right-clicking).
#[derive(Event, Debug, Clone)]
pub struct CancelTargetDataEvent {
    /// The task entity to cancel.
    pub task: Entity,
}

/// Input action for WaitInputPress task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputAction {
    /// Primary action button (e.g., left mouse, gamepad A).
    Confirm,
    /// Secondary action button (e.g., right mouse, gamepad B).
    Cancel,
    /// Custom action identified by index.
    Custom(u32),
}

/// WaitInputPress task - waits for a specific input action.
///
/// Completes when the specified input action is pressed.
#[derive(Component, Debug, Clone)]
pub struct WaitInputPressTask {
    /// The input action to wait for.
    pub action: InputAction,
    /// Whether the input has been pressed.
    pub pressed: bool,
}

impl WaitInputPressTask {
    /// Create a new wait input press task.
    pub fn new(action: InputAction) -> Self {
        Self {
            action,
            pressed: false,
        }
    }

    /// Create a task waiting for confirm input.
    pub fn confirm() -> Self {
        Self::new(InputAction::Confirm)
    }

    /// Create a task waiting for cancel input.
    pub fn cancel() -> Self {
        Self::new(InputAction::Cancel)
    }
}

/// Event sent when an input action is pressed.
///
/// User code should send this event when input is detected.
#[derive(Event, Debug, Clone, Copy)]
pub struct InputPressedEvent {
    /// The entity that triggered the input (usually the player character).
    pub entity: Entity,
    /// The input action that was pressed.
    pub action: InputAction,
}

/// WaitOverlap task - waits for collision overlap with entities matching a filter.
///
/// Completes when the owner overlaps with an entity that has the specified component.
/// This is a simplified version - real collision detection would integrate with
/// a physics engine like bevy_rapier or avian.
#[derive(Component, Debug, Clone)]
pub struct WaitOverlapTask {
    /// Type name of the component to filter by (e.g., "Enemy", "Projectile").
    /// Empty string = any entity.
    pub filter_component: String,
    /// Whether to trigger only once.
    pub only_trigger_once: bool,
    /// The overlapping entity once detected.
    pub overlapping_entity: Option<Entity>,
}

impl WaitOverlapTask {
    /// Create a new wait overlap task for any entity.
    pub fn new() -> Self {
        Self {
            filter_component: String::new(),
            only_trigger_once: true,
            overlapping_entity: None,
        }
    }

    /// Create a task that waits for overlap with entities having a specific component.
    pub fn with_filter(filter_component: impl Into<String>) -> Self {
        Self {
            filter_component: filter_component.into(),
            only_trigger_once: true,
            overlapping_entity: None,
        }
    }

    /// Set whether to trigger only once.
    pub fn with_only_trigger_once(mut self, only_once: bool) -> Self {
        self.only_trigger_once = only_once;
        self
    }
}

impl Default for WaitOverlapTask {
    fn default() -> Self {
        Self::new()
    }
}

/// Event sent when two entities overlap.
///
/// User code should send this event from collision detection systems.
#[derive(Event, Debug, Clone, Copy)]
pub struct OverlapEvent {
    /// The first entity in the overlap.
    pub entity_a: Entity,
    /// The second entity in the overlap.
    pub entity_b: Entity,
    /// Optional component type name for filtering (e.g., "Enemy").
    pub component_type: Option<&'static str>,
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
        (
            Entity,
            &crate::attributes::AttributeData,
            &crate::attributes::AttributeName,
        ),
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

/// System that checks WaitTargetData tasks for completion.
pub fn check_wait_target_data_tasks_system(
    mut commands: Commands,
    mut tasks: Query<(
        Entity,
        &AbilityTask,
        &mut WaitTargetDataTask,
        &mut TaskState,
    )>,
) {
    for (task_entity, ability_task, mut wait_target, mut state) in tasks.iter_mut() {
        if *state != TaskState::Running {
            continue;
        }

        // Check if target data has been provided
        if wait_target.data_received && wait_target.target_data.is_some() {
            *state = TaskState::Completed;
            commands.trigger(TaskCompletedEvent {
                task: task_entity,
                ability_instance: ability_task.ability_instance,
                ability_spec: ability_task.ability_spec,
                owner: ability_task.owner,
            });

            // Reset for next trigger if not only_trigger_once
            if !wait_target.only_trigger_once {
                wait_target.data_received = false;
                wait_target.target_data = None;
                *state = TaskState::Running;
            }
        }
    }
}

/// Observer that handles target data provision for WaitTargetData tasks.
pub fn handle_provide_target_data_system(
    trigger: On<ProvideTargetDataEvent>,
    mut commands: Commands,
    mut tasks: Query<(
        Entity,
        &AbilityTask,
        &mut WaitTargetDataTask,
        &mut TaskState,
    )>,
) {
    let event = trigger.event();

    let Ok((task_entity, ability_task, mut wait_target, mut state)) = tasks.get_mut(event.task)
    else {
        return;
    };

    if *state != TaskState::Running {
        return;
    }

    // Provide target data
    wait_target.target_data = Some(event.target_data.clone());
    wait_target.data_received = true;

    // Complete immediately
    *state = TaskState::Completed;
    commands.trigger(TaskCompletedEvent {
        task: task_entity,
        ability_instance: ability_task.ability_instance,
        ability_spec: ability_task.ability_spec,
        owner: ability_task.owner,
    });
}

/// Observer that handles target data cancellation for WaitTargetData tasks.
pub fn handle_cancel_target_data_system(
    trigger: On<CancelTargetDataEvent>,
    mut tasks: Query<(&mut WaitTargetDataTask, &mut TaskState)>,
) {
    let event = trigger.event();

    let Ok((mut wait_target, mut state)) = tasks.get_mut(event.task) else {
        return;
    };

    if *state != TaskState::Running {
        return;
    }

    // Cancel the task
    wait_target.data_received = false;
    wait_target.target_data = None;
    *state = TaskState::Cancelled;
}

/// Observer that handles input pressed events for WaitInputPress tasks.
pub fn handle_input_pressed_for_tasks_system(
    trigger: On<InputPressedEvent>,
    mut commands: Commands,
    mut tasks: Query<(
        Entity,
        &AbilityTask,
        &mut WaitInputPressTask,
        &mut TaskState,
    )>,
) {
    let event = trigger.event();

    for (task_entity, ability_task, mut wait_input, mut state) in tasks.iter_mut() {
        if *state != TaskState::Running || wait_input.pressed {
            continue;
        }

        // Check if this task belongs to the entity that triggered the input
        if ability_task.owner != event.entity {
            continue;
        }

        // Check if the input action matches
        if wait_input.action != event.action {
            continue;
        }

        wait_input.pressed = true;
        *state = TaskState::Completed;
        commands.trigger(TaskCompletedEvent {
            task: task_entity,
            ability_instance: ability_task.ability_instance,
            ability_spec: ability_task.ability_spec,
            owner: ability_task.owner,
        });
    }
}

/// Observer that handles overlap events for WaitOverlap tasks.
pub fn handle_overlap_for_tasks_system(
    trigger: On<OverlapEvent>,
    mut commands: Commands,
    mut tasks: Query<(Entity, &AbilityTask, &mut WaitOverlapTask, &mut TaskState)>,
) {
    let event = trigger.event();

    for (task_entity, ability_task, mut wait_overlap, mut state) in tasks.iter_mut() {
        if *state != TaskState::Running {
            continue;
        }

        if wait_overlap.only_trigger_once && wait_overlap.overlapping_entity.is_some() {
            continue;
        }

        // Check if the owner is involved in the overlap
        let other_entity = if ability_task.owner == event.entity_a {
            Some(event.entity_b)
        } else if ability_task.owner == event.entity_b {
            Some(event.entity_a)
        } else {
            None
        };

        let Some(other) = other_entity else {
            continue;
        };

        // Check component filter if specified
        if !wait_overlap.filter_component.is_empty() {
            if let Some(component_type) = event.component_type {
                if component_type != wait_overlap.filter_component {
                    continue;
                }
            } else {
                // Filter specified but event has no component type
                continue;
            }
        }

        wait_overlap.overlapping_entity = Some(other);

        if wait_overlap.only_trigger_once {
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

/// PlayMontageAndWait task - plays an animation and waits for it to complete.
///
/// This task is used for abilities that need to play animations (montages) and
/// wait for them to finish before continuing. This is essential for skill animations,
/// attack sequences, and other animated abilities.
///
/// # Example
/// ```ignore
/// // Spawn a PlayMontageAndWait task
/// commands.spawn((
///     AbilityTask { ability_instance, ability_spec, owner },
///     PlayMontageAndWaitTask::new("attack_montage", 1.0),
///     TaskState::Running,
/// ));
///
/// // The task will complete when the animation finishes
/// // or can be cancelled via CancelMontageEvent
/// ```
#[derive(Component, Debug, Clone)]
pub struct PlayMontageAndWaitTask {
    /// The name/ID of the montage to play.
    pub montage_name: String,
    /// The playback rate (1.0 = normal speed).
    pub play_rate: f32,
    /// The section name to start from (None = start from beginning).
    pub start_section: Option<String>,
    /// Whether to stop the montage when the task is cancelled.
    pub stop_on_cancel: bool,
    /// The entity playing the animation (usually the owner).
    pub animation_entity: Option<Entity>,
    /// Elapsed time since the montage started.
    pub elapsed_time: f32,
    /// Total duration of the montage.
    pub duration: f32,
    /// Whether the montage has started playing.
    pub started: bool,
}

impl PlayMontageAndWaitTask {
    /// Create a new play montage and wait task.
    pub fn new(montage_name: impl Into<String>, duration: f32) -> Self {
        Self {
            montage_name: montage_name.into(),
            play_rate: 1.0,
            start_section: None,
            stop_on_cancel: true,
            animation_entity: None,
            elapsed_time: 0.0,
            duration,
            started: false,
        }
    }

    /// Set the playback rate.
    pub fn with_play_rate(mut self, play_rate: f32) -> Self {
        self.play_rate = play_rate;
        self
    }

    /// Set the start section.
    pub fn with_start_section(mut self, section: impl Into<String>) -> Self {
        self.start_section = Some(section.into());
        self
    }

    /// Set whether to stop the montage when cancelled.
    pub fn with_stop_on_cancel(mut self, stop: bool) -> Self {
        self.stop_on_cancel = stop;
        self
    }

    /// Set the animation entity.
    pub fn with_animation_entity(mut self, entity: Entity) -> Self {
        self.animation_entity = Some(entity);
        self
    }
}

/// Event for starting a montage playback.
#[derive(Event, Debug, Clone)]
pub struct StartMontageEvent {
    /// The task entity that requested the montage.
    pub task: Entity,
    /// The entity that should play the animation.
    pub animation_entity: Entity,
    /// The montage name to play.
    pub montage_name: String,
    /// The playback rate.
    pub play_rate: f32,
    /// The section to start from.
    pub start_section: Option<String>,
}

/// Event for cancelling a montage playback.
#[derive(Event, Debug, Clone)]
pub struct CancelMontageEvent {
    /// The task entity to cancel.
    pub task: Entity,
    /// Whether to blend out the animation.
    pub blend_out: bool,
}

/// System that updates PlayMontageAndWait tasks.
pub fn update_play_montage_tasks_system(
    mut commands: Commands,
    mut tasks: Query<(
        Entity,
        &AbilityTask,
        &mut PlayMontageAndWaitTask,
        &mut TaskState,
    )>,
    time: Res<Time>,
) {
    for (task_entity, ability_task, mut montage_task, mut state) in tasks.iter_mut() {
        if *state != TaskState::Running {
            continue;
        }

        // Start the montage on first update
        if !montage_task.started {
            let animation_entity = montage_task.animation_entity.unwrap_or(ability_task.owner);

            commands.trigger(StartMontageEvent {
                task: task_entity,
                animation_entity,
                montage_name: montage_task.montage_name.clone(),
                play_rate: montage_task.play_rate,
                start_section: montage_task.start_section.clone(),
            });

            montage_task.started = true;
        }

        // Update elapsed time
        montage_task.elapsed_time += time.delta_secs() * montage_task.play_rate;

        // Check if montage has finished
        if montage_task.elapsed_time >= montage_task.duration {
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

/// System that handles montage cancellation.
pub fn handle_cancel_montage_system(
    trigger: On<CancelMontageEvent>,
    mut tasks: Query<(&mut PlayMontageAndWaitTask, &mut TaskState)>,
) {
    let event = trigger.event();

    let Ok((_montage_task, mut state)) = tasks.get_mut(event.task) else {
        return;
    };

    if *state != TaskState::Running {
        return;
    }

    *state = TaskState::Cancelled;

    // Note: The actual animation stopping should be handled by the animation system
    // listening to this event. This task only manages the state.
}

/// SpawnActor task - spawns an entity at a specified location.
///
/// This task is used for abilities that need to spawn entities such as
/// projectiles, summons, traps, or other game objects.
///
/// # Example
/// ```ignore
/// // Spawn a projectile
/// commands.spawn((
///     AbilityTask { ability_instance, ability_spec, owner },
///     SpawnActorTask::new(projectile_template, Vec3::new(0.0, 0.0, 0.0))
///         .with_rotation(Quat::from_rotation_y(1.57)),
///     TaskState::Running,
/// ));
/// ```
#[derive(Component, Debug, Clone)]
pub struct SpawnActorTask {
    /// The entity template to spawn (can be a prefab or bundle)
    pub template: Option<Entity>,
    /// Spawn location in world space
    pub location: Vec3,
    /// Spawn rotation
    pub rotation: Quat,
    /// The spawned entity (once created)
    pub spawned_entity: Option<Entity>,
    /// Whether the entity has been spawned
    pub spawned: bool,
    /// Optional bundle components to add to the spawned entity
    pub bundle_name: Option<String>,
}

impl SpawnActorTask {
    /// Create a new spawn actor task with a template entity.
    pub fn new(template: Entity, location: Vec3) -> Self {
        Self {
            template: Some(template),
            location,
            rotation: Quat::IDENTITY,
            spawned_entity: None,
            spawned: false,
            bundle_name: None,
        }
    }

    /// Create a new spawn actor task with a bundle name.
    pub fn with_bundle(bundle_name: impl Into<String>, location: Vec3) -> Self {
        Self {
            template: None,
            location,
            rotation: Quat::IDENTITY,
            spawned_entity: None,
            spawned: false,
            bundle_name: Some(bundle_name.into()),
        }
    }

    /// Set the spawn rotation.
    pub fn with_rotation(mut self, rotation: Quat) -> Self {
        self.rotation = rotation;
        self
    }

    /// Set the spawn location.
    pub fn with_location(mut self, location: Vec3) -> Self {
        self.location = location;
        self
    }
}

/// Event triggered when an actor is spawned by a SpawnActor task.
#[derive(Event, Debug, Clone)]
pub struct ActorSpawnedEvent {
    /// The task entity that spawned the actor.
    pub task: Entity,
    /// The spawned entity.
    pub spawned_entity: Entity,
    /// The ability instance that owns this task.
    pub ability_instance: Option<Entity>,
    /// The ability spec entity.
    pub ability_spec: Entity,
    /// The owner entity (character).
    pub owner: Entity,
}

/// System that spawns entities for SpawnActor tasks.
pub fn spawn_actor_tasks_system(
    mut commands: Commands,
    mut tasks: Query<(Entity, &AbilityTask, &mut SpawnActorTask, &mut TaskState)>,
) {
    for (task_entity, ability_task, mut spawn_task, mut state) in tasks.iter_mut() {
        if *state != TaskState::Running || spawn_task.spawned {
            continue;
        }

        // Spawn the entity
        let spawned = commands
            .spawn((
                Transform::from_translation(spawn_task.location).with_rotation(spawn_task.rotation),
                GlobalTransform::default(),
            ))
            .id();

        spawn_task.spawned_entity = Some(spawned);
        spawn_task.spawned = true;
        *state = TaskState::Completed;

        // Trigger spawned event
        commands.trigger(ActorSpawnedEvent {
            task: task_entity,
            spawned_entity: spawned,
            ability_instance: ability_task.ability_instance,
            ability_spec: ability_task.ability_spec,
            owner: ability_task.owner,
        });

        // Trigger task completed event
        commands.trigger(TaskCompletedEvent {
            task: task_entity,
            ability_instance: ability_task.ability_instance,
            ability_spec: ability_task.ability_spec,
            owner: ability_task.owner,
        });
    }
}

/// Repeat task - repeats an action multiple times with a delay.
///
/// This task is used for abilities that need to execute repeatedly,
/// such as channeled abilities, multi-hit attacks, or periodic effects.
///
/// # Example
/// ```ignore
/// // Repeat 5 times with 1 second delay
/// commands.spawn((
///     AbilityTask { ability_instance, ability_spec, owner },
///     RepeatTask::times(5, 1.0),
///     TaskState::Running,
/// ));
///
/// // Infinite repeat with 0.5 second delay
/// commands.spawn((
///     AbilityTask { ability_instance, ability_spec, owner },
///     RepeatTask::infinite(0.5),
///     TaskState::Running,
/// ));
/// ```
#[derive(Component, Debug, Clone)]
pub struct RepeatTask {
    /// Number of times to repeat (None = infinite)
    pub repeat_count: Option<u32>,
    /// Current iteration (starts at 0)
    pub current_iteration: u32,
    /// Delay between iterations (seconds)
    pub delay: f32,
    /// Elapsed time since last iteration
    pub elapsed: f32,
}

impl RepeatTask {
    /// Create a new repeat task with a specific count.
    pub fn times(count: u32, delay: f32) -> Self {
        Self {
            repeat_count: Some(count),
            current_iteration: 0,
            delay,
            elapsed: 0.0,
        }
    }

    /// Create a new infinite repeat task.
    pub fn infinite(delay: f32) -> Self {
        Self {
            repeat_count: None,
            current_iteration: 0,
            delay,
            elapsed: 0.0,
        }
    }

    /// Check if the task should continue repeating.
    pub fn should_continue(&self) -> bool {
        match self.repeat_count {
            Some(max) => self.current_iteration < max,
            None => true,
        }
    }
}

/// Event triggered on each iteration of a Repeat task.
#[derive(Event, Debug, Clone)]
pub struct TaskIterationEvent {
    /// The task entity.
    pub task: Entity,
    /// Current iteration number (starts at 1).
    pub iteration: u32,
    /// The ability instance that owns this task.
    pub ability_instance: Option<Entity>,
    /// The ability spec entity.
    pub ability_spec: Entity,
    /// The owner entity (character).
    pub owner: Entity,
}

/// System that updates Repeat tasks.
pub fn update_repeat_tasks_system(
    mut commands: Commands,
    mut tasks: Query<(Entity, &AbilityTask, &mut RepeatTask, &mut TaskState)>,
    time: Res<Time>,
) {
    for (task_entity, ability_task, mut repeat_task, mut state) in tasks.iter_mut() {
        if *state != TaskState::Running {
            continue;
        }

        repeat_task.elapsed += time.delta_secs();

        if repeat_task.elapsed >= repeat_task.delay {
            repeat_task.elapsed = 0.0;
            repeat_task.current_iteration += 1;

            // Trigger iteration event
            commands.trigger(TaskIterationEvent {
                task: task_entity,
                iteration: repeat_task.current_iteration,
                ability_instance: ability_task.ability_instance,
                ability_spec: ability_task.ability_spec,
                owner: ability_task.owner,
            });

            // Check if we should complete
            if !repeat_task.should_continue() {
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
