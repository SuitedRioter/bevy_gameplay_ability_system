//! Task system implementations for new task types.
//!
//! This module contains the update systems for the additional task types
//! that were added to match Unreal GAS functionality.

use super::tasks::*;
use crate::attributes::{AttributeData, AttributeName};
use crate::core::OwnedTags;
use bevy::ecs::relationship::Relationship;
use bevy::prelude::*;

/// System that updates WaitGameplayTagAdded tasks.
pub fn update_wait_tag_added_tasks_system(
    mut commands: Commands,
    mut tasks: Query<(
        Entity,
        &AbilityTask,
        &mut WaitGameplayTagAddedTask,
        &mut TaskState,
    )>,
    owner_tags: Query<&OwnedTags>,
) {
    for (task_entity, ability_task, mut task, mut state) in tasks.iter_mut() {
        if *state != TaskState::Running || task.triggered {
            continue;
        }

        // Check if owner has the tag
        if let Ok(tags) = owner_tags.get(ability_task.owner)
            && tags.0.explicit_tags.has_tag(&task.tag)
        {
            task.triggered = true;
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

/// System that updates WaitGameplayTagRemoved tasks.
pub fn update_wait_tag_removed_tasks_system(
    mut commands: Commands,
    mut tasks: Query<(
        Entity,
        &AbilityTask,
        &mut WaitGameplayTagRemovedTask,
        &mut TaskState,
    )>,
    owner_tags: Query<&OwnedTags>,
) {
    for (task_entity, ability_task, mut task, mut state) in tasks.iter_mut() {
        if *state != TaskState::Running || task.triggered {
            continue;
        }

        // Check if owner no longer has the tag
        if let Ok(tags) = owner_tags.get(ability_task.owner)
            && !tags.0.explicit_tags.has_tag(&task.tag)
        {
            task.triggered = true;
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

/// Observer that handles ability activation events for WaitAbilityActivate tasks.
pub fn on_ability_activated_for_wait_tasks(
    ev: On<crate::abilities::systems::AbilityActivatedEvent>,
    mut commands: Commands,
    mut tasks: Query<(
        Entity,
        &AbilityTask,
        &mut WaitAbilityActivateTask,
        &mut TaskState,
    )>,
    ability_specs: Query<&crate::abilities::components::AbilitySpec>,
) {
    let event = ev.event();
    for (task_entity, ability_task, mut task, mut state) in tasks.iter_mut() {
        if *state != TaskState::Running || task.triggered {
            continue;
        }

        // Check if this is the owner's ability activation
        if event.owner != ability_task.owner {
            continue;
        }

        // Check if definition ID matches (if specified)
        if let Some(ref target_id) = task.ability_definition_id {
            // Get the ability spec to check definition ID
            if let Ok(spec) = ability_specs.get(event.ability_spec) {
                if spec.definition_id.as_ref() != target_id {
                    continue;
                }
            } else {
                continue;
            }
        }

        task.triggered = true;
        *state = TaskState::Completed;
        commands.trigger(TaskCompletedEvent {
            task: task_entity,
            ability_instance: ability_task.ability_instance,
            ability_spec: ability_task.ability_spec,
            owner: ability_task.owner,
        });
    }
}

/// Observer that handles ability end events for WaitAbilityEnd tasks.
pub fn on_ability_ended_for_wait_tasks(
    ev: On<crate::abilities::systems::OnGameplayAbilityEnded>,
    mut commands: Commands,
    mut tasks: Query<(
        Entity,
        &AbilityTask,
        &mut WaitAbilityEndTask,
        &mut TaskState,
    )>,
    ability_specs: Query<&crate::abilities::components::AbilitySpec>,
) {
    let event = ev.event();

    // Get the ability instance entity from the event target
    let ability_instance = event.ability_instance;

    for (task_entity, ability_task, mut task, mut state) in tasks.iter_mut() {
        if *state != TaskState::Running || task.triggered {
            continue;
        }

        // Check if this is the owner's ability ending
        // We need to match by ability_spec since the event doesn't have owner
        if ability_task.ability_instance != Some(ability_instance) {
            continue;
        }

        // Check if definition ID matches (if specified)
        if let Some(ref target_id) = task.ability_definition_id {
            // Get the ability spec to check definition ID
            if let Ok(spec) = ability_specs.get(ability_task.ability_spec) {
                if spec.definition_id.as_ref() != target_id {
                    continue;
                }
            } else {
                continue;
            }
        }

        task.triggered = true;
        *state = TaskState::Completed;
        commands.trigger(TaskCompletedEvent {
            task: task_entity,
            ability_instance: ability_task.ability_instance,
            ability_spec: ability_task.ability_spec,
            owner: ability_task.owner,
        });
    }
}

/// System that updates WaitAttributeChangeRatio tasks.
pub fn update_wait_attribute_ratio_tasks_system(
    mut commands: Commands,
    mut tasks: Query<(
        Entity,
        &AbilityTask,
        &mut WaitAttributeChangeRatioTask,
        &mut TaskState,
    )>,
    attributes: Query<(&AttributeData, &AttributeName, &ChildOf)>,
) {
    for (task_entity, ability_task, mut task, mut state) in tasks.iter_mut() {
        if *state != TaskState::Running || task.triggered {
            continue;
        }

        // Find numerator and denominator attributes
        let numerator = attributes
            .iter()
            .find(|(_, name, child_of)| {
                child_of.get() == ability_task.owner && name.as_str() == task.numerator_attribute
            })
            .map(|(data, _, _)| data.current_value);

        let denominator = attributes
            .iter()
            .find(|(_, name, child_of)| {
                child_of.get() == ability_task.owner && name.as_str() == task.denominator_attribute
            })
            .map(|(data, _, _)| data.current_value);

        if let (Some(num), Some(denom)) = (numerator, denominator) {
            if denom.abs() < f32::EPSILON {
                continue; // Avoid division by zero
            }

            let ratio = num / denom;
            if task.comparison.check(ratio, task.threshold) {
                task.triggered = true;
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
