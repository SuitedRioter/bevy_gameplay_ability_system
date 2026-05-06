//! Optimized batch aggregation for attribute modifiers.
//!
//! This module provides performance-optimized batch processing for attribute modifiers,
//! reducing the number of iterations and improving cache locality.

use bevy::prelude::*;
use std::collections::{BTreeMap, HashMap};
use string_cache::DefaultAtom as Atom;

use crate::effects::components::{AttributeModifier, EvaluationChannel, ModifierOperation};

/// Batch of modifiers targeting the same attribute.
///
/// Pre-groups modifiers by channel and operation for efficient evaluation.
#[derive(Debug, Default)]
pub struct ModifierBatch {
    /// Modifiers grouped by channel, then by operation.
    /// BTreeMap ensures channels are evaluated in order (Channel0 → Channel9).
    pub channels: BTreeMap<EvaluationChannel, ChannelModifiers>,
}

/// Modifiers within a single evaluation channel, pre-grouped by operation.
#[derive(Debug, Default)]
pub struct ChannelModifiers {
    /// Override modifiers (only first one is used).
    pub overrides: Vec<f32>,
    /// AddBase modifiers (sum).
    pub add_base: Vec<f32>,
    /// AddCurrent modifiers (sum).
    pub add_current: Vec<f32>,
    /// MultiplyAdditive modifiers (sum, then apply as 1 + sum).
    pub multiply_additive: Vec<f32>,
    /// MultiplyMultiplicative modifiers (compound, each applied as 1 + m).
    pub multiply_multiplicative: Vec<f32>,
}

impl ChannelModifiers {
    /// Adds a modifier to the appropriate operation bucket.
    #[inline]
    fn add_modifier(&mut self, operation: ModifierOperation, magnitude: f32) {
        match operation {
            ModifierOperation::Override => self.overrides.push(magnitude),
            ModifierOperation::AddBase => self.add_base.push(magnitude),
            ModifierOperation::AddCurrent => self.add_current.push(magnitude),
            ModifierOperation::MultiplyAdditive => self.multiply_additive.push(magnitude),
            ModifierOperation::MultiplyMultiplicative => {
                self.multiply_multiplicative.push(magnitude)
            }
        }
    }

    /// Evaluates all modifiers in this channel.
    ///
    /// Formula: ((input + AddBase + AddCurrent) * (1 + sum(MultiplyAdditive)) * prod(1 + MultiplyMultiplicative))
    ///
    /// Override short-circuits and returns immediately.
    #[inline]
    fn evaluate(&self, input: f32) -> f32 {
        // Check for Override first (short-circuit)
        if let Some(&override_value) = self.overrides.first() {
            return override_value;
        }

        let mut current = input;

        // Step 1: AddBase - sum all base additions
        if !self.add_base.is_empty() {
            current += self.add_base.iter().sum::<f32>();
        }

        // Step 2: AddCurrent - sum all current additions
        if !self.add_current.is_empty() {
            current += self.add_current.iter().sum::<f32>();
        }

        // Step 3: MultiplyAdditive - sum multipliers then apply: (1 + sum)
        if !self.multiply_additive.is_empty() {
            let additive_multiplier: f32 = self.multiply_additive.iter().sum();
            current *= 1.0 + additive_multiplier;
        }

        // Step 4: MultiplyMultiplicative - compound each multiplier: prod(1 + m)
        if !self.multiply_multiplicative.is_empty() {
            for &multiplier in &self.multiply_multiplicative {
                current *= 1.0 + multiplier;
            }
        }

        current
    }

    /// Returns true if this channel has any modifiers.
    #[inline]
    fn is_empty(&self) -> bool {
        self.overrides.is_empty()
            && self.add_base.is_empty()
            && self.add_current.is_empty()
            && self.multiply_additive.is_empty()
            && self.multiply_multiplicative.is_empty()
    }
}

impl ModifierBatch {
    /// Creates a new empty batch.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a modifier to the batch.
    #[inline]
    pub fn add_modifier(
        &mut self,
        channel: EvaluationChannel,
        operation: ModifierOperation,
        magnitude: f32,
    ) {
        self.channels
            .entry(channel)
            .or_insert_with(ChannelModifiers::default)
            .add_modifier(operation, magnitude);
    }

