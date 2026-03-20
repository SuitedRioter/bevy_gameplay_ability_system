//! Attribute lifecycle hooks.

use bevy::prelude::*;
use string_cache::DefaultAtom as Atom;
use std::any::TypeId;
use std::collections::HashMap;

/// Context for attribute modification.
#[derive(Debug, Clone)]
pub struct AttributeModifyContext {
    pub owner: Entity,
    pub attribute: Entity,
    pub attribute_name: Atom,
    pub old_value: f32,
    pub new_value: f32,
    pub source_effect: Option<Entity>,
}

/// Lifecycle hook functions for an AttributeSet.
#[derive(Clone, Copy)]
pub struct AttributeSetHooks {
    pub pre_change: fn(&mut AttributeModifyContext),
    pub post_change: fn(&AttributeModifyContext),
    pub pre_base_change: fn(&mut AttributeModifyContext),
    pub post_base_change: fn(&AttributeModifyContext),
}

/// Resource storing hooks per AttributeSet type.
#[derive(Resource, Default)]
pub struct AttributeLifecycleHooks {
    hooks: HashMap<TypeId, AttributeSetHooks>,
}

impl AttributeLifecycleHooks {
    pub fn register(&mut self, type_id: TypeId, hooks: AttributeSetHooks) {
        self.hooks.insert(type_id, hooks);
    }

    pub fn get(&self, type_id: TypeId) -> Option<&AttributeSetHooks> {
        self.hooks.get(&type_id)
    }
}
