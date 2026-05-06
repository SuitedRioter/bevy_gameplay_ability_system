//! Built-in GameplayEffectComponent implementations.
//!
//! This module provides standard components that extend gameplay effect behavior.

use bevy::prelude::*;

use super::ge_component::{EffectRemovalInfo, EffectRemovalReason, GameplayEffectComponent};
use super::query::GameplayEffectQuery;

/// Component that applies a probability check before allowing effect application.
///
/// Matches UE GAS's `UChanceToApplyGameplayEffectComponent`.
///
/// # Example
///
/// ```ignore
/// // 50% chance to apply
/// let component = ChanceToApplyComponent::new(0.5);
///
/// // Add to effect definition
/// let effect = GameplayEffectDefinition::new("critical_hit")
///     .add_component(Arc::new(component));
/// ```
#[derive(Debug, Clone)]
pub struct ChanceToApplyComponent {
    /// Probability of application [0.0, 1.0]
    pub chance: f32,
}

impl ChanceToApplyComponent {
    /// Creates a new chance component.
    ///
    /// # Parameters
    /// - `chance`: Probability [0.0, 1.0]. Values outside this range are clamped.
    pub fn new(chance: f32) -> Self {
        Self {
            chance: chance.clamp(0.0, 1.0),
        }
    }
}

impl GameplayEffectComponent for ChanceToApplyComponent {
    fn can_apply(
        &self,
        _effect_definition_id: &str,
        _source: Entity,
        _target: Entity,
        _world: &World,
    ) -> bool {
        // Simple pseudo-random based on system time
        let random_value = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as f32)
            .fract();
        random_value < self.chance
    }
}

/// Component that grants immunity to effects matching specific queries.
///
/// Matches UE GAS's `UImmunityGameplayEffectComponent`.
///
/// # Example
///
/// ```ignore
/// // Immune to all poison effects
/// let query = GameplayEffectQuery::new()
///     .with_owning_tags_any(vec!["Effect.Debuff.Poison"], &manager);
///
/// let component = ImmunityComponent::new(vec![query]);
///
/// // Add to effect definition
/// let effect = GameplayEffectDefinition::new("poison_immunity")
///     .add_component(Arc::new(component));
/// ```
#[derive(Clone)]
pub struct ImmunityComponent {
    /// Queries that define which effects to block
    pub immunity_queries: Vec<GameplayEffectQuery>,
}

impl std::fmt::Debug for ImmunityComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImmunityComponent")
            .field("immunity_queries", &self.immunity_queries)
            .finish()
    }
}

impl ImmunityComponent {
    /// Creates a new immunity component.
    pub fn new(immunity_queries: Vec<GameplayEffectQuery>) -> Self {
        Self { immunity_queries }
    }

    /// Adds an immunity query.
    pub fn add_query(mut self, query: GameplayEffectQuery) -> Self {
        self.immunity_queries.push(query);
        self
    }
}

impl GameplayEffectComponent for ImmunityComponent {
    fn on_effect_applied(&self, effect: Entity, target: Entity, _world: &mut World) {
        // Register immunity callback
        // This would need to be implemented in the effect system
        // For now, we just log
        info!(
            "Immunity effect {:?} applied to {:?}, blocking {} queries",
            effect,
            target,
            self.immunity_queries.len()
        );
    }

    fn can_apply(
        &self,
        _effect_definition_id: &str,
        _source: Entity,
        _target: Entity,
        _world: &World,
    ) -> bool {
        // Check if any active immunity effects on the target block this effect
        // This is a simplified check - full implementation would query all active immunity effects
        true
    }
}

/// Component that applies additional effects at specific lifecycle points.
///
/// Matches UE GAS's `UAdditionalEffectsGameplayEffectComponent`.
///
/// # Example
///
/// ```ignore
/// let component = AdditionalEffectsComponent::new()
///     .on_application(vec!["apply_damage".into()])
///     .on_complete_normal(vec!["heal_on_expire".into()]);
///
/// let effect = GameplayEffectDefinition::new("buff_with_effects")
///     .add_component(Arc::new(component));
/// ```
#[derive(Debug, Clone, Default)]
pub struct AdditionalEffectsComponent {
    /// Effects to apply when this effect is applied
    pub on_application: Vec<String>,
    /// Effects to apply when this effect completes (any reason)
    pub on_complete_always: Vec<String>,
    /// Effects to apply when this effect expires naturally
    pub on_complete_normal: Vec<String>,
    /// Effects to apply when this effect is removed prematurely
    pub on_complete_prematurely: Vec<String>,
}

impl AdditionalEffectsComponent {
    /// Creates a new additional effects component.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets effects to apply on application.
    pub fn on_application(mut self, effects: Vec<String>) -> Self {
        self.on_application = effects;
        self
    }

    /// Sets effects to apply on any completion.
    pub fn on_complete_always(mut self, effects: Vec<String>) -> Self {
        self.on_complete_always = effects;
        self
    }

    /// Sets effects to apply on normal expiration.
    pub fn on_complete_normal(mut self, effects: Vec<String>) -> Self {
        self.on_complete_normal = effects;
        self
    }

