use bevy::ecs::entity::Entity;
use bevy::ecs::event::{EntityEvent, Event};

use crate::attributes::types::{AttributeSet, GameplayAttribute};
use crate::gameplay_effect::components::GameplayEffectSpec;
use crate::gameplay_effect::types::GameplayModOp;

///当游戏效果被移除时触发的事件委托。
#[derive(EntityEvent, Debug)]
pub struct OnEffectRemoved {
    pub entity: Entity,
}

///当游戏效果的堆叠数量发生变化时触发的事件委托。
#[derive(EntityEvent, Debug)]
pub struct OnStackChanged {
    pub entity: Entity,
}

///当游戏效果的时间（开始时间或持续时间）发生变化时触发的事件委托。
#[derive(EntityEvent, Debug)]
pub struct OnTimeChanged {
    pub entity: Entity,
}

///当游戏效果的抑制状态发生变化时触发的事件委托。
#[derive(EntityEvent, Debug)]
pub struct OnInhibitionChanged {
    pub entity: Entity,
}

#[derive(Event, Debug)]
pub struct OnAddModifierEvaluatedData<T: AttributeSet> {
    pub attribute: GameplayAttribute,
    pub modifier_op: GameplayModOp,
    pub magnitude: f32,
    pub effect_spec: GameplayEffectSpec,
    pub attribute_set: T,
}
