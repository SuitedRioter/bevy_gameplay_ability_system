//! Conditional effect application system.
//!
//! Allows custom logic to determine whether an effect should be applied.

use crate::attributes::{AttributeData, AttributeName};
use crate::core::OwnedTags;
use bevy::prelude::*;
use std::collections::HashMap;
use string_cache::DefaultAtom as Atom;

/// Attribute value captured for application requirement checks.
#[derive(Debug, Clone)]
pub struct ApplicationAttributeSnapshot {
    pub owner: Entity,
    pub attribute_name: Atom,
    pub base_value: f32,
    pub current_value: f32,
}

impl ApplicationAttributeSnapshot {
    pub fn new(owner: Entity, attribute_name: &AttributeName, data: &AttributeData) -> Self {
        Self {
            owner,
            attribute_name: attribute_name.0.clone(),
            base_value: data.base_value,
            current_value: data.current_value,
        }
    }
}

/// Context passed to application requirement checks.
#[derive(Debug)]
pub struct ApplicationContext<'w> {
    /// The entity applying the effect (source).
    pub source: Option<Entity>,
    /// The entity receiving the effect (target).
    pub target: Entity,
    /// The level of the effect.
    pub level: i32,
    /// Query for reading target's tags.
    pub target_tags: Option<&'w OwnedTags>,
    /// Query for reading source's tags.
    pub source_tags: Option<&'w OwnedTags>,
    /// Snapshot of attributes for source/target checks.
    pub attributes: &'w [ApplicationAttributeSnapshot],
}

impl<'w> ApplicationContext<'w> {
    /// Gets an attribute value from the target entity.
    pub fn get_target_attribute(&self, attribute_name: &Atom) -> Option<f32> {
        self.attributes
            .iter()
            .find(|snapshot| {
                snapshot.owner == self.target && snapshot.attribute_name == *attribute_name
            })
            .map(|snapshot| snapshot.current_value)
    }

    /// Gets an attribute value from the source entity.
    pub fn get_source_attribute(&self, attribute_name: &Atom) -> Option<f32> {
        let source = self.source?;
        self.attributes
            .iter()
            .find(|snapshot| snapshot.owner == source && snapshot.attribute_name == *attribute_name)
            .map(|snapshot| snapshot.current_value)
    }
}

/// Trait for custom application requirements.
///
/// Implement this to add conditional logic for effect application.
///
/// # Example
/// ```ignore
/// struct HealthThresholdRequirement {
///     min_health_percent: f32,
/// }
///
/// impl ApplicationRequirement for HealthThresholdRequirement {
///     fn can_apply(&self, ctx: &ApplicationContext) -> bool {
///         let health = ctx.get_target_attribute(&"Health".into()).unwrap_or(0.0);
///         let max_health = ctx.get_target_attribute(&"MaxHealth".into()).unwrap_or(100.0);
///         (health / max_health) >= self.min_health_percent
///     }
/// }
/// ```
pub trait ApplicationRequirement: Send + Sync {
    /// Returns true if the effect can be applied.
    fn can_apply(&self, ctx: &ApplicationContext) -> bool;
}

/// Registry for application requirements.
///
/// Register custom requirements at startup so they can be looked up by name.
#[derive(Resource, Default)]
pub struct ApplicationRequirementRegistry {
    requirements: HashMap<Atom, Box<dyn ApplicationRequirement>>,
}

impl ApplicationRequirementRegistry {
    /// Registers a requirement.
    pub fn register(
        &mut self,
        name: impl Into<Atom>,
        requirement: Box<dyn ApplicationRequirement>,
    ) {
        self.requirements.insert(name.into(), requirement);
    }

    /// Gets a requirement by name.
    pub fn get(&self, name: &Atom) -> Option<&dyn ApplicationRequirement> {
        self.requirements.get(name).map(|b| b.as_ref())
    }
}
