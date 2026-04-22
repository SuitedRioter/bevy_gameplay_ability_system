//! Custom magnitude calculation system.
//!
//! Allows users to implement complex magnitude calculations that can capture
//! multiple attributes from source and target entities.

use bevy::prelude::*;
use bevy::ecs::relationship::Relationship;
use std::collections::HashMap;
use string_cache::DefaultAtom as Atom;

use crate::attributes::{AttributeData, AttributeName};

/// Context passed to custom magnitude calculators.
///
/// Provides access to source and target entities for attribute queries.
#[derive(Debug)]
pub struct CalculationContext<'w> {
    /// The entity applying the effect (instigator).
    pub source: Option<Entity>,
    /// The entity receiving the effect.
    pub target: Entity,
    /// The level of the effect.
    pub level: i32,
    /// Query for reading attributes.
    pub attributes: &'w Query<
        'w,
        'w,
        (
            &'static AttributeData,
            &'static AttributeName,
            &'static ChildOf,
        ),
    >,
}

impl<'w> CalculationContext<'w> {
    /// Gets an attribute value from the source entity.
    ///
    /// Returns None if source doesn't exist or attribute not found.
    pub fn get_source_attribute(&self, attribute_name: &Atom) -> Option<f32> {
        let source = self.source?;
        self.attributes
            .iter()
            .find(
                |(_, name, parent): &(&AttributeData, &AttributeName, &ChildOf)| {
                    parent.get() == source && name.0 == *attribute_name
                },
            )
            .map(|(data, _, _)| data.current_value)
    }

    /// Gets an attribute value from the target entity.
    pub fn get_target_attribute(&self, attribute_name: &Atom) -> Option<f32> {
        self.attributes
            .iter()
            .find(
                |(_, name, parent): &(&AttributeData, &AttributeName, &ChildOf)| {
                    parent.get() == self.target && name.0 == *attribute_name
                },
            )
            .map(|(data, _, _)| data.current_value)
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
/// }
/// ```
pub trait CustomMagnitudeCalculation: Send + Sync {
    /// Calculates the magnitude based on the context.
    fn calculate(&self, ctx: &CalculationContext) -> f32;
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
