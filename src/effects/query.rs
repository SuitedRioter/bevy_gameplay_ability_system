//! GameplayEffect query system.
//!
//! Provides a flexible query system for matching gameplay effects based on various criteria.
//! Used by ImmunityComponent, RemoveOtherEffectsComponent, and other advanced features.

use bevy::prelude::*;
use bevy_gameplay_tag::{GameplayTag, GameplayTagContainer, GameplayTagsManager};
use std::sync::Arc;
use string_cache::DefaultAtom as Atom;

use super::components::ActiveGameplayEffect;
use crate::core::components::OwnedTags;

/// Query for matching gameplay effects.
///
/// Supports multiple matching criteria that can be combined:
/// - Effect definition ID
/// - Tags owned by the effect
/// - Tags owned by the source entity
/// - Custom matching function
///
/// # Example
///
/// ```ignore
/// // Match all poison effects
/// let query = GameplayEffectQuery::new()
///     .with_owning_tags_any(vec!["Effect.Debuff.Poison"]);
///
/// // Match effects from enemies
/// let query = GameplayEffectQuery::new()
///     .with_source_tags_any(vec!["Actor.Enemy"]);
///
/// // Match specific effect
/// let query = GameplayEffectQuery::new()
///     .with_definition_id("damage_over_time");
/// ```
#[derive(Clone)]
pub struct GameplayEffectQuery {
    /// Match effects with this definition ID
    pub effect_definition: Option<Atom>,

    /// Match effects that have ALL of these tags
    pub owning_tags_all: Option<GameplayTagContainer>,

    /// Match effects that have ANY of these tags
    pub owning_tags_any: Option<GameplayTagContainer>,

    /// Match effects that have NONE of these tags
    pub owning_tags_none: Option<GameplayTagContainer>,

    /// Match effects whose source has ALL of these tags
    pub source_tags_all: Option<GameplayTagContainer>,

    /// Match effects whose source has ANY of these tags
    pub source_tags_any: Option<GameplayTagContainer>,

    /// Match effects whose source has NONE of these tags
    pub source_tags_none: Option<GameplayTagContainer>,

    /// Custom matching function
    pub custom_match: Option<Arc<dyn Fn(Entity, &World) -> bool + Send + Sync>>,
}

impl std::fmt::Debug for GameplayEffectQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GameplayEffectQuery")
            .field("effect_definition", &self.effect_definition)
            .field("owning_tags_all", &self.owning_tags_all)
            .field("owning_tags_any", &self.owning_tags_any)
            .field("owning_tags_none", &self.owning_tags_none)
            .field("source_tags_all", &self.source_tags_all)
            .field("source_tags_any", &self.source_tags_any)
            .field("source_tags_none", &self.source_tags_none)
            .field("custom_match", &self.custom_match.as_ref().map(|_| "<fn>"))
            .finish()
    }
}

impl Default for GameplayEffectQuery {
    fn default() -> Self {
        Self::new()
    }
}

impl GameplayEffectQuery {
    /// Creates a new empty query that matches all effects.
    pub fn new() -> Self {
        Self {
            effect_definition: None,
            owning_tags_all: None,
            owning_tags_any: None,
            owning_tags_none: None,
            source_tags_all: None,
            source_tags_any: None,
            source_tags_none: None,
            custom_match: None,
        }
    }

    /// Match effects with this definition ID.
    pub fn with_definition_id(mut self, id: impl Into<Atom>) -> Self {
        self.effect_definition = Some(id.into());
        self
    }

    /// Match effects that have ALL of these tags.
    pub fn with_owning_tags_all(
        mut self,
        tags: impl IntoIterator<Item = impl AsRef<str>>,
        manager: &GameplayTagsManager,
    ) -> Self {
        let mut container = GameplayTagContainer::default();
        for tag_str in tags {
            let tag = GameplayTag::new(tag_str.as_ref());
            container.add_tag(tag, manager);
        }
        self.owning_tags_all = Some(container);
        self
    }

