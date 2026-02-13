//! GameplayCue manager.
//!
//! This module manages the registration and execution of gameplay cues.

use bevy::prelude::*;
use bevy_gameplay_tag::gameplay_tag::GameplayTag;
use std::collections::HashMap;

/// GameplayCue event type.
///
/// Determines how the cue should be executed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameplayCueEvent {
    /// Cue is executed once.
    OnActive,
    /// Cue executes continuously while active.
    WhileActive,
    /// Cue is executed once (similar to OnActive, but for instant effects).
    Executed,
    /// Cue is removed/cleaned up.
    Removed,
}

/// Parameters passed to gameplay cue handlers.
#[derive(Debug, Clone)]
pub struct GameplayCueParameters {
    /// Normalized magnitude (0.0 to 1.0).
    pub normalized_magnitude: f32,
    /// Raw magnitude value.
    pub raw_magnitude: f32,
    /// Location where the cue should be spawned.
    pub location: Vec3,
    /// Normal vector (for surface effects).
    pub normal: Vec3,
    /// The entity that instigated this cue.
    pub instigator: Option<Entity>,
    /// The entity that caused the effect (e.g., projectile).
    pub effect_causer: Option<Entity>,
    /// The target entity.
    pub target: Option<Entity>,
}

impl Default for GameplayCueParameters {
    fn default() -> Self {
        Self {
            normalized_magnitude: 0.0,
            raw_magnitude: 0.0,
            location: Vec3::ZERO,
            normal: Vec3::Y,
            instigator: None,
            effect_causer: None,
            target: None,
        }
    }
}

impl GameplayCueParameters {
    /// Creates new cue parameters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the magnitude values.
    pub fn with_magnitude(mut self, raw: f32, normalized: f32) -> Self {
        self.raw_magnitude = raw;
        self.normalized_magnitude = normalized;
        self
    }

    /// Sets the location.
    pub fn with_location(mut self, location: Vec3) -> Self {
        self.location = location;
        self
    }

    /// Sets the normal.
    pub fn with_normal(mut self, normal: Vec3) -> Self {
        self.normal = normal;
        self
    }

    /// Sets the instigator.
    pub fn with_instigator(mut self, instigator: Entity) -> Self {
        self.instigator = Some(instigator);
        self
    }

    /// Sets the effect causer.
    pub fn with_effect_causer(mut self, causer: Entity) -> Self {
        self.effect_causer = Some(causer);
        self
    }

    /// Sets the target.
    pub fn with_target(mut self, target: Entity) -> Self {
        self.target = Some(target);
        self
    }
}

/// Information about a registered cue notify.
#[derive(Debug, Clone)]
pub struct CueNotifyInfo {
    /// The tag this cue responds to.
    pub tag: GameplayTag,
    /// Whether this is a static cue (function-based) or actor-based.
    pub is_static: bool,
}

/// Pending cue execution.
#[derive(Debug, Clone)]
pub struct PendingCueExecution {
    /// The cue tag to execute.
    pub cue_tag: GameplayTag,
    /// The event type.
    pub event_type: GameplayCueEvent,
    /// The parameters.
    pub parameters: GameplayCueParameters,
}

/// GameplayCue manager resource.
///
/// This manages all registered cues and handles cue execution.
#[derive(Resource, Default)]
pub struct GameplayCueManager {
    /// Map of registered cues.
    pub loaded_cues: HashMap<GameplayTag, CueNotifyInfo>,
    /// Active cue actors (tag -> entity).
    pub active_cues: HashMap<GameplayTag, Vec<Entity>>,
    /// Pending cue executions (for batching).
    pub pending_cues: Vec<PendingCueExecution>,
    /// Whether batching is currently active.
    pub batching_active: bool,
}

impl GameplayCueManager {
    /// Creates a new cue manager.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a static cue notify.
    pub fn register_static_cue(&mut self, tag: GameplayTag) {
        self.loaded_cues.insert(
            tag.clone(),
            CueNotifyInfo {
                tag,
                is_static: true,
            },
        );
    }

