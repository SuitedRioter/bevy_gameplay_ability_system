use crate::gameplay_ability::components::{GameplayAbilityActorInfo, GameplayAbilitySpec};
use crate::gameplay_ability::states::{AbilityPreActivating, AbilityWaitingActivation};
use bevy::prelude::{Commands, Entity, Query, With};

#[expect(dead_code)]
fn can_activate_ability(
    mut commands: Commands,
    ability_query: Query<
        (Entity, &GameplayAbilitySpec, &GameplayAbilityActorInfo),
        With<AbilityWaitingActivation>,
    >,
) {
    // TODO: check ability activation
    // 移除状态标记，添加下一步标记（但是要在允许激活的情况下）
    for (ability_entity, ability_spec, ability_actor_info) in &ability_query {
        if !check_cooldown(ability_spec, ability_actor_info) {
            continue;
        }
        if !check_cost(ability_spec, ability_actor_info) {
            continue;
        }
        commands
            .entity(ability_entity)
            .remove::<AbilityWaitingActivation>()
            .insert(AbilityPreActivating);
    }
}

fn check_cooldown(
    ability_spec: &GameplayAbilitySpec,
    ability_actor_info: &GameplayAbilityActorInfo,
) -> bool {
    true
}

fn check_cost(
    ability_spec: &GameplayAbilitySpec,
    ability_actor_info: &GameplayAbilityActorInfo,
) -> bool {
    true
}