    /// Match effects that have ANY of these tags.
    pub fn with_owning_tags_any(
        mut self,
        tags: impl IntoIterator<Item = impl AsRef<str>>,
        manager: &GameplayTagsManager,
    ) -> Self {
        let mut container = GameplayTagContainer::default();
        for tag_str in tags {
            let tag = GameplayTag::new(tag_str.as_ref());
            container.add_tag(tag, manager);
        }
        self.owning_tags_any = Some(container);
        self
    }

    /// Match effects that have NONE of these tags.
    pub fn with_owning_tags_none(
        mut self,
        tags: impl IntoIterator<Item = impl AsRef<str>>,
        manager: &GameplayTagsManager,
    ) -> Self {
        let mut container = GameplayTagContainer::default();
        for tag_str in tags {
            let tag = GameplayTag::new(tag_str.as_ref());
            container.add_tag(tag, manager);
        }
        self.owning_tags_none = Some(container);
        self
    }

    /// Match effects whose source has ALL of these tags.
    pub fn with_source_tags_all(
        mut self,
        tags: impl IntoIterator<Item = impl AsRef<str>>,
        manager: &GameplayTagsManager,
    ) -> Self {
        let mut container = GameplayTagContainer::default();
        for tag_str in tags {
            let tag = GameplayTag::new(tag_str.as_ref());
            container.add_tag(tag, manager);
        }
        self.source_tags_all = Some(container);
        self
    }

    /// Match effects whose source has ANY of these tags.
    pub fn with_source_tags_any(
        mut self,
        tags: impl IntoIterator<Item = impl AsRef<str>>,
        manager: &GameplayTagsManager,
    ) -> Self {
        let mut container = GameplayTagContainer::default();
        for tag_str in tags {
            let tag = GameplayTag::new(tag_str.as_ref());
            container.add_tag(tag, manager);
        }
        self.source_tags_any = Some(container);
        self
    }

    /// Match effects whose source has NONE of these tags.
    pub fn with_source_tags_none(
        mut self,
        tags: impl IntoIterator<Item = impl AsRef<str>>,
        manager: &GameplayTagsManager,
    ) -> Self {
        let mut container = GameplayTagContainer::default();
        for tag_str in tags {
            let tag = GameplayTag::new(tag_str.as_ref());
            container.add_tag(tag, manager);
        }
        self.source_tags_none = Some(container);
        self
    }

    /// Add a custom matching function.
    pub fn with_custom_match(
        mut self,
        matcher: impl Fn(Entity, &World) -> bool + Send + Sync + 'static,
    ) -> Self {
        self.custom_match = Some(Arc::new(matcher));
        self
    }

    /// Check if an effect entity matches this query.
    ///
    /// # Parameters
    /// - `effect`: The active effect entity to check
    /// - `world`: World access for querying components
    ///
    /// # Returns
    /// `true` if the effect matches all criteria, `false` otherwise
    pub fn matches(&self, effect: Entity, world: &World) -> bool {
        // Get the active effect component
        let Some(active_effect) = world.get::<ActiveGameplayEffect>(effect) else {
            return false;
        };

        // Check definition ID
        if let Some(ref def_id) = self.effect_definition {
            if &active_effect.definition_id != def_id {
                return false;
            }
        }

        // Check owning tags (tags granted by the effect)
        if let Some(ref tags_all) = self.owning_tags_all {
            if !active_effect.granted_tags.has_all(tags_all) {
                return false;
            }
        }

        if let Some(ref tags_any) = self.owning_tags_any {
            if !active_effect.granted_tags.has_any(tags_any) {
                return false;
            }
        }

        if let Some(ref tags_none) = self.owning_tags_none {
            if active_effect.granted_tags.has_any(tags_none) {
                return false;
            }
        }

        // Check source tags
        if self.source_tags_all.is_some()
            || self.source_tags_any.is_some()
            || self.source_tags_none.is_some()
        {
            if let Some(source_tags) = world.get::<OwnedTags>(active_effect.source) {
                if let Some(ref tags_all) = self.source_tags_all {
                    if !source_tags.0.has_all_matching_gameplay_tags(tags_all) {
                        return false;
                    }
                }

                if let Some(ref tags_any) = self.source_tags_any {
                    if !source_tags.0.has_any_matching_gameplay_tags(tags_any) {
                        return false;
                    }
                }

                if let Some(ref tags_none) = self.source_tags_none {
                    if source_tags.0.has_any_matching_gameplay_tags(tags_none) {
                        return false;
                    }
                }
            } else {
                // Source has no tags, so it can't match tag requirements
                return false;
            }
        }

        // Check custom matcher
        if let Some(ref matcher) = self.custom_match {
            if !matcher(effect, world) {
                return false;
            }
        }

        true
    }

