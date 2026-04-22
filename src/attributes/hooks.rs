//! Attribute lifecycle hooks.
//!
//! This module provides hooks for attribute modifications, inspired by UE's AttributeSet callbacks.
//!
//! UE has several hook types:
//! - PreGameplayEffectExecute/PostGameplayEffectExecute: Called when an effect modifies base value (instant effects)
//! - PreAttributeChange/PostAttributeChange: Called for any attribute modification (including aggregation)
//! - PreAttributeBaseChange/PostAttributeBaseChange: Called when base value changes with aggregator present
//!
//! Our implementation:
//! - pre_effect_execute/post_effect_execute: For instant effect application (modifies base value)
//! - pre_change/post_change: For aggregated current value changes
//! - pre_base_change/post_base_change: For base value changes (level-ups, permanent modifications)

use bevy::prelude::*;
use std::any::TypeId;
use std::collections::HashMap;
use string_cache::DefaultAtom as Atom;

/// Context for attribute modification.
///
/// Passed to hook functions to provide information about the change.
#[derive(Debug, Clone)]
pub struct AttributeModifyContext {
    /// The entity that owns the attribute.
    pub owner: Entity,
    /// The attribute entity being modified.
    pub attribute: Entity,
    /// The name of the attribute.
    pub attribute_name: Atom,
    /// The old value before modification.
    pub old_value: f32,
    /// The new value after modification (mutable in pre hooks for clamping).
    pub new_value: f32,
    /// The effect entity that caused this modification (if any).
    pub source_effect: Option<Entity>,
}

/// Lifecycle hook functions for an AttributeSet.
///
/// These hooks allow you to:
/// - Clamp values (in pre hooks by modifying context.new_value)
/// - Trigger gameplay events (in post hooks)
/// - Validate modifications (return false in pre_effect_execute to reject)
///
/// # Example
/// ```rust,ignore
/// fn pre_health_change(ctx: &mut AttributeModifyContext) {
///     // Clamp health between 0 and max_health
///     ctx.new_value = ctx.new_value.clamp(0.0, 100.0);
/// }
///
/// fn post_health_change(ctx: &AttributeModifyContext) {
///     if ctx.new_value <= 0.0 {
///         // Trigger death event
///     }
/// }
/// ```
#[derive(Clone, Copy)]
pub struct AttributeSetHooks {
    /// Called before an instant effect modifies an attribute's base value.
    /// Return false to reject the modification.
    /// Modify context.new_value to clamp/adjust the value.
    pub pre_effect_execute: fn(&mut AttributeModifyContext) -> bool,

    /// Called after an instant effect modifies an attribute's base value.
    /// Use this to trigger gameplay events based on the change.
    pub post_effect_execute: fn(&AttributeModifyContext),

    /// Called before the aggregated current value changes.
    /// This is called for any modification (effects, aggregation, etc).
    /// Modify context.new_value to clamp the final value.
    pub pre_change: fn(&mut AttributeModifyContext),

    /// Called after the aggregated current value changes.
    /// Use this to trigger events when the visible attribute value changes.
    pub post_change: fn(&AttributeModifyContext),

    /// Called before the base value changes (level-ups, permanent mods).
    /// Modify context.new_value to clamp the base value.
    pub pre_base_change: fn(&mut AttributeModifyContext),

    /// Called after the base value changes.
    pub post_base_change: fn(&AttributeModifyContext),
}

impl Default for AttributeSetHooks {
    fn default() -> Self {
        Self {
            pre_effect_execute: |_| true,
            post_effect_execute: |_| {},
            pre_change: |_| {},
            post_change: |_| {},
            pre_base_change: |_| {},
            post_base_change: |_| {},
        }
    }
}

/// Resource storing hooks per AttributeSet type.
///
/// Register hooks for your custom attribute sets using the TypeId.
///
/// # Example
/// ```rust,ignore
/// fn setup_hooks(mut hooks: ResMut<AttributeLifecycleHooks>) {
///     hooks.register(
///         TypeId::of::<CharacterAttributes>(),
///         AttributeSetHooks {
///             pre_change: clamp_health,
///             post_change: check_death,
///             ..Default::default()
///         }
///     );
/// }
/// ```
#[derive(Resource, Default)]
pub struct AttributeLifecycleHooks {
    hooks: HashMap<TypeId, AttributeSetHooks>,
}

impl AttributeLifecycleHooks {
    /// Registers hooks for an AttributeSet type.
    pub fn register(&mut self, type_id: TypeId, hooks: AttributeSetHooks) {
        self.hooks.insert(type_id, hooks);
    }

    /// Gets hooks for an AttributeSet type.
    pub fn get(&self, type_id: TypeId) -> Option<&AttributeSetHooks> {
        self.hooks.get(&type_id)
    }
}
