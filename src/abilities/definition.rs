//! Gameplay ability definitions.
//!
//! This module defines the structure of gameplay abilities and their properties.

use bevy::prelude::*;
use bevy_gameplay_tag::{GameplayTagContainer, GameplayTagsManager, gameplay_tag::GameplayTag};
use string_cache::DefaultAtom as Atom;

/// Instancing policy for abilities.
///
/// Determines how ability instances are created and managed.
/// ECS no need this.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[warn(dead_code)]
pub enum InstancingPolicy {
    /// Ability is not instanced. The spec itself is used for activation.
    NonInstanced,
    /// A new instance is created per execution.
    InstancedPerExecution,
    /// A single instance is created per actor.
    InstancedPerActor,
}

/// Net execution policy for abilities.
///
/// Determines where the ability executes in a networked environment.
/// For single-player games, this is mostly informational.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NetExecutionPolicy {
    /// Execute on local client only.
    LocalOnly,
    /// Execute on server only.
    ServerOnly,
    /// Execute on both client and server.
    LocalPredicted,
}

/// Ability definition.
///
/// This is the template for creating ability instances.
/// Store these in a resource or asset system.
#[derive(Debug, Clone, PartialEq)]
pub struct AbilityDefinition {
    /// Unique identifier for this ability.
    pub id: Atom,
    /// Net execution policy (for future networking support).
    pub net_execution_policy: NetExecutionPolicy,
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
}

impl AbilityDefinition {
    /// Creates a new ability definition.
    pub fn new(id: impl Into<Atom>) -> Self {
        Self {
            id: id.into(),
            net_execution_policy: NetExecutionPolicy::LocalOnly,
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
        }
    }

    /// Sets the net execution policy.
    pub fn with_net_execution_policy(mut self, policy: NetExecutionPolicy) -> Self {
        self.net_execution_policy = policy;
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
    /// Creates a new empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers an ability definition.
    pub fn register(&mut self, definition: AbilityDefinition) {
        self.definitions.insert(definition.id.clone(), definition);
    }

    /// Gets an ability definition by ID.
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
