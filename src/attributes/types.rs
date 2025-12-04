use std::any::TypeId;

use bevy::{ecs::component::Component, reflect::Reflect};
use string_cache::DefaultAtom as FName;

#[derive(Debug)]
pub struct GameplayAttributeData {
    base_value: f32,
    current_value: f32,
}

impl Default for GameplayAttributeData {
    fn default() -> Self {
        GameplayAttributeData::new(0.0)
    }
}

impl GameplayAttributeData {
    pub fn new(value: f32) -> Self {
        GameplayAttributeData {
            base_value: value,
            current_value: value,
        }
    }

    pub fn set_base_value(&mut self, value: f32) {
        self.base_value = value;
    }

    pub fn set_current_value(&mut self, value: f32) {
        self.current_value = value;
    }

    pub fn get_base_value(&self) -> f32 {
        self.base_value
    }

    pub fn get_current_value(&self) -> f32 {
        self.current_value
    }
}

#[derive(Debug)]
pub struct GameplayAttribute {
    attribute: GameplayAttributeData,
    attribute_name: FName,
}

impl GameplayAttribute {
    pub fn new(attribute_name: FName) -> Self {
        GameplayAttribute {
            attribute: GameplayAttributeData::default(),
            attribute_name,
        }
    }

    pub fn with_value(value: GameplayAttributeData, attribute_name: FName) -> Self {
        GameplayAttribute {
            attribute: value,
            attribute_name,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub struct AttributeSetId(TypeId);

impl AttributeSetId {
    pub fn of<T: 'static>() -> Self {
        Self(TypeId::of::<T>())
    }
}

pub trait AttributeSet: Component {
    /// 获取属性的唯一标识符
    fn attribute_set_id() -> AttributeSetId
    where
        Self: Sized;

    fn get_attribute(&self, name: FName) -> GameplayAttribute;

    /// 在修改 current_value 之前调用,可以修改即将设置的新值
    /// 用于值的限制(clamping),例如确保 Health 不超过 MaxHealth
    fn pre_attribute_change(&self, atrribute: GameplayAttribute, _new_value: &mut f32) {}

    /// 在修改 current_value 之后调用
    /// 可以触发游戏逻辑相关的事件或回调
    fn post_attribute_change(
        &self,
        atrribute: GameplayAttribute,
        _old_value: f32,
        _new_value: f32,
    ) {
    }

    /// 在修改 base_value 之前调用,可以修改即将设置的新值
    /// 应该只用于值的限制,不应该触发游戏逻辑事件
    fn pre_attribute_base_change(&self, atrribute: GameplayAttribute, _new_value: &mut f32) {}

    /// 在修改 base_value 之后调用
    fn post_attribute_base_change(
        &self,
        atrribute: GameplayAttribute,
        _old_value: f32,
        _new_value: f32,
    ) {
    }
}