    /// Sets effects to apply on premature removal.
    pub fn on_complete_prematurely(mut self, effects: Vec<String>) -> Self {
        self.on_complete_prematurely = effects;
        self
    }
}

impl GameplayEffectComponent for AdditionalEffectsComponent {
    fn on_effect_applied(&self, effect: Entity, target: Entity, _world: &mut World) {
        // Apply on_application effects
        // Note: We need to trigger events through commands, not directly
        // This is a simplified implementation - full version would use Commands
        for effect_id in &self.on_application {
            info!(
                "Would apply additional effect '{}' from {:?} to {:?}",
                effect_id, effect, target
            );
            // TODO: Implement proper event triggering through Commands
            // commands.trigger(ApplyGameplayEffectEvent::new(effect_id.clone(), target));
        }
    }

    fn on_effect_removed(
        &self,
        effect: Entity,
        target: Entity,
        removal_info: &EffectRemovalInfo,
        _world: &mut World,
    ) {
        // Apply on_complete_always effects
        for effect_id in &self.on_complete_always {
            info!(
                "Would apply on_complete_always effect '{}' from {:?} to {:?}",
                effect_id, effect, target
            );
        }

        // Apply conditional effects based on removal reason
        let conditional_effects = match removal_info.reason {
            EffectRemovalReason::DurationExpired => &self.on_complete_normal,
            EffectRemovalReason::Cleared
            | EffectRemovalReason::TargetDespawned
            | EffectRemovalReason::Immunity
            | EffectRemovalReason::RemovedByOtherEffect => &self.on_complete_prematurely,
        };

        for effect_id in conditional_effects {
            info!(
                "Would apply conditional effect '{}' from {:?} to {:?}",
                effect_id, effect, target
            );
        }
    }
}

/// Component that removes other effects matching specific queries.
///
/// Matches UE GAS's `URemoveOtherGameplayEffectComponent`.
///
/// # Example
///
/// ```ignore
/// // Remove all poison effects when applied
/// let query = GameplayEffectQuery::new()
///     .with_owning_tags_any(vec!["Effect.Debuff.Poison"], &manager);
///
/// let component = RemoveOtherEffectsComponent::new(vec![query]);
///
/// let effect = GameplayEffectDefinition::new("cure_poison")
///     .add_component(Arc::new(component));
/// ```
#[derive(Clone)]
pub struct RemoveOtherEffectsComponent {
    /// Queries that define which effects to remove
    pub removal_queries: Vec<GameplayEffectQuery>,
}

impl std::fmt::Debug for RemoveOtherEffectsComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RemoveOtherEffectsComponent")
            .field("removal_queries", &self.removal_queries)
            .finish()
    }
}

impl RemoveOtherEffectsComponent {
    /// Creates a new remove other effects component.
    pub fn new(removal_queries: Vec<GameplayEffectQuery>) -> Self {
        Self { removal_queries }
    }

    /// Adds a removal query.
    pub fn add_query(mut self, query: GameplayEffectQuery) -> Self {
        self.removal_queries.push(query);
        self
    }
}

impl GameplayEffectComponent for RemoveOtherEffectsComponent {
    fn on_effect_applied(&self, _effect: Entity, target: Entity, world: &mut World) {
        // Find and remove matching effects
        let mut effects_to_remove = Vec::new();

        for query in &self.removal_queries {
            let matching = query.find_matching_effects(target, world);
            effects_to_remove.extend(matching);
        }

        // Remove the effects
        for effect_entity in effects_to_remove {
            world.despawn(effect_entity);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chance_to_apply_always() {
        let component = ChanceToApplyComponent::new(1.0);
        let mut app = App::new();
        let world = app.world_mut();

        let source = world.spawn_empty().id();
        let target = world.spawn_empty().id();

        // Should always allow
        for _ in 0..10 {
            assert!(component.can_apply("test", source, target, world));
        }
    }

    #[test]
    fn test_chance_to_apply_never() {
        let component = ChanceToApplyComponent::new(0.0);
        let mut app = App::new();
        let world = app.world_mut();

        let source = world.spawn_empty().id();
        let target = world.spawn_empty().id();

        // Should never allow
        for _ in 0..10 {
            assert!(!component.can_apply("test", source, target, world));
        }
    }

    #[test]
    fn test_chance_to_apply_clamps() {
        let component = ChanceToApplyComponent::new(1.5);
        assert_eq!(component.chance, 1.0);

        let component = ChanceToApplyComponent::new(-0.5);
        assert_eq!(component.chance, 0.0);
    }

    #[test]
    fn test_additional_effects_component_builder() {
        let component = AdditionalEffectsComponent::new()
            .on_application(vec!["effect1".to_string()])
            .on_complete_always(vec!["effect2".to_string()])
            .on_complete_normal(vec!["effect3".to_string()])
            .on_complete_prematurely(vec!["effect4".to_string()]);

        assert_eq!(component.on_application.len(), 1);
        assert_eq!(component.on_complete_always.len(), 1);
        assert_eq!(component.on_complete_normal.len(), 1);
        assert_eq!(component.on_complete_prematurely.len(), 1);
    }
}
