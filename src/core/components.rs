//! Core components used across the GAS system.

use bevy::prelude::*;
use bevy_gameplay_tag::gameplay_tag_count_container::GameplayTagCountContainer;

/// 玩家拥有的标签(来自能力、效果等)
#[derive(Component, Debug, Default)]
pub struct OwnedTags(pub GameplayTagCountContainer);
