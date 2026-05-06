//! GameplayCue notify traits and components.
//!
//! This module defines the traits for implementing custom cue handlers.

use super::manager::GameplayCueParameters;
use bevy::prelude::*;
use bevy_gameplay_tag::gameplay_tag::GameplayTag;

/// Trait for static gameplay cue notifies.
///
/// Static cues are function-based and don't spawn entities.
/// They're ideal for simple effects like playing sounds or spawning particles.
pub trait GameplayCueNotifyStatic: Send + Sync + 'static {
    /// Called when the cue is executed.
    fn on_execute(&self, target: Entity, params: &GameplayCueParameters, commands: &mut Commands);

    /// Called when the cue becomes active (for duration-based cues).
    fn on_active(&self, target: Entity, params: &GameplayCueParameters, commands: &mut Commands) {
        // Default implementation just calls on_execute
        self.on_execute(target, params, commands);
    }

    /// Called when the cue is removed (for duration-based cues).
    fn on_remove(&self, target: Entity, params: &GameplayCueParameters, commands: &mut Commands) {
        // Default: do nothing
        let _ = (target, params, commands);
    }

    /// Called every frame while the cue is active (for WhileActive cues).
    fn while_active(
        &self,
        target: Entity,
        params: &GameplayCueParameters,
        commands: &mut Commands,
    ) {
        // Default: do nothing
        let _ = (target, params, commands);
    }
}

/// Component for actor-based gameplay cue notifies.
///
/// Actor cues spawn entities that persist for the duration of the cue.
/// They're ideal for complex effects that need to track state or update over time.
#[derive(Component, Debug, Clone)]
pub struct GameplayCueNotifyActor {
    /// The cue tag this actor responds to.
    pub cue_tag: GameplayTag,
    /// The target entity.
    pub target: Entity,
    /// Whether to automatically destroy this actor when the cue is removed.
    pub auto_destroy_on_remove: bool,
    /// The time when this cue was activated.
    pub activation_time: f32,
}

impl GameplayCueNotifyActor {
    /// Creates a new cue notify actor.
    pub fn new(cue_tag: GameplayTag, target: Entity, activation_time: f32) -> Self {
        Self {
            cue_tag,
            target,
            auto_destroy_on_remove: true,
            activation_time,
        }
    }

    /// Sets whether to auto-destroy on remove.
    pub fn with_auto_destroy(mut self, auto_destroy: bool) -> Self {
        self.auto_destroy_on_remove = auto_destroy;
        self
    }
}

/// Marker component for cue actors that should be removed.
#[derive(Component, Debug)]
pub struct CueActorPendingRemoval;

/// Burst cue component - for one-shot effects.
///
/// Burst cues are instantaneous effects that fire once and complete immediately.
/// They're ideal for impact effects, explosions, or any visual/audio feedback
/// that doesn't need to persist.
///
/// Maps to UE's `UGameplayCueNotify_Burst`.
///
/// # Example
/// ```ignore
/// commands.spawn((
///     BurstCue::new(GameplayTag::new("GameplayCue.Impact.Sword")),
///     Transform::from_translation(hit_location),
/// ));
/// ```
#[derive(Component, Debug, Clone)]
pub struct BurstCue {
    /// The cue tag this burst responds to.
    pub cue_tag: GameplayTag,
    /// The target entity (if any).
    pub target: Option<Entity>,
    /// Whether this burst has been executed.
    pub executed: bool,
}

impl BurstCue {
    /// Create a new burst cue.
    pub fn new(cue_tag: GameplayTag) -> Self {
        Self {
            cue_tag,
            target: None,
            executed: false,
        }
    }

    /// Set the target entity.
    pub fn with_target(mut self, target: Entity) -> Self {
        self.target = Some(target);
        self
    }
}

