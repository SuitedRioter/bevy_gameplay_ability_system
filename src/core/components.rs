//! Core components used across the GAS system.

use bevy::prelude::*;
use bevy_gameplay_tag::gameplay_tag_count_container::GameplayTagCountContainer;

/// 玩家拥有的标签(来自能力、效果等)
#[derive(Component, Debug, Default)]
pub struct OwnedTags(pub GameplayTagCountContainer);

/// Component that stores tags blocking other abilities from activating.
///
/// Lives on the owner entity. Each active ability instance adds its
/// `block_abilities_with_tags` to this container during pre_activate,
/// and removes them during end.
#[derive(Component, Debug, Default)]
pub struct BlockedAbilityTags(pub GameplayTagCountContainer);
