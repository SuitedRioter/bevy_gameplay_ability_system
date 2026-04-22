//! Gameplay ability definitions.
//!
//! This module defines the structure of gameplay abilities and their properties.

use bevy::prelude::*;
use bevy_gameplay_tag::{GameplayTagContainer, GameplayTagsManager, gameplay_tag::GameplayTag};
use std::sync::Arc;
use string_cache::DefaultAtom as Atom;

use super::traits::AbilityBehavior;

/// Instancing policy for abilities.
///
/// Determines how ability instances are created and managed.
/// Follows UE GAS's instancing model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InstancingPolicy {
    /// No instance is created. Logic executes directly from the definition.
    /// - No per-activation state storage
    /// - Best performance for simple abilities
    /// - Cannot use instance-specific data
    /// - Example: Simple buff application, montage playback
    NonInstanced,

    /// One instance per actor. Reused across activations.
    /// - State persists between activations
    /// - Useful for abilities that need to track cumulative state
    /// - Example: Channeled abilities, combo counters
    InstancedPerActor,

    /// New instance created for each activation (default).
    /// - State exists only for the duration of activation
    /// - Most common pattern
    /// - Example: Most abilities (fireball, dash, etc.)
    InstancedPerExecution,
}

impl Default for InstancingPolicy {
    fn default() -> Self {
        Self::InstancedPerExecution
    }
}

/// Ability definition — pure configuration data stored in the AbilityRegistry.
///
/// Each ability type is described by one definition. When granted to a character,
/// an AbilitySpec entity is spawned referencing this definition by ID. When
/// activated, behavior depends on the instancing policy:
/// - NonInstanced: No spec instance, logic executes from definition
/// - InstancedPerActor: Reuses existing spec entity
/// - InstancedPerExecution: Spawns new spec instance entity
#[derive(Clone)]
pub struct AbilityDefinition {
    /// Unique identifier for this ability.
    pub id: Atom,
    /// Instancing policy for this ability.
    pub instancing_policy: InstancingPolicy,
    /// Effect ID to apply as costs when the ability is committed.
    pub cost_effect: Option<Atom>,
    /// Effect ID to apply as cooldown when the ability is committed.
    pub cooldown_effect: Option<Atom>,
    /// Tags describing this ability (used for cancel matching).
    pub ability_tags: GameplayTagContainer,
    /// Tags granted to the owner while this ability is active.
    pub activation_owned_tags: GameplayTagContainer,
    /// Tags required on the owner to activate this ability.
    pub activation_required_tags: GameplayTagContainer,
    /// Tags that block activation if present on the owner.
    pub activation_blocked_tags: GameplayTagContainer,
    /// Tags required on the source to activate this ability.
    pub source_required_tags: GameplayTagContainer,
    /// Tags that block activation if present on the source.
    pub source_blocked_tags: GameplayTagContainer,
    /// Tags required on the target to activate this ability.
    pub target_required_tags: GameplayTagContainer,
    /// Tags that block activation if present on the target.
    pub target_blocked_tags: GameplayTagContainer,
    /// Tags added to owner to block other abilities while active.
    pub block_abilities_with_tags: GameplayTagContainer,
    /// Tags to cancel when this ability activates.
    pub cancel_abilities_with_tags: GameplayTagContainer,
    /// Custom behavior implementation.
    pub behavior: Option<Arc<dyn AbilityBehavior>>,
    /// Whether instances of this ability block other abilities by default.
    pub default_blocks_other_abilities: bool,
    /// Whether instances of this ability are cancelable by default.
    pub default_is_cancelable: bool,
}