/// Looping cue component - for continuous effects with start/stop.
///
/// Looping cues persist for a duration and can be updated every frame.
/// They support multiple lifecycle events:
/// - OnApplication: Fired once when the cue starts
/// - OnLoopingStart: Fired when looping effects begin
/// - OnRecurring: Fired periodically (e.g., for DoT ticks)
/// - OnRemoval: Fired once when the cue ends
///
/// Maps to UE's `AGameplayCueNotify_Looping`.
///
/// # Example
/// ```ignore
/// commands.spawn((
///     LoopingCue::new(GameplayTag::new("GameplayCue.Buff.Shield"))
///         .with_target(player_entity)
///         .with_recurring_interval(1.0),
///     Transform::from_translation(player_position),
/// ));
/// ```
#[derive(Component, Debug, Clone)]
pub struct LoopingCue {
    /// The cue tag this looping cue responds to.
    pub cue_tag: GameplayTag,
    /// The target entity.
    pub target: Entity,
    /// Time when this cue was activated.
    pub activation_time: f64,
    /// Whether the looping effects have started.
    pub looping_started: bool,
    /// Whether the looping effects have been removed.
    pub looping_removed: bool,
    /// Optional recurring interval (in seconds).
    /// If set, OnRecurring will be called at this interval.
    pub recurring_interval: Option<f32>,
    /// Time of the last recurring event.
    pub last_recurring_time: f64,
}

impl LoopingCue {
    /// Create a new looping cue.
    pub fn new(cue_tag: GameplayTag, target: Entity, activation_time: f64) -> Self {
        Self {
            cue_tag,
            target,
            activation_time,
            looping_started: false,
            looping_removed: false,
            recurring_interval: None,
            last_recurring_time: activation_time,
        }
    }

    /// Set the target entity.
    pub fn with_target(mut self, target: Entity) -> Self {
        self.target = target;
        self
    }

    /// Set the recurring interval.
    pub fn with_recurring_interval(mut self, interval: f32) -> Self {
        self.recurring_interval = Some(interval);
        self
    }

    /// Check if it's time for a recurring event.
    pub fn should_recur(&self, current_time: f64) -> bool {
        if let Some(interval) = self.recurring_interval {
            current_time - self.last_recurring_time >= interval as f64
        } else {
            false
        }
    }

    /// Mark that a recurring event has occurred.
    pub fn mark_recurred(&mut self, current_time: f64) {
        self.last_recurring_time = current_time;
    }
}

/// HitImpact cue component - for collision-based effects.
///
/// HitImpact cues are specialized burst cues that include collision information
/// like hit normal, surface type, and impact velocity. They're ideal for
/// weapon impacts, projectile hits, or any collision-based feedback.
///
/// Maps to UE's `UGameplayCueNotify_HitImpact`.
///
/// # Example
/// ```ignore
/// commands.spawn((
///     HitImpactCue::new(
///         GameplayTag::new("GameplayCue.Impact.Bullet"),
///         hit_location,
///         hit_normal,
///     )
///     .with_target(hit_entity)
///     .with_surface_type("Metal"),
/// ));
/// ```
#[derive(Component, Debug, Clone)]
pub struct HitImpactCue {
    /// The cue tag this impact responds to.
    pub cue_tag: GameplayTag,
    /// The target entity that was hit.
    pub target: Option<Entity>,
    /// The location of the impact.
    pub hit_location: Vec3,
    /// The normal vector at the impact point.
    pub hit_normal: Vec3,
    /// Optional surface type (e.g., "Metal", "Wood", "Flesh").
    pub surface_type: Option<String>,
    /// Optional impact velocity magnitude.
    pub impact_velocity: Option<f32>,
    /// Whether this impact has been executed.
    pub executed: bool,
}

impl HitImpactCue {
    /// Create a new hit impact cue.
    pub fn new(cue_tag: GameplayTag, hit_location: Vec3, hit_normal: Vec3) -> Self {
        Self {
            cue_tag,
            target: None,
            hit_location,
            hit_normal,
            surface_type: None,
            impact_velocity: None,
            executed: false,
        }
    }

    /// Set the target entity.
    pub fn with_target(mut self, target: Entity) -> Self {
        self.target = Some(target);
        self
    }

    /// Set the surface type.
    pub fn with_surface_type(mut self, surface_type: impl Into<String>) -> Self {
        self.surface_type = Some(surface_type.into());
        self
    }

    /// Set the impact velocity.
    pub fn with_impact_velocity(mut self, velocity: f32) -> Self {
        self.impact_velocity = Some(velocity);
        self
    }
}

/// Marker component for looping cues that should be removed.
#[derive(Component, Debug)]
pub struct LoopingCuePendingRemoval;

/// Example static cue implementation.
///
/// This is a simple example showing how to implement a static cue.
pub struct ExampleStaticCue;

