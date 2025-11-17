use bevy::prelude::Component;

/// 激活了，但是等待检测激活条件，能力还未真正生效
/// 一般使用技能就是添加这个状态标记组件即可
#[derive(Component)]
pub struct AbilityWaitingActivation;

#[derive(Component)]
pub struct AbilityPreActivating;

#[derive(Component)]
pub struct AbilityActivated;

#[derive(Component)]
pub struct AbilityApplyingEffects;

#[derive(Component)]
pub struct AbilityEnding;

#[derive(Component)]
pub struct AbilityCooldown;
