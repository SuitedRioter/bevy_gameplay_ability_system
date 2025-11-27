use std::any::TypeId;

use bevy::prelude::Component;
use bevy::prelude::Reflect;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub struct GameplayAttributeId(TypeId);

impl GameplayAttributeId {
    pub fn of<T: 'static>() -> Self {
        Self(TypeId::of::<T>())
    }
}

/// 提供了属性组件base_value,current_value修改的pre，post方法。
/// 可根据需要自行实现业务逻辑
pub trait GameplayAttribute: Component {
    /// 获取属性的唯一标识符
    fn attribute_id() -> GameplayAttributeId
    where
        Self: Sized;
    /// 获取base_value
    fn get_base_value(&self) -> f32;
    /// 获取current_value
    fn get_current_value(&self) -> f32;
    /// 设置current_value
    fn set_current_value_internal(&mut self, value: f32);
    /// 设置base_value
    fn set_base_value_internal(&mut self, value: f32);

    // 默认实现 - 包含 hook 调用逻辑
    fn set_current_value(&mut self, value: f32) {
        let old_value = self.get_current_value();
        let mut new_value = value;
        self.pre_attribute_change(old_value, &mut new_value);
        self.set_current_value_internal(new_value);
        self.post_attribute_change(old_value, new_value);
    }

    fn set_base_value(&mut self, value: f32) {
        let old_value = self.get_base_value();
        let mut new_value = value;
        self.pre_attribute_base_change(old_value, &mut new_value);
        self.set_base_value_internal(new_value);
        self.post_attribute_base_change(old_value, new_value);
    }

    /// 在修改 current_value 之前调用,可以修改即将设置的新值
    /// 用于值的限制(clamping),例如确保 Health 不超过 MaxHealth
    fn pre_attribute_change(&mut self, _old_value: f32, _new_value: &mut f32) {}

    /// 在修改 current_value 之后调用
    /// 可以触发游戏逻辑相关的事件或回调
    fn post_attribute_change(&mut self, _old_value: f32, _new_value: f32) {}

    /// 在修改 base_value 之前调用,可以修改即将设置的新值
    /// 应该只用于值的限制,不应该触发游戏逻辑事件
    fn pre_attribute_base_change(&mut self, _old_value: f32, _new_value: &mut f32) {}

    /// 在修改 base_value 之后调用
    fn post_attribute_base_change(&mut self, _old_value: f32, _new_value: f32) {}
}

#[derive(Component, Debug, Clone, PartialEq, Reflect)]
pub struct AttributeSet;
