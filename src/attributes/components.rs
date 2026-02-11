//! Attribute system components.
//!
//! This module defines the core components for the attribute system, following
//! a pure ECS architecture where each attribute is a separate entity.

use bevy::prelude::*;

/// Single attribute data component.
///
/// Each attribute has a base value and a current value. The base value is the
/// permanent value, while the current value is the result of applying all
/// modifiers (from gameplay effects).
///
/// # Example
/// ```
/// # use bevy::prelude::*;
/// # use bevy_gameplay_ability_system::attributes::AttributeData;
/// let health = AttributeData {
///     base_value: 100.0,
///     current_value: 100.0,
/// };
/// ```
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct AttributeData {
    /// The base value of the attribute (permanent).
    pub base_value: f32,
    /// The current value after applying all modifiers.
    pub current_value: f32,
}

impl AttributeData {
    /// Creates a new attribute with the given base value.
    ///
    /// The current value is initialized to the base value.
    pub fn new(base_value: f32) -> Self {
        Self {
            base_value,
            current_value: base_value,
        }
    }

    /// Sets the base value and updates the current value.
    ///
    /// This should be used for permanent changes to the attribute.
    pub fn set_base_value(&mut self, value: f32) {
        self.base_value = value;
        self.current_value = value;
    }
}

/// Metadata for an attribute.
///
/// This defines the constraints and properties of an attribute type.
#[derive(Debug, Clone, PartialEq)]
pub struct AttributeMetadata {
    /// The name of the attribute (e.g., "Health", "Mana").
    pub name: &'static str,
    /// Minimum allowed value (if any).
    pub min_value: Option<f32>,
    /// Maximum allowed value (if any).
    pub max_value: Option<f32>,
}

impl AttributeMetadata {
    /// Creates new attribute metadata.
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            min_value: None,
            max_value: None,
        }
    }

    /// Sets the minimum value constraint.
    pub fn with_min(mut self, min: f32) -> Self {
        self.min_value = Some(min);
        self
    }

    /// Sets the maximum value constraint.
    pub fn with_max(mut self, max: f32) -> Self {
        self.max_value = Some(max);
        self
    }

    /// Clamps a value to the attribute's constraints.
    pub fn clamp(&self, value: f32) -> f32 {
        let mut result = value;
        if let Some(min) = self.min_value {
            result = result.max(min);
        }
        if let Some(max) = self.max_value {
            result = result.min(max);
        }
        result
    }
}

/// Component that stores the metadata for an attribute.
///
/// This is attached to attribute entities to define their constraints.
#[derive(Component, Debug, Clone, PartialEq)]
pub struct AttributeMetadataComponent(pub AttributeMetadata);

/// Component that links an attribute to its owner entity.
///
/// This creates a relationship between an attribute entity and the entity that owns it.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttributeOwner(pub Entity);

/// Component that identifies which attribute this entity represents.
///
/// This stores the name/identifier of the attribute for lookups.
#[derive(Component, Debug, Clone, PartialEq, Eq, Hash)]
pub struct AttributeName(pub String);

impl AttributeName {
    /// Creates a new attribute name.
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// Gets the attribute name as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attribute_data_new() {
        let attr = AttributeData::new(100.0);
        assert_eq!(attr.base_value, 100.0);
        assert_eq!(attr.current_value, 100.0);
    }

    #[test]
    fn test_attribute_data_set_base_value() {
        let mut attr = AttributeData::new(100.0);
        attr.set_base_value(150.0);
        assert_eq!(attr.base_value, 150.0);
        assert_eq!(attr.current_value, 150.0);
    }

    #[test]
    fn test_attribute_metadata_clamp() {
        let metadata = AttributeMetadata::new("Health")
            .with_min(0.0)
            .with_max(100.0);

        assert_eq!(metadata.clamp(-10.0), 0.0);
        assert_eq!(metadata.clamp(50.0), 50.0);
        assert_eq!(metadata.clamp(150.0), 100.0);
    }

    #[test]
    fn test_attribute_metadata_no_constraints() {
        let metadata = AttributeMetadata::new("Damage");
        assert_eq!(metadata.clamp(-100.0), -100.0);
        assert_eq!(metadata.clamp(1000.0), 1000.0);
    }
}
