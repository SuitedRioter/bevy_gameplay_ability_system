//! Ability behavior traits.
//!
//! Defines the lifecycle hooks for custom ability implementations.

use bevy::prelude::*;
use bevy_gameplay_tag::gameplay_tag_count_container::GameplayTagCountContainer;
use bevy_gameplay_tag::{GameplayTagContainer, GameplayTagsManager};

use crate::effects::GameplayEffectRegistry;
use crate::prelude::{AbilityRegistry, AbilitySpec};

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
/// Implement this trait to define custom behavior for abilities.
/// All methods have default implementations that do nothing.
pub trait AbilityBehavior: Send + Sync + 'static {
    /// Check if the ability can be activated.
    ///
    /// Called before any costs are applied. Return Err with details if activation should be prevented.
    fn can_activate(
        &self,
        world: &World,
        ability_entity: Entity,
        source: Entity,
        target: Option<Entity>,
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
        let Some(source_tags) = world.get::<GameplayTagCountContainer>(source) else {
            return Err(ActivationCheckFailure::MissingComponents);
        };

        // Check cooldown
        if let Some(cd_id) = &definition.cooldown_effect
            && let Some(cd_def) = effect_registry.get(cd_id.as_ref())
            && source_tags.has_any_matching_gameplay_tags(&cd_def.granted_tags)
        {
            let mut cooldown_tags = GameplayTagContainer::default();
            cooldown_tags.append_matches_tags(
                &source_tags.explicit_tags,
                &cd_def.granted_tags,
                tags_manager,
            );
            return Err(ActivationCheckFailure::OnCooldown(cooldown_tags));
        }

        // TODO: Check cost when attribute system is ready

        // Check source required tags
        if !definition.source_required_tags.is_empty()
            && !source_tags.has_all_matching_gameplay_tags(&definition.source_required_tags)
        {
            let mut missing_tags = GameplayTagContainer::default();
            missing_tags.append_matches_tags(
                &definition.source_required_tags,
                &source_tags.explicit_tags,
                tags_manager,
            );
            return Err(ActivationCheckFailure::SourceMissingRequiredTags(
                missing_tags,
            ));
        }

        // Check source blocked tags
        if source_tags.has_any_matching_gameplay_tags(&definition.source_blocked_tags) {
            let mut blocked_tags = GameplayTagContainer::default();
            blocked_tags.append_matches_tags(
                &source_tags.explicit_tags,
                &definition.source_blocked_tags,
                tags_manager,
            );
            return Err(ActivationCheckFailure::SourceHasBlockedTags(blocked_tags));
        }

        // Check target tags
        if let Some(target_entity) = target
            && let Some(target_tags) = world.get::<GameplayTagCountContainer>(target_entity)
        {
            // Check target required tags
            if !definition.target_required_tags.is_empty()
                && !target_tags.has_all_matching_gameplay_tags(&definition.target_required_tags)
            {
                let mut missing_tags = GameplayTagContainer::default();
                missing_tags.append_matches_tags(
                    &definition.target_required_tags,
                    &target_tags.explicit_tags,
                    tags_manager,
                );
                return Err(ActivationCheckFailure::TargetMissingRequiredTags(
                    missing_tags,
                ));
            }

            // Check target blocked tags
            if target_tags.has_any_matching_gameplay_tags(&definition.target_blocked_tags) {
                let mut blocked_tags = GameplayTagContainer::default();
                blocked_tags.append_matches_tags(
                    &target_tags.explicit_tags,
                    &definition.target_blocked_tags,
                    tags_manager,
                );
                return Err(ActivationCheckFailure::TargetHasBlockedTags(blocked_tags));
            }
        }

        Ok(())
    }

    /// Called before activation begins.
    ///
    /// Use this for setup logic before the ability enters the Activating state.
    fn pre_activate(
        &self,
        world: &mut World,
        ability_entity: Entity,
        _source: Entity,
        _target: Option<Entity>,
    ) {
        if let Some(mut spec) =
            world.get_mut::<crate::abilities::components::AbilitySpec>(ability_entity)
        {
            spec.active_count += 1;
        }
    }

    /// Called when the ability is activated.
    ///
    /// This is where the main ability logic should go (spawn projectiles, apply effects, etc).
    fn activate(
        &self,
        _world: &mut World,
        _ability_entity: Entity,
        _source: Entity,
        _target: Option<Entity>,
    ) {
    }

    /// Check if the ability can be committed (cost and cooldown check).
    ///
    /// Called before applying costs and cooldowns to ensure resources haven't changed.
    fn commit_check(
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
        let Some(source_tags) = world.get::<GameplayTagCountContainer>(source) else {
            return Err(ActivationCheckFailure::MissingComponents);
        };

        // Check cooldown
        if let Some(cd_id) = &definition.cooldown_effect
            && let Some(cd_def) = effect_registry.get(cd_id.as_ref())
            && source_tags.has_any_matching_gameplay_tags(&cd_def.granted_tags)
        {
            let mut cooldown_tags = GameplayTagContainer::default();
            cooldown_tags.append_matches_tags(
                &source_tags.explicit_tags,
                &cd_def.granted_tags,
                tags_manager,
            );
            return Err(ActivationCheckFailure::OnCooldown(cooldown_tags));
        }

        // TODO: Check cost when attribute system is ready

        Ok(())
    }

    /// Called when the ability is committed.
    ///
    /// Re-checks cost and cooldown, then executes commit logic.
    fn commit(
        &self,
        world: &mut World,
        ability_entity: Entity,
        source: Entity,
        tags_manager: &Res<GameplayTagsManager>,
    ) -> ActivationCheckResult {
        // Re-check cost and cooldown
        self.commit_check(world, ability_entity, source, tags_manager)?;

        // Execute commit logic
        self.commit_execute(world, ability_entity, source);

        Ok(())
    }

    /// Execute commit logic (apply costs and cooldowns).
    ///
    /// Override this for custom commit behavior.
    fn commit_execute(&self, _world: &mut World, _ability_entity: Entity, _source: Entity) {
        // TODO: Implement after GameplayEffect logic is complete
        // self.apply_cooldown(world, ability_entity, source);
        // self.apply_cost(world, ability_entity, source);
    }

    /// Apply cooldown effect to source.
    fn apply_cooldown(&self, _world: &mut World, _ability_entity: Entity, _source: Entity) {
        // TODO: Implement after GameplayEffect logic is complete
    }

    /// Apply cost effect to source.
    fn apply_cost(&self, _world: &mut World, _ability_entity: Entity, _source: Entity) {
        // TODO: Implement after GameplayEffect logic is complete
    }

    /// Called when the ability ends.
    ///
    /// Use this for cleanup logic. The `was_cancelled` parameter indicates
    /// whether the ability ended normally or was cancelled.
    fn end(&self, world: &mut World, ability_entity: Entity, _was_cancelled: bool) {
        if let Some(mut spec) = world.get_mut::<AbilitySpec>(ability_entity) {
            spec.is_active = false;
        }
    }
}
