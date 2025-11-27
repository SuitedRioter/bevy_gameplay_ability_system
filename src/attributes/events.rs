use bevy::ecs::{entity::Entity, event::Event};

use crate::attributes::core::GameplayAttributeId;

#[derive(Event, Debug, Clone)]
pub struct AttributeValueModifyEvent {
    pub target: Entity,
    pub attribute_id: GameplayAttributeId,
    pub magnitude: f32,
    pub target_base_value: bool, // true 修改 base_value，false 修改 current_value
}
