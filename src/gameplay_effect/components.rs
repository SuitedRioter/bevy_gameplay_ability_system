use bevy::ecs::{component::Component, entity::Entity};
use string_cache::DefaultAtom as FName;

use crate::{
    attributes::types::GameplayAttribute,
    gameplay_effect::types::{GameplayEffect, GameplayEffectBehavior, GameplayModOp},
};

#[derive(Component, Debug)]
#[component(storage = "SparseSet")]
pub struct Pending;

#[derive(Component, Debug)]
#[component(storage = "SparseSet")]
pub struct Applied;

#[derive(Component, Debug)]
#[component(storage = "SparseSet")]
pub struct GameplayEffectSpec {
    pub def: GameplayEffect,
}

#[derive(Component, Debug)]
#[expect(dead_code)]
pub struct ActiveEffectsContainer {
    pub active_effects: Vec<Entity>,
}

#[derive(Component, Debug)]
#[expect(dead_code)]
pub struct ActiveGameplayEffect {
    pub effect_spec: GameplayEffectSpec,
}

#[derive(Component, Debug)]
pub struct GameplayModifierEvaluatedData {
    pub attribute: GameplayAttribute,
    pub modifier_op: GameplayModOp,
    pub magnitude: f32,
}

impl Default for GameplayModifierEvaluatedData {
    fn default() -> Self {
        GameplayModifierEvaluatedData::new(
            GameplayAttribute::new(FName::default()),
            GameplayModOp::AddBase,
            0.0,
        )
    }
}

impl GameplayModifierEvaluatedData {
    pub fn new(attribute: GameplayAttribute, modifier_op: GameplayModOp, magnitude: f32) -> Self {
        Self {
            attribute,
            modifier_op,
            magnitude,
        }
    }
}
