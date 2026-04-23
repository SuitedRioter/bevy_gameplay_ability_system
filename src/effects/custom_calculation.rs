//! Custom magnitude calculation system.
//!
//! Allows users to implement complex magnitude calculations that can capture
//! multiple attributes from source and target entities.

use bevy::prelude::*;
use std::collections::HashMap;
use string_cache::DefaultAtom as Atom;

/// Context passed to custom magnitude calculators.
///
/// Provides captured attribute values from source and target entities.
#[derive(Debug, Clone)]
pub struct CalculationContext {
    /// The entity applying the effect (instigator).
    pub source: Option<Entity>,
    /// The entity receiving the effect.
    pub target: Entity,
    /// The level of the effect.
    pub level: i32,
    /// Captured attribute values from source entity.
    pub source_attributes: HashMap<Atom, f32>,
    /// Captured attribute values from target entity.
    pub target_attributes: HashMap<Atom, f32>,
}

impl CalculationContext {
    /// Gets an attribute value from the source entity.
    pub fn get_source_attribute(&self, attribute_name: &Atom) -> Option<f32> {
        self.source_attributes.get(attribute_name).copied()
    }

    /// Gets an attribute value from the target entity.
    pub fn get_target_attribute(&self, attribute_name: &Atom) -> Option<f32> {
        self.target_attributes.get(attribute_name).copied()
    }
}

/// Trait for custom magnitude calculations.
///
/// Implement this trait to create complex calculations that can capture
/// multiple attributes from source and target entities.
///
/// # Example
/// ```ignore
/// struct CriticalDamageCalculator;
///
/// impl CustomMagnitudeCalculation for CriticalDamageCalculator {
///     fn calculate(&self, ctx: &CalculationContext) -> f32 {
///         let base_damage = ctx.get_source_attribute(&"Attack".into()).unwrap_or(10.0);
///         let crit_chance = ctx.get_source_attribute(&"CritChance".into()).unwrap_or(0.0);
///         let crit_multiplier = ctx.get_source_attribute(&"CritMultiplier".into()).unwrap_or(1.5);
///
///         // Random crit calculation
///         if rand::random::<f32>() < crit_chance {
///             base_damage * crit_multiplier
///         } else {
///             base_damage
///         }
///     }
///
///     fn required_source_attributes(&self) -> &[&'static str] {
///         &["Attack", "CritChance", "CritMultiplier"]
///     }
///
///     fn required_target_attributes(&self) -> &[&'static str] {
///         &[]
///     }
/// }
/// ```
pub trait CustomMagnitudeCalculation: Send + Sync {
    /// Calculates the magnitude based on the context.
    fn calculate(&self, ctx: &CalculationContext) -> f32;

    /// Returns the list of source attributes this calculator needs.
    ///
    /// These attributes will be captured from the source entity before calling calculate().
    fn required_source_attributes(&self) -> &[&'static str] {
        &[]
    }

    /// Returns the list of target attributes this calculator needs.
    ///
    /// These attributes will be captured from the target entity before calling calculate().
    fn required_target_attributes(&self) -> &[&'static str] {
        &[]
    }
}

/// Registry for custom magnitude calculators.
///
/// Register your custom calculators at startup so they can be looked up
/// by name when evaluating CustomClass magnitude calculations.
///
/// # Example
/// ```ignore
/// fn setup_calculators(mut registry: ResMut<CustomCalculationRegistry>) {
///     registry.register("CriticalDamage", Box::new(CriticalDamageCalculator));
///     registry.register("ScaledHealing", Box::new(ScaledHealingCalculator));
/// }
/// ```
#[derive(Resource, Default)]
pub struct CustomCalculationRegistry {
    calculators: HashMap<Atom, Box<dyn CustomMagnitudeCalculation>>,
}

impl CustomCalculationRegistry {
    /// Registers a custom calculator.
    pub fn register(
        &mut self,
        name: impl Into<Atom>,
        calculator: Box<dyn CustomMagnitudeCalculation>,
    ) {
        self.calculators.insert(name.into(), calculator);
    }

    /// Gets a calculator by name.
    pub fn get(&self, name: &Atom) -> Option<&dyn CustomMagnitudeCalculation> {
        self.calculators.get(name).map(|b| b.as_ref())
    }
}
