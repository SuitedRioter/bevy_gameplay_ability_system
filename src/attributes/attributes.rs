/// 后面可以通过build.rs模式读取json配置。然后生成这些代码，目前就手动硬编码写死。
use crate::define_attribute;
use bevy::prelude::Component;
use bevy::prelude::Reflect;

define_attribute!(Health, default = 100.0);
define_attribute!(MaxHealth, min = 0.0, max = 1000.0, default = 100.0);

#[derive(Component, Debug, Clone, PartialEq, Reflect)]
pub struct AttributeSet;