impl std::fmt::Debug for AbilityDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AbilityDefinition")
            .field("id", &self.id)
            .field("instancing_policy", &self.instancing_policy)
            .field("cost_effect", &self.cost_effect)
            .field("cooldown_effect", &self.cooldown_effect)
            .field("ability_tags", &self.ability_tags)
            .field("activation_owned_tags", &self.activation_owned_tags)
            .field("activation_required_tags", &self.activation_required_tags)
            .field("activation_blocked_tags", &self.activation_blocked_tags)
            .field("source_required_tags", &self.source_required_tags)
            .field("source_blocked_tags", &self.source_blocked_tags)
            .field("target_required_tags", &self.target_required_tags)
            .field("target_blocked_tags", &self.target_blocked_tags)
            .field("block_abilities_with_tags", &self.block_abilities_with_tags)
            .field(
                "cancel_abilities_with_tags",
                &self.cancel_abilities_with_tags,
            )
            .field("behavior", &self.behavior.as_ref().map(|_| "<behavior>"))
            .finish()
    }
}

impl PartialEq for AbilityDefinition {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for AbilityDefinition {}

impl AbilityDefinition {
    /// Creates a new ability definition.
    pub fn new(id: impl Into<Atom>) -> Self {
        Self {
            id: id.into(),
            instancing_policy: InstancingPolicy::default(),
            cost_effect: None,
            cooldown_effect: None,
            ability_tags: GameplayTagContainer::default(),
            activation_owned_tags: GameplayTagContainer::default(),
            activation_required_tags: GameplayTagContainer::default(),
            activation_blocked_tags: GameplayTagContainer::default(),
            source_required_tags: GameplayTagContainer::default(),
            source_blocked_tags: GameplayTagContainer::default(),
            target_required_tags: GameplayTagContainer::default(),
            target_blocked_tags: GameplayTagContainer::default(),
            block_abilities_with_tags: GameplayTagContainer::default(),
            cancel_abilities_with_tags: GameplayTagContainer::default(),
            behavior: None,
            default_blocks_other_abilities: true,
            default_is_cancelable: true,
        }
    }

    /// Sets the behavior implementation.
    pub fn with_behavior(mut self, behavior: Arc<dyn AbilityBehavior>) -> Self {
        self.behavior = Some(behavior);
        self
    }

    /// Sets the instancing policy.
    pub fn with_instancing_policy(mut self, policy: InstancingPolicy) -> Self {
        self.instancing_policy = policy;
        self
    }

    /// Adds a cost effect.
    pub fn with_cost_effect(mut self, effect_id: impl Into<Atom>) -> Self {
        self.cost_effect = Some(effect_id.into());
        self
    }

    /// Sets the cooldown effect.
    pub fn with_cooldown_effect(mut self, effect_id: impl Into<Atom>) -> Self {
        self.cooldown_effect = Some(effect_id.into());
        self
    }

    /// Sets whether instances block other abilities by default.
    pub fn with_blocks_other_abilities(mut self, blocks: bool) -> Self {
        self.default_blocks_other_abilities = blocks;
        self
    }

    /// Sets whether instances are cancelable by default.
    pub fn with_cancelable(mut self, cancelable: bool) -> Self {
        self.default_is_cancelable = cancelable;
        self
    }

    /// Adds a tag describing this ability.
    pub fn add_ability_tag(
        mut self,
        tag: GameplayTag,
        tags_manager: &Res<GameplayTagsManager>,
    ) -> Self {
        self.ability_tags.add_tag(tag, tags_manager);
        self
    }

    /// Adds an activation owned tag.
    pub fn add_activation_owned_tag(
        mut self,
        tag: GameplayTag,
        tags_manager: &Res<GameplayTagsManager>,
    ) -> Self {
        self.activation_owned_tags.add_tag(tag, tags_manager);
        self
    }

    /// Adds an activation required tag.
    pub fn add_activation_required_tag(
        mut self,
        tag: GameplayTag,
        tags_manager: &Res<GameplayTagsManager>,
    ) -> Self {
        self.activation_required_tags.add_tag(tag, tags_manager);
        self
    }

    /// Adds an activation blocked tag.
    pub fn add_activation_blocked_tag(
        mut self,
        tag: GameplayTag,
        tags_manager: &Res<GameplayTagsManager>,
    ) -> Self {
        self.activation_blocked_tags.add_tag(tag, tags_manager);
        self
    }

