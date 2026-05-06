//! Ability trigger system.
//!
//! Defines how abilities can be automatically activated by external events.

use bevy::prelude::*;
use bevy_gameplay_tag::gameplay_tag::GameplayTag;

/// Trigger source type.
///
/// Defines what type of event will activate the ability.
/// Matches UE GAS's `EGameplayAbilityTriggerSource`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AbilityTriggerSource {
    /// Triggered by an external gameplay event.
    ///
    /// The ability will receive a GameplayEvent payload.
    GameplayEvent,

    /// Triggered when the owner gains the specified tag.
    ///
    /// Will not cancel when the tag is removed.
    OwnedTagAdded,

    /// Triggered when the owner has the specified tag.
    ///
    /// The ability will be canceled if the tag is later removed.
    OwnedTagPresent,
}

/// Trigger data for an ability.
///
/// Defines how an ability will be triggered by external events.
/// Matches UE GAS's `FAbilityTriggerData`.
#[derive(Debug, Clone, PartialEq)]
pub struct AbilityTriggerData {
    /// The tag to respond to.
    pub trigger_tag: GameplayTag,
    /// The type of trigger to respond to.
    pub trigger_source: AbilityTriggerSource,
}

impl AbilityTriggerData {
    /// Creates a new trigger data.
    pub fn new(trigger_tag: GameplayTag, trigger_source: AbilityTriggerSource) -> Self {
        Self {
            trigger_tag,
            trigger_source,
        }
    }

    /// Creates a gameplay event trigger.
    pub fn gameplay_event(trigger_tag: GameplayTag) -> Self {
        Self::new(trigger_tag, AbilityTriggerSource::GameplayEvent)
    }

    /// Creates an owned tag added trigger.
    pub fn owned_tag_added(trigger_tag: GameplayTag) -> Self {
        Self::new(trigger_tag, AbilityTriggerSource::OwnedTagAdded)
    }

    /// Creates an owned tag present trigger.
    pub fn owned_tag_present(trigger_tag: GameplayTag) -> Self {
        Self::new(trigger_tag, AbilityTriggerSource::OwnedTagPresent)
    }
}

/// Component that stores trigger data for an ability.
///
/// When present on an AbilitySpec entity, the ability can be automatically
/// activated by the specified triggers.
#[derive(Component, Debug, Clone)]
pub struct AbilityTriggers {
    pub triggers: Vec<AbilityTriggerData>,
}

impl AbilityTriggers {
    /// Creates a new empty trigger list.
    pub fn new() -> Self {
        Self {
            triggers: Vec::new(),
        }
    }

    /// Adds a trigger.
    pub fn add_trigger(mut self, trigger: AbilityTriggerData) -> Self {
        self.triggers.push(trigger);
        self
    }

    /// Checks if any trigger matches the given tag and source.
    pub fn has_trigger(&self, tag: &GameplayTag, source: AbilityTriggerSource) -> bool {
        self.triggers
            .iter()
            .any(|t| t.trigger_tag == *tag && t.trigger_source == source)
    }
}

impl Default for AbilityTriggers {
    fn default() -> Self {
        Self::new()
    }
}