    /// Evaluates all modifiers in the batch, starting from the base value.
    ///
    /// Channels are evaluated in order (Channel0 → Channel9).
    /// The output of one channel becomes the input to the next.
    #[inline]
    pub fn evaluate(&self, base_value: f32) -> f32 {
        let mut current = base_value;

        for (_channel, channel_modifiers) in &self.channels {
            if !channel_modifiers.is_empty() {
                current = channel_modifiers.evaluate(current);
            }
        }

        current
    }

    /// Returns true if this batch has no modifiers.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.channels.is_empty()
    }

    /// Clears all modifiers from the batch.
    #[inline]
    pub fn clear(&mut self) {
        self.channels.clear();
    }
}

/// Key for grouping modifiers by target attribute.
///
/// Uses (owner_entity, attribute_name) as the unique identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AttributeKey {
    pub owner: Entity,
    pub attribute_name: Atom,
}

impl AttributeKey {
    #[inline]
    pub fn new(owner: Entity, attribute_name: Atom) -> Self {
        Self {
            owner,
            attribute_name,
        }
    }
}

/// Batch aggregator for collecting and processing modifiers efficiently.
///
/// This aggregator pre-groups modifiers by target attribute and channel,
/// reducing the number of iterations needed during evaluation.
#[derive(Debug, Default)]
pub struct ModifierAggregator {
    /// Map of attribute key to modifier batch.
    batches: HashMap<AttributeKey, ModifierBatch>,
}

impl ModifierAggregator {
    /// Creates a new empty aggregator.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a modifier to the aggregator.
    #[inline]
    pub fn add_modifier(&mut self, modifier: &AttributeModifier) {
        let key = AttributeKey::new(modifier.target_entity, modifier.target_attribute.clone());
        self.batches
            .entry(key)
            .or_insert_with(ModifierBatch::new)
            .add_modifier(modifier.channel, modifier.operation, modifier.magnitude);
    }

    /// Gets the batch for a specific attribute, if any.
    #[inline]
    pub fn get_batch(&self, owner: Entity, attribute_name: &Atom) -> Option<&ModifierBatch> {
        let key = AttributeKey::new(owner, attribute_name.clone());
        self.batches.get(&key)
    }

    /// Clears all batches.
    #[inline]
    pub fn clear(&mut self) {
        self.batches.clear();
    }

    /// Returns the number of unique attributes with modifiers.
    #[inline]
    pub fn len(&self) -> usize {
        self.batches.len()
    }

    /// Returns true if there are no modifiers.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.batches.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_modifiers_add_base() {
        let mut channel = ChannelModifiers::default();
        channel.add_modifier(ModifierOperation::AddBase, 10.0);
        channel.add_modifier(ModifierOperation::AddBase, 20.0);