    /// Adds a source required tag.
    pub fn add_source_required_tag(
        mut self,
        tag: GameplayTag,
        tags_manager: &Res<GameplayTagsManager>,
    ) -> Self {
        self.source_required_tags.add_tag(tag, tags_manager);
        self
    }

    /// Adds a source blocked tag.
    pub fn add_source_blocked_tag(
        mut self,
        tag: GameplayTag,
        tags_manager: &Res<GameplayTagsManager>,
    ) -> Self {
        self.source_blocked_tags.add_tag(tag, tags_manager);
        self
    }

    /// Adds a target required tag.
    pub fn add_target_required_tag(
        mut self,
        tag: GameplayTag,
        tags_manager: &Res<GameplayTagsManager>,
    ) -> Self {
        self.target_required_tags.add_tag(tag, tags_manager);
        self
    }

    /// Adds a target blocked tag.
    pub fn add_target_blocked_tag(
        mut self,
        tag: GameplayTag,
        tags_manager: &Res<GameplayTagsManager>,
    ) -> Self {
        self.target_blocked_tags.add_tag(tag, tags_manager);
        self
    }

    /// Adds a tag that blocks other abilities while this one is active.
    pub fn add_block_abilities_with_tag(
        mut self,
        tag: GameplayTag,
        tags_manager: &Res<GameplayTagsManager>,
    ) -> Self {
        self.block_abilities_with_tags.add_tag(tag, tags_manager);
        self
    }

    /// Adds a tag that will cancel abilities when this ability activates.
    pub fn add_cancel_abilities_with_tag(
        mut self,
        tag: GameplayTag,
        tags_manager: &Res<GameplayTagsManager>,
    ) -> Self {
        self.cancel_abilities_with_tags.add_tag(tag, tags_manager);
        self
    }
}

/// Resource that stores all ability definitions.
#[derive(Resource, Default)]
pub struct AbilityRegistry {
    pub definitions: std::collections::HashMap<Atom, AbilityDefinition>,
}

impl AbilityRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, definition: AbilityDefinition) {
        self.definitions.insert(definition.id.clone(), definition);
    }

    pub fn get(&self, id: impl Into<Atom>) -> Option<&AbilityDefinition> {
        self.definitions.get(&id.into())
    }
}

#[cfg(test)]
mod tests {
    use bevy::ecs::system::RunSystemOnce;
    use bevy_gameplay_tag::GameplayTagsPlugin;

    use super::*;

    #[test]
    fn test_ability_definition_builder() {
        let mut app = App::new();
        app.add_plugins(GameplayTagsPlugin::with_data_path(
            "assets/gameplay_tags.json".to_string(),
        ));
        app.update();

        app.world_mut()
            .run_system_once(|tags_manager: Res<GameplayTagsManager>| {
                let ability = AbilityDefinition::new("test_ability")
                    .with_cost_effect("mana_cost")
                    .with_cooldown_effect("cooldown_5s")
                    .add_activation_required_tag(GameplayTag::new("State.Alive"), &tags_manager)
                    .add_activation_blocked_tag(GameplayTag::new("State.Stunned"), &tags_manager)
                    .add_ability_tag(GameplayTag::new("Ability.Casting"), &tags_manager)
                    .add_block_abilities_with_tag(
                        GameplayTag::new("Ability.Casting"),
                        &tags_manager,
                    );

                assert_eq!(ability.id, Atom::from("test_ability"));
                assert_eq!(ability.activation_required_tags.gameplay_tags.len(), 1);
                assert_eq!(ability.activation_blocked_tags.gameplay_tags.len(), 1);
                assert_eq!(ability.ability_tags.gameplay_tags.len(), 1);
                assert_eq!(ability.block_abilities_with_tags.gameplay_tags.len(), 1);
            })
            .expect("System should run successfully");
    }

    #[test]
    fn test_registry() {
        let mut registry = AbilityRegistry::new();
        let ability = AbilityDefinition::new("test");
        registry.register(ability);

        assert!(registry.get("test").is_some());
        assert!(registry.get("nonexistent").is_none());
    }
}
