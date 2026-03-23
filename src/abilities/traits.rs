//! Ability behavior traits.
//!
//! Defines the lifecycle hooks for custom ability implementations.

use crate::abilities::OnGameplayAbilityEnded;
use crate::core::{ApplyGameplayEffectEvent, OwnedTags};
use bevy::prelude::*;
use bevy_gameplay_tag::{GameplayTagContainer, GameplayTagsManager};

use crate::effects::GameplayEffectRegistry;
use crate::prelude::{AbilityDefinition, AbilityRegistry, AbilitySpec};

/// Reason why ability activation check failed.
#[derive(Debug, Clone, PartialEq)]
pub enum ActivationCheckFailure {
    /// Ability is on cooldown (contains the cooldown tags).
    OnCooldown(GameplayTagContainer),
    /// Source is missing required tags (contains the missing tags).
    SourceMissingRequiredTags(GameplayTagContainer),
    /// Source has blocked tags (contains the conflicting tags).
    SourceHasBlockedTags(GameplayTagContainer),
    /// Target is missing required tags (contains the missing tags).
    TargetMissingRequiredTags(GameplayTagContainer),
    /// Target has blocked tags (contains the conflicting tags).
    TargetHasBlockedTags(GameplayTagContainer),
    /// Missing required components or resources.
    MissingComponents,
}

/// Result type for activation checks.
pub type ActivationCheckResult = Result<(), ActivationCheckFailure>;

/// Ability behavior trait for custom ability logic.
///
/// The behavior is stored on the AbilityDefinition (in the registry) and
/// cloned (via Arc) onto each AbilitySpecInstance when activated.
///
/// Lifecycle: can_activate → pre_activate → activate → commit → end
pub trait AbilityBehavior: Send + Sync + 'static {
    /// Check if the ability can be activated.
    ///
    /// Called before any costs are applied.
    fn can_activate(
        &self,
        world: &World,
        ability_entity: Entity,
        source: Entity,
        tags_manager: &Res<GameplayTagsManager>,
    ) -> ActivationCheckResult {
        let Some(spec) = world.get::<AbilitySpec>(ability_entity) else {
            return Err(ActivationCheckFailure::MissingComponents);
        };

        let registry = world.resource::<AbilityRegistry>();
        let Some(definition) = registry.get(&spec.definition_id) else {
            return Err(ActivationCheckFailure::MissingComponents);
        };
        let effect_registry = world.resource::<GameplayEffectRegistry>();
        let Some(source_tags) = world.get::<OwnedTags>(source) else {
            return Err(ActivationCheckFailure::MissingComponents);
        };

        // Check cooldown
        if let Some(cd_id) = &definition.cooldown_effect
            && let Some(cd_def) = effect_registry.get(cd_id.as_ref())
            && source_tags
                .0
                .has_any_matching_gameplay_tags(&cd_def.granted_tags)
        {
            let mut cooldown_tags = GameplayTagContainer::default();
            cooldown_tags.append_matches_tags(
                &source_tags.0.explicit_tags,
                &cd_def.granted_tags,
                tags_manager,
            );
            return Err(ActivationCheckFailure::OnCooldown(cooldown_tags));
        }

        // Check source required tags
        if !definition.source_required_tags.is_empty()
            && !source_tags
                .0
                .has_all_matching_gameplay_tags(&definition.source_required_tags)
        {
            let mut missing_tags = GameplayTagContainer::default();
            missing_tags.append_matches_tags(
                &definition.source_required_tags,
                &source_tags.0.explicit_tags,
                tags_manager,
            );
            return Err(ActivationCheckFailure::SourceMissingRequiredTags(
                missing_tags,
            ));
        }

        // Check source blocked tags
        if source_tags
            .0
            .has_any_matching_gameplay_tags(&definition.source_blocked_tags)
        {
            let mut blocked_tags = GameplayTagContainer::default();
            blocked_tags.append_matches_tags(
                &source_tags.0.explicit_tags,
                &definition.source_blocked_tags,
                tags_manager,
            );
            return Err(ActivationCheckFailure::SourceHasBlockedTags(blocked_tags));
        }

        Ok(())
    }

    /// Called before activation begins. Runs with &mut World access.
    fn pre_activate(
        &self,
        _world: &mut World,
        _instance_entity: Entity,
        _spec_entity: Entity,
        _source: Entity,
    ) {
    }

    /// Called when the ability instance is activated. Main ability logic goes here.
    fn activate(
        &self,
        _world: &mut World,
        _instance_entity: Entity,
        _spec_entity: Entity,
        _source: Entity,
        _target: Option<Entity>,
    ) {
    }

    /// Check if the ability can be committed (cost and cooldown re-check).
    fn commit_check(
        &self,
        world: &World,
        definition: &AbilityDefinition,
        source: Entity,
        tags_manager: &Res<GameplayTagsManager>,
    ) -> ActivationCheckResult {
        let effect_registry = world.resource::<GameplayEffectRegistry>();
        let Some(source_tags) = world.get::<OwnedTags>(source) else {
            return Err(ActivationCheckFailure::MissingComponents);
        };

        // Check cooldown
        if let Some(cd_id) = &definition.cooldown_effect
            && let Some(cd_def) = effect_registry.get(cd_id.as_ref())
            && source_tags
                .0
                .has_any_matching_gameplay_tags(&cd_def.granted_tags)
        {
            let mut cooldown_tags = GameplayTagContainer::default();
            cooldown_tags.append_matches_tags(
                &source_tags.0.explicit_tags,
                &cd_def.granted_tags,
                tags_manager,
            );
            return Err(ActivationCheckFailure::OnCooldown(cooldown_tags));
        }

        Ok(())
    }

    /// Called when the ability is committed. Re-checks cost/cooldown, then executes.
    fn commit(
        &self,
        world: &World,
        commands: &mut Commands,
        definition: &AbilityDefinition,
        spec: &AbilitySpec,
        source: Entity,
        tags_manager: &Res<GameplayTagsManager>,
    ) -> ActivationCheckResult {
        self.commit_check(world, definition, source, tags_manager)?;
        self.commit_execute(commands, definition, spec, source);
        Ok(())
    }

    /// Execute commit logic (apply costs and cooldowns).
    fn commit_execute(
        &self,
        commands: &mut Commands,
        definition: &AbilityDefinition,
        spec: &AbilitySpec,
        source: Entity,
    ) {
        if let Some(cd_id) = &definition.cooldown_effect {
            commands.trigger(ApplyGameplayEffectEvent {
                effect_id: cd_id.clone(),
                target: source,
                instigator: Some(source),
                level: spec.level,
            });
        };

        if let Some(cost_id) = &definition.cost_effect {
            commands.trigger(ApplyGameplayEffectEvent {
                effect_id: cost_id.clone(),
                target: source,
                instigator: Some(source),
                level: spec.level,
            });
        }
    }

    /// Called when the ability instance ends. Cleanup logic goes here.
    fn end(&self, commands: &mut Commands, instance_entity: Entity, was_cancelled: bool) {
        commands.trigger(OnGameplayAbilityEnded {
            ability_instance: instance_entity,
            was_cancelled,
        });
    }
}

/// Default behavior (zero-sized type that uses trait defaults).
#[derive(Debug, Clone, Copy)]
pub struct DefaultAbilityBehavior;

impl AbilityBehavior for DefaultAbilityBehavior {}
