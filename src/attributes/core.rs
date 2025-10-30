use bevy::prelude::Component;
use bevy::prelude::Reflect;

#[derive(Component, Debug, Clone, PartialEq, Reflect)]
pub struct AttributeSet;

/// 提供了属性组件base_value,current_value修改的pre，post方法。
/// 可根据需要自行实现业务逻辑
pub trait GameplayAttribute {
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
