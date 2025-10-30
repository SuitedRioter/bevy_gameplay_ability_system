// 自动实现 GameplayAttribute trait 的宏
#[macro_export]
macro_rules! define_attribute {
    // 完整参数：最小值、最大值、默认值
    ($name:ident, min = $min:expr, max = $max:expr, default = $default:expr) => {
        $crate::define_attribute_with_auto_impl!($name, $min, $max, $default);
    };

    // 最小值和最大值（默认值为最小值）
    ($name:ident, min = $min:expr, max = $max:expr) => {
        $crate::define_attribute_with_auto_impl!($name, $min, $max, $min);
    };

    // 只有最小值和默认值
    ($name:ident, min = $min:expr, default = $default:expr) => {
        $crate::define_attribute_with_auto_impl!($name, $min, f32::MAX, $default);
    };

    // 只有最大值和默认值
    ($name:ident, max = $max:expr, default = $default:expr) => {
        $crate::define_attribute_with_auto_impl!($name, f32::MIN, $max, $default);
    };

    // 只有默认值（无限制）
    ($name:ident, default = $default:expr) => {
        $crate::define_attribute_with_auto_impl!($name, f32::MIN, f32::MAX, $default);
    };

    // 只有最小值（默认值为最小值）
    ($name:ident, min = $min:expr) => {
        $crate::define_attribute_with_auto_impl!($name, $min, f32::MAX, $min);
    };

    // 只有最大值（默认值为 0.0）
    ($name:ident, max = $max:expr) => {
        $crate::define_attribute_with_auto_impl!($name, f32::MIN, $max, 0.0);
    };

    // 无限制（默认值为 0.0）
    ($name:ident) => {
        $crate::define_attribute_with_auto_impl!($name, f32::MIN, f32::MAX, 0.0);
    };
}

/// 不自动实现 GameplayAttribute trait 的宏
/// 需要用户手动实现 GameplayAttribute trait，否则会报错。
#[macro_export]
macro_rules! define_attribute_manual {
    // 完整参数：最小值、最大值、默认值
    ($name:ident, min = $min:expr, max = $max:expr, default = $default:expr) => {
        $crate::define_attribute_without_auto_impl!($name, $min, $max, $default);
    };

    // 最小值和最大值（默认值为最小值）
    ($name:ident, min = $min:expr, max = $max:expr) => {
        $crate::define_attribute_without_auto_impl!($name, $min, $max, $min);
    };

    // 只有最小值和默认值
    ($name:ident, min = $min:expr, default = $default:expr) => {
        $crate::define_attribute_without_auto_impl!($name, $min, f32::MAX, $default);
    };

    // 只有最大值和默认值
    ($name:ident, max = $max:expr, default = $default:expr) => {
        $crate::define_attribute_without_auto_impl!($name, f32::MIN, $max, $default);
    };

    // 只有默认值（无限制）
    ($name:ident, default = $default:expr) => {
        $crate::define_attribute_without_auto_impl!($name, f32::MIN, f32::MAX, $default);
    };

    // 只有最小值（默认值为最小值）
    ($name:ident, min = $min:expr) => {
        $crate::define_attribute_without_auto_impl!($name, $min, f32::MAX, $min);
    };

    // 只有最大值（默认值为 0.0）
    ($name:ident, max = $max:expr) => {
        $crate::define_attribute_without_auto_impl!($name, f32::MIN, $max, 0.0);
    };

    // 无限制（默认值为 0.0）
    ($name:ident) => {
        $crate::define_attribute_without_auto_impl!($name, f32::MIN, f32::MAX, 0.0);
    };
}

// 内部宏：自动实现 GameplayAttribute trait
#[macro_export]
macro_rules! define_attribute_with_auto_impl {
    ($name:ident, $min:expr, $max:expr, $default:expr) => {
        // 生成结构体和基本方法
        $crate::define_attribute_core!($name, $min, $max, $default);

        // 自动实现空 trait
        impl $crate::attributes::core::GameplayAttribute for $name {}
    };
}

// 内部宏：不自动实现 GameplayAttribute trait
#[macro_export]
macro_rules! define_attribute_without_auto_impl {
    ($name:ident, $min:expr, $max:expr, $default:expr) => {
        // 只生成结构体和基本方法，不实现 trait
        $crate::define_attribute_core!($name, $min, $max, $default);
    };
}

// 核心宏：生成结构体和基本方法（不包含 trait 实现）
#[macro_export]
macro_rules! define_attribute_core {
    (
        $name:ident,
        $min:expr,
        $max:expr,
        $default:expr
    ) => {
        #[derive(Component, Debug, Clone, Copy, PartialEq, Reflect)]
        pub struct $name {
            pub base_value: f32,
            pub current_value: f32,
        }

        impl $name {
            /// 使用默认值创建
            pub fn new() -> Self {
                let default_value = ($default as f32).clamp($min, $max);
                Self {
                    base_value: default_value,
                    current_value: default_value,
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

            /// 获取默认值
            pub const fn default_value() -> f32 {
                $default
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
