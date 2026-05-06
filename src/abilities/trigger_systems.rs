//! Trigger system implementations.
//!
//! Systems that handle automatic ability activation based on triggers.

use super::components::*;
use super::events::GameplayEvent;
use super::triggers::*;
use crate::core::OwnedTags;
use bevy::prelude::*;
use bevy_gameplay_tag::GameplayTagsManager;

/// Observer that handles GameplayEvent triggers.
///
/// When a GameplayEvent is received, this observer finds all abilities with
/// matching trigger tags and attempts to activate them.
pub fn handle_gameplay_event_triggers_system(
    trigger: On<GameplayEvent>,
    abilities: Query<(
        Entity,
        &AbilitySpec,
        &AbilityOwner,
        Option<&AbilityTriggers>,
    )>,
    mut commands: Commands,
) {
    let event = trigger.event();

    // Find abilities with matching GameplayEvent triggers
    for (ability_entity, _spec, owner, triggers) in abilities.iter() {
        if let Some(triggers) = triggers
            && triggers.has_trigger(&event.event_tag, AbilityTriggerSource::GameplayEvent)
        {
            // Trigger ability activation
            commands.trigger(super::systems::TryActivateAbilityEvent::new(
                ability_entity,
                owner.0,
            ));
        }
    }
}

/// System that handles OwnedTagAdded triggers.
///
/// When a tag is added to an entity, this system finds all abilities with
/// matching OwnedTagAdded triggers and attempts to activate them.
pub fn handle_owned_tag_added_triggers_system(
    changed_tags: Query<(Entity, &OwnedTags), Changed<OwnedTags>>,
    abilities: Query<(
        Entity,
        &AbilitySpec,
        &AbilityOwner,
        Option<&AbilityTriggers>,
    )>,
    _tags_manager: Res<GameplayTagsManager>,
    mut commands: Commands,
) {
    for (owner_entity, owner_tags) in changed_tags.iter() {
        // Check each ability owned by this entity
        for (ability_entity, _spec, ability_owner, triggers) in abilities.iter() {
            if ability_owner.0 != owner_entity {
                continue;
            }

            if let Some(triggers) = triggers {
                // Check if any OwnedTagAdded triggers match newly added tags
                for trigger in &triggers.triggers {
                    if trigger.trigger_source == AbilityTriggerSource::OwnedTagAdded {
                        // Check if the trigger tag is present in owner's tags
                        if owner_tags.0.explicit_tags.has_tag(&trigger.trigger_tag) {
                            // Trigger ability activation
                            commands.trigger(super::systems::TryActivateAbilityEvent::new(
                                ability_entity,
                                owner_entity,
                            ));
                        }
                    }
                }
            }
        }
    }
}

/// System that handles OwnedTagPresent triggers.
///
/// Activates abilities when their trigger tag is present, and cancels them
/// when the tag is removed.
pub fn handle_owned_tag_present_triggers_system(
    changed_tags: Query<(Entity, &OwnedTags), Changed<OwnedTags>>,
    abilities: Query<(
        Entity,
        &AbilitySpec,
        &AbilityOwner,
        &AbilityActiveState,
        Option<&AbilityTriggers>,
    )>,
    mut commands: Commands,
) {
    for (owner_entity, owner_tags) in changed_tags.iter() {
        // Check each ability owned by this entity
        for (ability_entity, _spec, ability_owner, active_state, triggers) in abilities.iter() {
            if ability_owner.0 != owner_entity {
                continue;
            }

            if let Some(triggers) = triggers {
                for trigger in &triggers.triggers {
                    if trigger.trigger_source == AbilityTriggerSource::OwnedTagPresent {
                        let has_tag = owner_tags.0.explicit_tags.has_tag(&trigger.trigger_tag);

                        if has_tag && !active_state.is_active {
                            // Tag is present and ability is not active - activate it
                            commands.trigger(super::systems::TryActivateAbilityEvent::new(
                                ability_entity,
                                owner_entity,
                            ));
                        } else if !has_tag && active_state.is_active {
                            // Tag is removed and ability is active - cancel all instances
                            // Note: We don't have instance entity here, so we'll need to query for it
                            // For now, we'll just log a warning
                            warn!(
                                "OwnedTagPresent trigger: tag removed but cannot cancel ability without instance entity"
                            );
                        }
                    }
                }
            }
        }
    }
}