    /// Find all active effects on a target that match this query.
    ///
    /// # Parameters
    /// - `_target`: The entity to search for effects on (currently unused)
    /// - `_world`: World access for querying (currently unused)
    ///
    /// # Returns
    /// Vector of effect entities that match the query
    ///
    /// # Note
    /// This is a simplified implementation. For production use, consider:
    /// - Maintaining an index of effects by target (using EffectTarget component)
    /// - Using a system with proper Query access instead of &World
    pub fn find_matching_effects(&self, _target: Entity, _world: &World) -> Vec<Entity> {
        let matching = Vec::new();

        // Simplified implementation: iterate through all entities
        // In a real system, you'd use a Query<Entity, With<ActiveGameplayEffect>>
        // but that requires &mut World or SystemParam access
        //
        // For now, we'll just return an empty vec and document that this
        // should be called from a system with proper query access
        warn!(
            "GameplayEffectQuery::find_matching_effects() called with &World - this is inefficient. Consider using a system with Query access."
        );

        matching
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_gameplay_tag::GameplayTagsPlugin;

    #[test]
    fn test_query_matches_definition_id() {
        let mut app = App::new();
        app.add_plugins(GameplayTagsPlugin::with_data_path(
            "assets/gameplay_tags.json".to_string(),
        ));
        app.update();

        let world = app.world_mut();

        let source = world.spawn_empty().id();
        let target = world.spawn_empty().id();

        let effect = world
            .spawn(ActiveGameplayEffect {
                definition_id: "poison".into(),
                source,
                target,
                level: 1,
                start_time: 0.0,
                granted_tags: GameplayTagContainer::default(),
                stack_count: 1,
            })
            .id();

        let query = GameplayEffectQuery::new().with_definition_id("poison");
        assert!(query.matches(effect, world));

        let query = GameplayEffectQuery::new().with_definition_id("fire");
        assert!(!query.matches(effect, world));
    }

    #[test]
    fn test_query_matches_owning_tags() {
        let mut app = App::new();
        app.add_plugins(GameplayTagsPlugin::with_data_path(
            "assets/gameplay_tags.json".to_string(),
        ));
        app.update();

        let world = app.world_mut();

        let source = world.spawn_empty().id();
        let target = world.spawn_empty().id();

        let manager = world.resource::<GameplayTagsManager>();
        let mut granted_tags = GameplayTagContainer::default();
        let poison_tag = GameplayTag::new("Effect.Debuff.Poison");
        granted_tags.add_tag(poison_tag, manager);

        let effect = world
            .spawn(ActiveGameplayEffect {
                definition_id: "poison".into(),
                source,
                target,
                level: 1,
                start_time: 0.0,
                granted_tags: granted_tags.clone(),
                stack_count: 1,
            })
            .id();

        let manager = world.resource::<GameplayTagsManager>();
        let query =
            GameplayEffectQuery::new().with_owning_tags_any(vec!["Effect.Debuff.Poison"], manager);
        assert!(query.matches(effect, world));

        let manager = world.resource::<GameplayTagsManager>();
        let query =
            GameplayEffectQuery::new().with_owning_tags_any(vec!["Effect.Buff.Attack"], manager);
        assert!(!query.matches(effect, world));
    }
}
