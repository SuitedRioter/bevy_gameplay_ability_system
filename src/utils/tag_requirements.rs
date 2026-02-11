//! Tag requirement checking utilities.
//!
//! This module provides utilities for checking gameplay tag requirements,
//! which are used throughout the GAS system for activation conditions.

use bevy_gameplay_tag::gameplay_tag::GameplayTag;
use bevy_gameplay_tag::gameplay_tag_count_container::GameplayTagCountContainer;

/// Requirements for gameplay tags.
///
/// This structure defines which tags must be present, which must be absent,
/// and how many of each are required.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TagRequirements {
    /// Tags that must be present (any of these).
    pub require_tags: Vec<GameplayTag>,
    /// Tags that must not be present (none of these).
    pub ignore_tags: Vec<GameplayTag>,
    /// Tags where at least one must be present.
    pub require_any_tags: Vec<GameplayTag>,
    /// Tags where all must be present.
    pub require_all_tags: Vec<GameplayTag>,
}

impl TagRequirements {
    /// Creates a new empty tag requirements.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a required tag (any of these must be present).
    pub fn require_tag(mut self, tag: GameplayTag) -> Self {
        self.require_tags.push(tag);
        self
    }

    /// Adds an ignored tag (this must not be present).
    pub fn ignore_tag(mut self, tag: GameplayTag) -> Self {
        self.ignore_tags.push(tag);
        self
    }

    /// Adds a tag to the "require any" list.
    pub fn require_any_tag(mut self, tag: GameplayTag) -> Self {
        self.require_any_tags.push(tag);
        self
    }

    /// Adds a tag to the "require all" list.
    pub fn require_all_tag(mut self, tag: GameplayTag) -> Self {
        self.require_all_tags.push(tag);
        self
    }

    /// Checks if the given tag container satisfies these requirements.
    pub fn are_requirements_met(&self, tags: &GameplayTagCountContainer) -> bool {
        // Check ignore tags - if any are present, fail
        for ignore_tag in &self.ignore_tags {
            if tags.has_matching_gameplay_tag(ignore_tag) {
                return false;
            }
        }

        // Check require tags - at least one must be present (if any specified)
        if !self.require_tags.is_empty() {
            let mut has_any = false;
            for require_tag in &self.require_tags {
                if tags.has_matching_gameplay_tag(require_tag) {
                    has_any = true;
                    break;
                }
            }
            if !has_any {
                return false;
            }
        }

        // Check require_any_tags - at least one must be present (if any specified)
        if !self.require_any_tags.is_empty() {
            let mut has_any = false;
            for require_tag in &self.require_any_tags {
                if tags.has_matching_gameplay_tag(require_tag) {
                    has_any = true;
                    break;
                }
            }
            if !has_any {
                return false;
            }
        }

        // Check require_all_tags - all must be present
        for require_tag in &self.require_all_tags {
            if !tags.has_matching_gameplay_tag(require_tag) {
                return false;
            }
        }

        true
    }

    /// Returns true if there are no requirements.
    pub fn is_empty(&self) -> bool {
        self.require_tags.is_empty()
            && self.ignore_tags.is_empty()
            && self.require_any_tags.is_empty()
            && self.require_all_tags.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_requirements() {
        let requirements = TagRequirements::new();
        let tags = GameplayTagCountContainer::default();

        assert!(requirements.is_empty());
        assert!(requirements.are_requirements_met(&tags));
    }

    #[test]
    fn test_require_tag() {
        let requirements = TagRequirements::new().require_tag(GameplayTag::new("State.Alive"));

        let tags = GameplayTagCountContainer::default();

        // Should fail without the tag
        assert!(!requirements.are_requirements_met(&tags));

        // Should succeed with the tag (if we could add it)
        // Note: GameplayTagCountContainer doesn't expose add methods in the current API
    }

    #[test]
    fn test_ignore_tag() {
        let requirements = TagRequirements::new().ignore_tag(GameplayTag::new("State.Dead"));

        let tags = GameplayTagCountContainer::default();

        // Should succeed when the ignored tag is not present
        assert!(requirements.are_requirements_met(&tags));
    }

    #[test]
    fn test_builder_pattern() {
        let requirements = TagRequirements::new()
            .require_tag(GameplayTag::new("State.Alive"))
            .ignore_tag(GameplayTag::new("State.Stunned"))
            .require_all_tag(GameplayTag::new("Ability.CanCast"));

        assert!(!requirements.is_empty());
        assert_eq!(requirements.require_tags.len(), 1);
        assert_eq!(requirements.ignore_tags.len(), 1);
        assert_eq!(requirements.require_all_tags.len(), 1);
    }
}
