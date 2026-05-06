//! Gameplay effect execution calculation system.
//!
//! Provides complex custom calculations that can capture multiple attributes
//! and execute custom logic, matching UE GAS's `UGameplayEffectExecutionCalculation`.

use bevy::prelude::*;
use std::collections::HashMap;
use std::fmt;
use string_cache::DefaultAtom as Atom;

use super::definition::AttributeCaptureSource;

/// Trait for custom gameplay effect execution calculations.
///
/// Execution calculations can capture multiple attributes from source/target
/// and perform complex calculations to produce modifier values.
///
/// Matches UE GAS's `UGameplayEffectExecutionCalculation`.
///
/// # Example
///
/// ```ignore
/// struct DamageCalculation;
///
/// impl GameplayEffectExecutionCalculation for DamageCalculation {
///     fn relevant_attributes_to_capture(&self) -> Vec<AttributeCaptureDefinition> {
///         vec![
///             AttributeCaptureDefinition {
///                 attribute_name: "AttackPower".into(),
///                 capture_source: AttributeCaptureSource::Source,
///                 snapshot: true,
///             },
///             AttributeCaptureDefinition {
///                 attribute_name: "Defense".into(),
///                 capture_source: AttributeCaptureSource::Target,
///                 snapshot: false,
///             },
///         ]
///     }
///
///     fn execute(
///         &self,
///         spec: &GameplayEffectSpec,
///         captured_attributes: &HashMap<Atom, f32>,
///         world: &World,
///     ) -> Vec<GameplayModifierEvaluatedData> {
///         let attack = captured_attributes.get(&"AttackPower".into()).copied().unwrap_or(0.0);
///         let defense = captured_attributes.get(&"Defense".into()).copied().unwrap_or(0.0);
///         let damage = (attack * 1.5 - defense * 0.5).max(0.0);
///
///         vec![GameplayModifierEvaluatedData {
///             attribute: "Health".into(),
///             modifier_op: ModifierOperation::AddCurrent,
///             magnitude: -damage,
///         }]
///     }
/// }
/// ```
pub trait GameplayEffectExecutionCalculation: Send + Sync + fmt::Debug {
    /// Defines which attributes need to be captured for this calculation.
    ///
    /// The system will capture these attributes before calling `execute()`.
    fn relevant_attributes_to_capture(&self) -> Vec<AttributeCaptureDefinition>;

    /// Executes the calculation and returns modifier data.
    ///
    /// # Parameters
    /// - `spec`: The gameplay effect spec being applied
    /// - `captured_attributes`: Map of attribute name to captured value
    /// - `world`: World access for additional queries
    ///
    /// # Returns
    /// Vector of evaluated modifiers to apply
    fn execute(
        &self,
        spec: &GameplayEffectSpec,
        captured_attributes: &HashMap<Atom, f32>,
        world: &World,
    ) -> Vec<GameplayModifierEvaluatedData>;
}

/// Defines an attribute to capture for execution calculations.
///
/// Matches UE GAS's `FGameplayEffectAttributeCaptureDefinition`.
#[derive(Debug, Clone)]
pub struct AttributeCaptureDefinition {
    /// Name of the attribute to capture
    pub attribute_name: Atom,
    /// Whether to capture from source or target
    pub capture_source: AttributeCaptureSource,
    /// Whether to snapshot the value at effect creation time
    pub snapshot: bool,
}

impl AttributeCaptureDefinition {
    /// Creates a new attribute capture definition.
    pub fn new(
        attribute_name: impl Into<Atom>,
        capture_source: AttributeCaptureSource,
        snapshot: bool,
    ) -> Self {
        Self {
            attribute_name: attribute_name.into(),
            capture_source,
            snapshot,
        }
    }

    /// Creates a snapshot capture from source.
    pub fn snapshot_source(attribute_name: impl Into<Atom>) -> Self {
        Self::new(attribute_name, AttributeCaptureSource::Source, true)
    }

