// 自动实现 GameplayAttribute trait 的宏
#[macro_export]
macro_rules! define_attribute {
    ($name:ident, min = $min:expr, max = $max:expr) => {
        // 生成结构体和基本方法
        $crate::define_attribute_core!($name, $min, $max);

        // 自动实现空 trait
        impl $crate::attributes::core::GameplayAttribute for $name {}
    };
}

/// 不自动实现 GameplayAttribute trait 的宏
/// 需要用户手动实现 GameplayAttribute trait，否则会报错。
#[macro_export]
macro_rules! define_attribute_manual {
    ($name:ident, min = $min:expr, max = $max:expr) => {
        // 只生成结构体和基本方法，不实现 trait
        $crate::define_attribute_core!($name, $min, $max);
    };
}

// 核心宏：生成结构体和基本方法（不包含 trait 实现）
#[macro_export]
macro_rules! define_attribute_core {
    (
        $name:ident,
        $min:expr,
        $max:expr
    ) => {
        #[derive(Component, Debug, Clone, Copy, PartialEq, Reflect)]
        pub struct $name {
            pub base_value: f32,
            pub current_value: f32,
        }

        impl $name {
            /// 使用默认值创建
            pub fn new() -> Self {
                Self {
                    base_value: 0.0,
                    current_value: 0.0,
                }
            }

            /// 使用自定义值创建
            pub fn with_value(value: f32) -> Self {
                let clamped_value = value.clamp($min, $max);
                Self {
                    base_value: clamped_value,
                    current_value: clamped_value,
                }
            }

            pub fn get_base_value(&self) -> f32 {
                self.base_value
            }

            pub fn get_current_value(&self) -> f32 {
                self.current_value
            }

            pub fn set_current_value(&mut self, value: f32) {
                let old_value = self.current_value;
                let mut new_value = value;
                // 调用 pre hook,允许进一步修改 new_value
                self.pre_attribute_change(old_value, &mut new_value);
                // 再次 clamp 确保 pre hook 修改后的值仍在范围内
                new_value = new_value.clamp($min, $max);
                // 实际修改值
                self.current_value = new_value;
                // 调用 post hook
                self.post_attribute_change(old_value, new_value);
            }

            pub fn set_base_value(&mut self, value: f32) {
                let old_value = self.base_value;
                let mut new_value = value;
                // 调用 pre hook,允许进一步修改 new_value
                self.pre_attribute_base_change(old_value, &mut new_value);
                // 再次 clamp 确保 pre hook 修改后的值仍在范围内
                new_value = new_value.clamp($min, $max);
                // 实际修改值
                self.base_value = new_value;
                // 调用 post hook
                self.post_attribute_base_change(old_value, new_value);
            }

            /// 获取最小值
            pub const fn min_value() -> f32 {
                $min
            }

            /// 获取最大值
            pub const fn max_value() -> f32 {
                $max
            }

            /// 检查值是否在有效范围内
            pub fn is_valid_value(value: f32) -> bool {
                ($min..$max).contains(&value)
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        /// 实现 Display trait 用于用户友好的输出
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "base:{:.1}, current:{:.1}",
                    self.base_value, self.current_value
                )
            }
        }
    };
}