    /// Registers an actor-based cue notify.
    pub fn register_actor_cue(&mut self, tag: GameplayTag) {
        self.loaded_cues.insert(
            tag.clone(),
            CueNotifyInfo {
                tag,
                is_static: false,
            },
        );
    }

    /// Executes a gameplay cue.
    pub fn execute_cue(
        &mut self,
        cue_tag: GameplayTag,
        event_type: GameplayCueEvent,
        parameters: GameplayCueParameters,
    ) {
        if self.batching_active {
            // Queue for later execution
            self.pending_cues.push(PendingCueExecution {
                cue_tag,
                event_type,
                parameters,
            });
        } else {
            // Execute immediately
            self.execute_cue_internal(cue_tag, event_type, parameters);
        }
    }

    /// Internal cue execution.
    fn execute_cue_internal(
        &mut self,
        cue_tag: GameplayTag,
        event_type: GameplayCueEvent,
        _parameters: GameplayCueParameters,
    ) {
        // Find matching cues (including parent tags)
        let matching_cues: Vec<_> = self
            .loaded_cues
            .keys()
            .filter(|tag| cue_tag.matches_tag_exact(tag))
            .cloned()
            .collect();

        for matched_tag in matching_cues {
            if let Some(info) = self.loaded_cues.get(&matched_tag) {
                if info.is_static {
                    // Static cues are handled by systems
                    // We'll emit an event for them
                } else {
                    // Actor-based cues need to be spawned/managed
                    match event_type {
                        GameplayCueEvent::OnActive => {
                            // Spawn a cue actor
                            // This will be handled by systems
                        }
                        GameplayCueEvent::Removed => {
                            // Remove active cue actors
                            if let Some(_actors) = self.active_cues.get(&matched_tag) {
                                // Mark for removal
                                // This will be handled by systems
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    /// Starts batching cue executions.
    pub fn start_batching(&mut self) {
        self.batching_active = true;
    }

    /// Ends batching and executes all pending cues.
    pub fn end_batching(&mut self) {
        self.batching_active = false;
        let pending = std::mem::take(&mut self.pending_cues);
        for cue in pending {
            self.execute_cue_internal(cue.cue_tag, cue.event_type, cue.parameters);
        }
    }

    /// Adds an active cue actor.
    pub fn add_active_cue(&mut self, tag: GameplayTag, entity: Entity) {
        self.active_cues.entry(tag).or_default().push(entity);
    }

    /// Removes an active cue actor.
    pub fn remove_active_cue(&mut self, tag: &GameplayTag, entity: Entity) {
        if let Some(actors) = self.active_cues.get_mut(tag) {
            actors.retain(|&e| e != entity);
            if actors.is_empty() {
                self.active_cues.remove(tag);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cue_manager_registration() {
        let mut manager = GameplayCueManager::new();
        let tag = GameplayTag::new("GameplayCue.Test");

        manager.register_static_cue(tag.clone());
        assert!(manager.loaded_cues.contains_key(&tag));
        assert!(manager.loaded_cues.get(&tag).unwrap().is_static);
    }

    #[test]
    fn test_cue_parameters_builder() {
        let params = GameplayCueParameters::new()
            .with_magnitude(100.0, 0.5)
            .with_location(Vec3::new(1.0, 2.0, 3.0));

        assert_eq!(params.raw_magnitude, 100.0);
        assert_eq!(params.normalized_magnitude, 0.5);
        assert_eq!(params.location, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_batching() {
        let mut manager = GameplayCueManager::new();
        let tag = GameplayTag::new("GameplayCue.Test");

        manager.start_batching();
        assert!(manager.batching_active);

        manager.execute_cue(
            tag.clone(),
            GameplayCueEvent::Executed,
            GameplayCueParameters::new(),
        );

        assert_eq!(manager.pending_cues.len(), 1);

        manager.end_batching();
        assert!(!manager.batching_active);
        assert_eq!(manager.pending_cues.len(), 0);
    }
}