        let result = channel.evaluate(100.0);
        assert_eq!(result, 130.0); // 100 + 10 + 20
    }

    #[test]
    fn test_channel_modifiers_multiply_additive() {
        let mut channel = ChannelModifiers::default();
        channel.add_modifier(ModifierOperation::MultiplyAdditive, 0.5); // +50%
        channel.add_modifier(ModifierOperation::MultiplyAdditive, 0.5); // +50%

        let result = channel.evaluate(100.0);
        assert_eq!(result, 200.0); // 100 * (1 + 0.5 + 0.5) = 100 * 2.0
    }

    #[test]
    fn test_channel_modifiers_multiply_multiplicative() {
        let mut channel = ChannelModifiers::default();
        channel.add_modifier(ModifierOperation::MultiplyMultiplicative, 0.5); // +50%
        channel.add_modifier(ModifierOperation::MultiplyMultiplicative, 0.5); // +50%

        let result = channel.evaluate(100.0);
        assert_eq!(result, 225.0); // 100 * 1.5 * 1.5 = 225
    }

    #[test]
    fn test_channel_modifiers_override() {
        let mut channel = ChannelModifiers::default();
        channel.add_modifier(ModifierOperation::Override, 50.0);
        channel.add_modifier(ModifierOperation::AddBase, 100.0); // Should be ignored

        let result = channel.evaluate(100.0);
        assert_eq!(result, 50.0); // Override short-circuits
    }

    #[test]
    fn test_channel_modifiers_full_formula() {
        let mut channel = ChannelModifiers::default();
        channel.add_modifier(ModifierOperation::AddBase, 10.0);
        channel.add_modifier(ModifierOperation::AddCurrent, 5.0);
        channel.add_modifier(ModifierOperation::MultiplyAdditive, 0.5); // +50%
        channel.add_modifier(ModifierOperation::MultiplyMultiplicative, 0.2); // +20%

        let result = channel.evaluate(100.0);
        // (100 + 10 + 5) * (1 + 0.5) * (1 + 0.2) = 115 * 1.5 * 1.2 = 207
        assert!((result - 207.0).abs() < 0.01); // Use epsilon comparison for floating point
    }

    #[test]
    fn test_modifier_batch_single_channel() {
        let mut batch = ModifierBatch::new();
        batch.add_modifier(
            EvaluationChannel::Channel0,
            ModifierOperation::AddBase,
            10.0,
        );
        batch.add_modifier(
            EvaluationChannel::Channel0,
            ModifierOperation::AddBase,
            20.0,
        );

        let result = batch.evaluate(100.0);
        assert_eq!(result, 130.0);
    }

    #[test]
    fn test_modifier_batch_multiple_channels() {
        let mut batch = ModifierBatch::new();
        // Channel0: +10
        batch.add_modifier(
            EvaluationChannel::Channel0,
            ModifierOperation::AddBase,
            10.0,
        );
        // Channel1: *1.5 (50% bonus)
        batch.add_modifier(
            EvaluationChannel::Channel1,
            ModifierOperation::MultiplyAdditive,
            0.5,
        );

        let result = batch.evaluate(100.0);
        // Channel0: 100 + 10 = 110
        // Channel1: 110 * 1.5 = 165
        assert_eq!(result, 165.0);
    }

    #[test]
    fn test_modifier_aggregator() {
        let mut aggregator = ModifierAggregator::new();

        let owner = Entity::from_bits(1);
        let attr_name = Atom::from("Health");

        let modifier1 = AttributeModifier {
            target_entity: owner,
            target_attribute: attr_name.clone(),
            channel: EvaluationChannel::Channel0,
            operation: ModifierOperation::AddBase,
            magnitude: 10.0,
            dynamic_magnitude: None,
        };

        let modifier2 = AttributeModifier {
            target_entity: owner,
            target_attribute: attr_name.clone(),
            channel: EvaluationChannel::Channel0,
            operation: ModifierOperation::AddBase,
            magnitude: 20.0,
            dynamic_magnitude: None,
        };

        aggregator.add_modifier(&modifier1);
        aggregator.add_modifier(&modifier2);

        let batch = aggregator.get_batch(owner, &attr_name).unwrap();
        let result = batch.evaluate(100.0);
        assert_eq!(result, 130.0);
    }

    #[test]
    fn test_modifier_aggregator_multiple_attributes() {
        let mut aggregator = ModifierAggregator::new();

        let owner = Entity::from_bits(1);
        let health = Atom::from("Health");
        let mana = Atom::from("Mana");

        let health_mod = AttributeModifier {
            target_entity: owner,
            target_attribute: health.clone(),
            channel: EvaluationChannel::Channel0,
            operation: ModifierOperation::AddBase,
            magnitude: 10.0,
            dynamic_magnitude: None,
        };

        let mana_mod = AttributeModifier {
            target_entity: owner,
            target_attribute: mana.clone(),
            channel: EvaluationChannel::Channel0,
            operation: ModifierOperation::AddBase,
            magnitude: 5.0,
            dynamic_magnitude: None,
        };

        aggregator.add_modifier(&health_mod);
        aggregator.add_modifier(&mana_mod);

        assert_eq!(aggregator.len(), 2);

        let health_batch = aggregator.get_batch(owner, &health).unwrap();
        assert_eq!(health_batch.evaluate(100.0), 110.0);

        let mana_batch = aggregator.get_batch(owner, &mana).unwrap();
        assert_eq!(mana_batch.evaluate(100.0), 105.0);
    }
}
