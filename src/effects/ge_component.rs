//! GameplayEffect component system.
//!
//! This module implements UE 5.3+'s modular GameplayEffectComponent architecture.
//! Components allow extending effect behavior without modifying the core definition.

use bevy::prelude::*;
use std::sync::Arc;

/// Trait for modular GameplayEffect components.
///
/// Components are executed at specific lifecycle points of a gameplay effect:
/// - `can_apply`: Before the effect is applied (can block application)
/// - `on_effect_applied`: After the effect is successfully applied
/// - `on_effect_removed`: When the effect is removed from the target
///
/// # Example
///
/// ```ignore
/// struct LoggingComponent;
///
/// impl GameplayEffectComponent for LoggingComponent {
///     fn on_effect_applied(&self, effect: Entity, target: Entity, world: &mut World) {
///         info!("Effect {:?} applied to {:?}", effect, target);
///     }
/// }
/// ```
pub trait GameplayEffectComponent: Send + Sync {
    /// Called when an effect is applied to a target.
    ///
    /// This is invoked after the effect entity is spawned and all modifiers are created.
    ///
    /// # Parameters
    /// - `effect`: The active effect entity
    /// - `target`: The entity receiving the effect
    /// - `world`: Mutable world access for spawning entities or triggering events
    fn on_effect_applied(&self, effect: Entity, target: Entity, world: &mut World) {
        // Default: no-op
        let _ = (effect, target, world);
    }

    /// Called when an effect is removed from a target.
    ///
    /// This is invoked before the effect entity is despawned.
    ///
    /// # Parameters
    /// - `effect`: The active effect entity being removed
    /// - `target`: The entity that had the effect
    /// - `removal_info`: Information about why the effect was removed
    /// - `world`: Mutable world access
    fn on_effect_removed(
        &self,
        effect: Entity,
        target: Entity,
        removal_info: &EffectRemovalInfo,
        world: &mut World,
    ) {
        // Default: no-op
        let _ = (effect, target, removal_info, world);
    }

    /// Check if the effect can be applied to the target.
    ///
    /// Return `false` to block the effect application.
    ///
    /// # Parameters
    /// - `effect_definition_id`: The definition ID of the effect being applied
    /// - `source`: The entity applying the effect
    /// - `target`: The entity receiving the effect
    /// - `world`: Immutable world access for queries
    ///
    /// # Returns
    /// `true` if the effect can be applied, `false` to block it
    fn can_apply(
        &self,
        effect_definition_id: &str,
        source: Entity,
        target: Entity,
        world: &World,
    ) -> bool {
        // Default: allow
        let _ = (effect_definition_id, source, target, world);
        true
    }
}

/// Information about why an effect was removed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectRemovalReason {
    /// Effect duration expired naturally
    DurationExpired,
    /// Effect was explicitly removed (e.g., via RemoveGameplayEffect)
    Cleared,
    /// Effect was removed because the target entity was despawned
    TargetDespawned,
    /// Effect was removed due to immunity
    Immunity,
    /// Effect was removed by a RemoveOtherEffects component
    RemovedByOtherEffect,
}

/// Context information for effect removal.
#[derive(Debug, Clone)]
pub struct EffectRemovalInfo {
    /// Why the effect was removed
    pub reason: EffectRemovalReason,
    /// The effect definition ID
    pub effect_definition_id: String,
    /// The source entity that applied the effect
    pub source: Entity,
    /// Stack count at removal (for stacking effects)
    pub stack_count: i32,
}

/// Type alias for boxed components.
pub type BoxedGameplayEffectComponent = Arc<dyn GameplayEffectComponent>;

/// Helper function to invoke `can_apply` on all components.
///
/// Returns `true` only if ALL components allow the application.
pub fn check_components_can_apply(
    components: &[BoxedGameplayEffectComponent],
    effect_definition_id: &str,
    source: Entity,
    target: Entity,
    world: &World,
) -> bool {
    components
        .iter()
        .all(|c| c.can_apply(effect_definition_id, source, target, world))
}

/// Helper function to invoke `on_effect_applied` on all components.
pub fn invoke_components_on_applied(
    components: &[BoxedGameplayEffectComponent],
    effect: Entity,
    target: Entity,
    world: &mut World,
) {
    for component in components {
        component.on_effect_applied(effect, target, world);
    }
}

/// Helper function to invoke `on_effect_removed` on all components.
pub fn invoke_components_on_removed(
    components: &[BoxedGameplayEffectComponent],
    effect: Entity,
    target: Entity,
    removal_info: &EffectRemovalInfo,
    world: &mut World,
) {
    for component in components {
        component.on_effect_removed(effect, target, removal_info, world);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestComponent {
        allow_apply: bool,
    }

    impl GameplayEffectComponent for TestComponent {
        fn can_apply(
            &self,
            _effect_definition_id: &str,
            _source: Entity,
            _target: Entity,
            _world: &World,
        ) -> bool {
            self.allow_apply
        }
    }

    #[test]
    fn test_check_components_can_apply_all_allow() {
        let mut app = App::new();
        let world = app.world_mut();

        let components: Vec<BoxedGameplayEffectComponent> = vec![
            Arc::new(TestComponent { allow_apply: true }),
            Arc::new(TestComponent { allow_apply: true }),
        ];

        let source = world.spawn_empty().id();
        let target = world.spawn_empty().id();

        assert!(check_components_can_apply(
            &components,
            "test_effect",
            source,
            target,
            world
        ));
    }

    #[test]
    fn test_check_components_can_apply_one_blocks() {
        let mut app = App::new();
        let world = app.world_mut();

        let components: Vec<BoxedGameplayEffectComponent> = vec![
            Arc::new(TestComponent { allow_apply: true }),
            Arc::new(TestComponent { allow_apply: false }),
        ];

        let source = world.spawn_empty().id();
        let target = world.spawn_empty().id();

        assert!(!check_components_can_apply(
            &components,
            "test_effect",
            source,
            target,
            world
        ));
    }
}
