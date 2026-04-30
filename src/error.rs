//! Unified error types for the Gameplay Ability System.
//!
//! This module provides a centralized error handling system for all GAS operations.

use bevy::prelude::*;
use std::fmt;
use string_cache::DefaultAtom as Atom;

/// Result type alias for GAS operations.
pub type GasResult<T> = Result<T, GasError>;

/// Unified error type for the Gameplay Ability System.
///
/// Provides structured error handling across all GAS subsystems.
#[derive(Debug, Clone, PartialEq)]
pub enum GasError {
    /// An attribute was not found on the target entity.
    AttributeNotFound {
        /// The name of the missing attribute.
        attribute_name: Atom,
        /// The entity that was expected to have the attribute.
        owner: Entity,
    },

    /// A gameplay effect definition was not found in the registry.
    EffectDefinitionNotFound {
        /// The ID of the missing effect definition.
        effect_id: Atom,
    },

    /// An ability definition was not found in the registry.
    AbilityDefinitionNotFound {
        /// The ID of the missing ability definition.
        ability_id: Atom,
    },

    /// An ability spec entity was not found or is invalid.
    AbilitySpecNotFound {
        /// The entity that was expected to be an ability spec.
        spec_entity: Entity,
    },

    /// Invalid state or operation.
    InvalidState {
        /// Description of the invalid state.
        message: String,
    },

    /// Missing required component on an entity.
    MissingComponent {
        /// The entity missing the component.
        entity: Entity,
        /// Name of the missing component type.
        component_name: &'static str,
    },

    /// Custom calculation not found in registry.
    CustomCalculationNotFound {
        /// The name of the missing calculator.
        calculator_name: Atom,
    },

    /// Application requirement not found in registry.
    ApplicationRequirementNotFound {
        /// The name of the missing requirement.
        requirement_name: Atom,
    },

    /// Effect application was blocked by requirements.
    EffectApplicationBlocked {
        /// The effect that was blocked.
        effect_id: Atom,
        /// Reason for blocking.
        reason: String,
    },

    /// Ability activation was blocked.
    AbilityActivationBlocked {
        /// The ability that was blocked.
        ability_id: Atom,
        /// Reason for blocking.
        reason: String,
    },
}

impl fmt::Display for GasError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GasError::AttributeNotFound {
                attribute_name,
                owner,
            } => write!(
                f,
                "Attribute '{}' not found on entity {:?}",
                attribute_name, owner
            ),
            GasError::EffectDefinitionNotFound { effect_id } => {
                write!(f, "Effect definition '{}' not found in registry", effect_id)
            }
            GasError::AbilityDefinitionNotFound { ability_id } => {
                write!(
                    f,
                    "Ability definition '{}' not found in registry",
                    ability_id
                )
            }
            GasError::AbilitySpecNotFound { spec_entity } => {
                write!(f, "Ability spec entity {:?} not found", spec_entity)
            }
            GasError::InvalidState { message } => write!(f, "Invalid state: {}", message),
            GasError::MissingComponent {
                entity,
                component_name,
            } => write!(
                f,
                "Entity {:?} is missing required component: {}",
                entity, component_name
            ),
            GasError::CustomCalculationNotFound { calculator_name } => write!(
                f,
                "Custom calculation '{}' not found in registry",
                calculator_name
            ),
            GasError::ApplicationRequirementNotFound { requirement_name } => write!(
                f,
                "Application requirement '{}' not found in registry",
                requirement_name
            ),
            GasError::EffectApplicationBlocked { effect_id, reason } => {
                write!(f, "Effect '{}' blocked: {}", effect_id, reason)
            }
            GasError::AbilityActivationBlocked { ability_id, reason } => {
                write!(f, "Ability '{}' blocked: {}", ability_id, reason)
            }
        }
    }
}

impl std::error::Error for GasError {}

/// Extension trait for converting Options to GasResults.
pub trait GasResultExt<T> {
    /// Converts an Option to a Result with a GasError.
    fn ok_or_gas(self, error: GasError) -> GasResult<T>;
}

impl<T> GasResultExt<T> for Option<T> {
    fn ok_or_gas(self, error: GasError) -> GasResult<T> {
        self.ok_or(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let error = GasError::AttributeNotFound {
            attribute_name: Atom::from("Health"),
            owner: Entity::from_bits(1),
        };
        let display = format!("{}", error);
        assert!(display.contains("Health"));
        assert!(display.contains("not found"));
    }

    #[test]
    fn test_effect_not_found_error() {
        let error = GasError::EffectDefinitionNotFound {
            effect_id: Atom::from("damage"),
        };
        let display = format!("{}", error);
        assert!(display.contains("damage"));
        assert!(display.contains("not found"));
    }

    #[test]
    fn test_invalid_state_error() {
        let error = GasError::InvalidState {
            message: "Cannot activate ability while stunned".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("Invalid state"));
        assert!(display.contains("stunned"));
    }

    #[test]
    fn test_gas_result_ext() {
        let some_value: Option<i32> = Some(42);
        let result = some_value.ok_or_gas(GasError::InvalidState {
            message: "test".to_string(),
        });
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);

        let none_value: Option<i32> = None;
        let result = none_value.ok_or_gas(GasError::InvalidState {
            message: "test".to_string(),
        });
        assert!(result.is_err());
    }
}