    /// Creates a snapshot capture from target.
    pub fn snapshot_target(attribute_name: impl Into<Atom>) -> Self {
        Self::new(attribute_name, AttributeCaptureSource::Target, true)
    }

    /// Creates a dynamic capture from source.
    pub fn dynamic_source(attribute_name: impl Into<Atom>) -> Self {
        Self::new(attribute_name, AttributeCaptureSource::Source, false)
    }

    /// Creates a dynamic capture from target.
    pub fn dynamic_target(attribute_name: impl Into<Atom>) -> Self {
        Self::new(attribute_name, AttributeCaptureSource::Target, false)
    }
}

/// Evaluated modifier data produced by execution calculations.
///
/// This is the output of an execution calculation, specifying which
/// attribute to modify and by how much.
#[derive(Debug, Clone)]
pub struct GameplayModifierEvaluatedData {
    /// The attribute to modify
    pub attribute: Atom,
    /// The modifier operation to apply
    pub modifier_op: ModifierOperation,
    /// The calculated magnitude
    pub magnitude: f32,
}

impl GameplayModifierEvaluatedData {
    /// Creates new evaluated modifier data.
    pub fn new(attribute: impl Into<Atom>, modifier_op: ModifierOperation, magnitude: f32) -> Self {
        Self {
            attribute: attribute.into(),
            modifier_op,
            magnitude,
        }
    }
}

/// Re-export types needed by execution calculations
pub use super::components::{GameplayEffectSpec, ModifierOperation};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attribute_capture_definition_builders() {
        let snapshot_src = AttributeCaptureDefinition::snapshot_source("Attack");
        assert_eq!(snapshot_src.attribute_name, Atom::from("Attack"));
        assert!(matches!(
            snapshot_src.capture_source,
            AttributeCaptureSource::Source
        ));
        assert!(snapshot_src.snapshot);

        let dynamic_tgt = AttributeCaptureDefinition::dynamic_target("Defense");
        assert_eq!(dynamic_tgt.attribute_name, Atom::from("Defense"));
        assert!(matches!(
            dynamic_tgt.capture_source,
            AttributeCaptureSource::Target
        ));
        assert!(!dynamic_tgt.snapshot);
    }

    #[test]
    fn test_evaluated_data_creation() {
        let data =
            GameplayModifierEvaluatedData::new("Health", ModifierOperation::AddCurrent, -50.0);
        assert_eq!(data.attribute, Atom::from("Health"));
        assert!(matches!(data.modifier_op, ModifierOperation::AddCurrent));
        assert_eq!(data.magnitude, -50.0);
    }

    #[derive(Debug)]
    struct TestCalculation;

    impl GameplayEffectExecutionCalculation for TestCalculation {
        fn relevant_attributes_to_capture(&self) -> Vec<AttributeCaptureDefinition> {
            vec![
                AttributeCaptureDefinition::snapshot_source("AttackPower"),
                AttributeCaptureDefinition::dynamic_target("Defense"),
            ]
        }

        fn execute(
            &self,
            _spec: &GameplayEffectSpec,
            captured_attributes: &HashMap<Atom, f32>,
            _world: &World,
        ) -> Vec<GameplayModifierEvaluatedData> {
            let attack = captured_attributes
                .get(&Atom::from("AttackPower"))
                .copied()
                .unwrap_or(0.0);
            let defense = captured_attributes
                .get(&Atom::from("Defense"))
                .copied()
                .unwrap_or(0.0);
            let damage = (attack * 1.5 - defense * 0.5).max(0.0);

            vec![GameplayModifierEvaluatedData::new(
                "Health",
                ModifierOperation::AddCurrent,
                -damage,
            )]
        }
    }

    #[test]
    fn test_execution_calculation_trait() {
        let calc = TestCalculation;
        let captures = calc.relevant_attributes_to_capture();
        assert_eq!(captures.len(), 2);
        assert_eq!(captures[0].attribute_name, Atom::from("AttackPower"));
        assert_eq!(captures[1].attribute_name, Atom::from("Defense"));
    }
}