impl GameplayCueNotifyStatic for ExampleStaticCue {
    fn on_execute(&self, target: Entity, params: &GameplayCueParameters, _commands: &mut Commands) {
        info!(
            "Example cue executed on {:?} at {:?} with magnitude {}",
            target, params.location, params.raw_magnitude
        );
    }

    fn on_active(&self, target: Entity, params: &GameplayCueParameters, _commands: &mut Commands) {
        info!(
            "Example cue activated on {:?} at {:?}",
            target, params.location
        );
    }

    fn on_remove(&self, target: Entity, _params: &GameplayCueParameters, _commands: &mut Commands) {
        info!("Example cue removed from {:?}", target);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cue_notify_actor_creation() {
        let tag = GameplayTag::new("GameplayCue.Test");
        let target = Entity::PLACEHOLDER;
        let actor = GameplayCueNotifyActor::new(tag.clone(), target, 0.0);

        assert_eq!(actor.cue_tag, tag);
        assert_eq!(actor.target, target);
        assert!(actor.auto_destroy_on_remove);
    }

    #[test]
    fn test_cue_notify_actor_builder() {
        let tag = GameplayTag::new("GameplayCue.Test");
        let target = Entity::PLACEHOLDER;
        let actor = GameplayCueNotifyActor::new(tag, target, 0.0).with_auto_destroy(false);

        assert!(!actor.auto_destroy_on_remove);
    }

    #[test]
    fn test_burst_cue_creation() {
        let tag = GameplayTag::new("GameplayCue.Impact");
        let burst = BurstCue::new(tag.clone());

        assert_eq!(burst.cue_tag, tag);
        assert!(burst.target.is_none());
        assert!(!burst.executed);
    }

    #[test]
    fn test_burst_cue_with_target() {
        let tag = GameplayTag::new("GameplayCue.Impact");
        let target = Entity::from_bits(42);
        let burst = BurstCue::new(tag).with_target(target);

        assert_eq!(burst.target, Some(target));
    }

    #[test]
    fn test_looping_cue_creation() {
        let tag = GameplayTag::new("GameplayCue.Buff");
        let target = Entity::from_bits(1);
        let looping = LoopingCue::new(tag.clone(), target, 0.0);

        assert_eq!(looping.cue_tag, tag);
        assert_eq!(looping.target, target);
        assert!(!looping.looping_started);
        assert!(!looping.looping_removed);
        assert!(looping.recurring_interval.is_none());
    }

    #[test]
    fn test_looping_cue_recurring() {
        let tag = GameplayTag::new("GameplayCue.DoT");
        let target = Entity::from_bits(1);
        let mut looping = LoopingCue::new(tag, target, 0.0).with_recurring_interval(1.0);

        // Should not recur immediately
        assert!(!looping.should_recur(0.5));

        // Should recur after interval
        assert!(looping.should_recur(1.0));

        // Mark as recurred
        looping.mark_recurred(1.0);
        assert_eq!(looping.last_recurring_time, 1.0);

        // Should not recur again immediately
        assert!(!looping.should_recur(1.5));
    }

    #[test]
    fn test_hit_impact_cue_creation() {
        let tag = GameplayTag::new("GameplayCue.Impact.Bullet");
        let location = Vec3::new(1.0, 2.0, 3.0);
        let normal = Vec3::new(0.0, 1.0, 0.0);
        let impact = HitImpactCue::new(tag.clone(), location, normal);

        assert_eq!(impact.cue_tag, tag);
        assert_eq!(impact.hit_location, location);
        assert_eq!(impact.hit_normal, normal);
        assert!(impact.target.is_none());
        assert!(impact.surface_type.is_none());
        assert!(impact.impact_velocity.is_none());
        assert!(!impact.executed);
    }

    #[test]
    fn test_hit_impact_cue_builder() {
        let tag = GameplayTag::new("GameplayCue.Impact.Sword");
        let location = Vec3::ZERO;
        let normal = Vec3::Y;
        let target = Entity::from_bits(99);

        let impact = HitImpactCue::new(tag, location, normal)
            .with_target(target)
            .with_surface_type("Metal")
            .with_impact_velocity(15.0);

        assert_eq!(impact.target, Some(target));
        assert_eq!(impact.surface_type, Some("Metal".to_string()));
        assert_eq!(impact.impact_velocity, Some(15.0));
    }
}
