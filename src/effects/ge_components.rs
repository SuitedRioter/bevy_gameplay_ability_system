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
    fn on_effect_applied(&self, effect: Entity, target: Entity, world: &mut World) {
        // Register this effect as granting immunity
        // Store the immunity queries in a component on the target
        if let Ok(mut entity_mut) = world.get_entity_mut(target) {
            if let Some(mut active_immunities) = entity_mut.get_mut::<ActiveImmunityEffects>() {
                active_immunities.add_immunity(effect, self.immunity_queries.clone());
            } else {
                entity_mut.insert(ActiveImmunityEffects::new(
                    effect,
                    self.immunity_queries.clone(),
                ));
            }
        }

        info!(
            "Immunity effect {:?} applied to {:?}, blocking {} queries",
            effect,
            target,
            self.immunity_queries.len()
        );
    }

    fn on_effect_removed(
        &self,
        effect: Entity,
        target: Entity,
        _removal_info: &EffectRemovalInfo,
        world: &mut World,
    ) {
        // Remove this effect's immunity grants
        if let Ok(mut entity_mut) = world.get_entity_mut(target) {
            if let Some(mut active_immunities) = entity_mut.get_mut::<ActiveImmunityEffects>() {
                active_immunities.remove_immunity(effect);

                // If no more immunities, remove the component
                if active_immunities.is_empty() {
                    entity_mut.remove::<ActiveImmunityEffects>();
                }
            }
        }

        info!(
            "Immunity effect {:?} removed from {:?}",
            effect, target
        );
    }

    fn can_apply(
        &self,
        _effect_definition_id: &str,
        _source: Entity,
        _target: Entity,
        _world: &World,
    ) -> bool {
        // Immunity components don't block their own application
        // They block OTHER effects after being applied
        true
    }
}

/// Component tracking active immunity effects on an entity.
///
/// This component is added to entities that have active immunity-granting effects.
/// It stores the immunity queries from all active immunity effects.
#[derive(Component, Debug, Clone)]
pub struct ActiveImmunityEffects {
    /// Map of effect entity -> immunity queries granted by that effect
    immunities: std::collections::HashMap<Entity, Vec<GameplayEffectQuery>>,
}

impl ActiveImmunityEffects {
    /// Creates a new active immunity tracker with one effect.
    pub fn new(effect: Entity, queries: Vec<GameplayEffectQuery>) -> Self {
        let mut immunities = std::collections::HashMap::new();
        immunities.insert(effect, queries);
        Self { immunities }
    }

    /// Adds immunity queries from a new effect.
    pub fn add_immunity(&mut self, effect: Entity, queries: Vec<GameplayEffectQuery>) {
        self.immunities.insert(effect, queries);
    }

    /// Removes immunity queries from an effect.
    pub fn remove_immunity(&mut self, effect: Entity) {
        self.immunities.remove(&effect);
    }

    /// Returns true if no immunities are active.
    pub fn is_empty(&self) -> bool {
        self.immunities.is_empty()
    }

    /// Checks if the given effect definition is blocked by any active immunity.
    ///
    /// Returns the blocking effect entity if blocked, None otherwise.
    pub fn is_effect_blocked(
        &self,
        effect_definition_id: &str,
        source: Option<Entity>,
        target: Entity,
        world: &World,
    ) -> Option<Entity> {
        for (immunity_effect, queries) in &self.immunities {
            for query in queries {
                if query.matches_effect(effect_definition_id, source, target, world) {
                    return Some(*immunity_effect);
                }
            }
        }
        None
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
    use bevy::ecs::system::RunSystemOnce;

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

    #[test]
    fn test_immunity_component_blocks_matching_effects() {
        let mut app = App::new();
        app.add_plugins(bevy_gameplay_tag::GameplayTagsPlugin::with_data_path(
            "assets/gameplay_tags.json".to_string(),
        ));
        app.add_plugins(crate::effects::EffectPlugin);
        app.update(); // Load tags

        // Register immunity effect
        let poison_query = crate::effects::query::GameplayEffectQuery::new()
            .with_definition_id("poison_damage");

        let immunity_effect = crate::effects::definition::GameplayEffectDefinition::new("poison_immunity")
            .with_duration_policy(crate::effects::definition::DurationPolicy::Infinite)
            .add_component(std::sync::Arc::new(ImmunityComponent::new(vec![poison_query])));

        app.world_mut()
            .resource_mut::<crate::effects::definition::GameplayEffectRegistry>()
            .register(immunity_effect);

        // Register poison effect
        let poison_effect = crate::effects::definition::GameplayEffectDefinition::new("poison_damage")
            .with_duration_policy(crate::effects::definition::DurationPolicy::HasDuration)
            .with_duration(5.0)
            .add_modifier(crate::effects::definition::ModifierInfo::new(
                "Health",
                crate::effects::components::ModifierOperation::AddBase,
                crate::effects::definition::MagnitudeCalculation::ScalableFloat {
                    base_value: -10.0,
                    level_multiplier: 1.0,
                },
            ));

        app.world_mut()
            .resource_mut::<crate::effects::definition::GameplayEffectRegistry>()
            .register(poison_effect);

        // Spawn player
        let player = app
            .world_mut()
            .spawn((
                Name::new("Player"),
                crate::core::OwnedTags::default(),
                crate::core::BlockedAbilityTags::default(),
            ))
            .id();

        // Apply immunity effect
        app.world_mut().commands().trigger(crate::effects::systems::ApplyGameplayEffectEvent {
            spec: crate::effects::components::GameplayEffectSpec {
                effect_id: "poison_immunity".into(),
                target: player,
                level: 1,
                context: Default::default(),
                set_by_caller_magnitudes: Default::default(),
                captured_attributes: Default::default(),
            },
        });

        app.update();

        // Verify immunity component was added
        assert!(app
            .world()
            .get::<ActiveImmunityEffects>(player)
            .is_some());

        // Try to apply poison - should be blocked
        app.world_mut().commands().trigger(crate::effects::systems::ApplyGameplayEffectEvent {
            spec: crate::effects::components::GameplayEffectSpec {
                effect_id: "poison_damage".into(),
                target: player,
                level: 1,
                context: Default::default(),
                set_by_caller_magnitudes: Default::default(),
                captured_attributes: Default::default(),
            },
        });

        app.update();

        // Verify poison was NOT applied by checking active effects
        let has_poison = app.world_mut().run_system_once(
            |effects: Query<&crate::effects::components::ActiveGameplayEffect>| {
                effects
                    .iter()
                    .any(|effect| effect.definition_id.as_ref() == "poison_damage")
            },
        ).unwrap_or(false);

        assert!(!has_poison, "Poison should have been blocked by immunity");
    }
}
