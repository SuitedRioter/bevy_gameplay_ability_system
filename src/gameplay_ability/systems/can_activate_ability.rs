use crate::gameplay_ability::components::{GameplayAbilityActorInfo, GameplayAbilitySpec};
use crate::gameplay_ability::states::{AbilityPreActivating, AbilityWaitingActivation};
use bevy::prelude::{Commands, Entity, Query, With};

///后面可能要加一个Option<TriggerEventData>组件查询，因为有可能是通过事件触发激活的技能
///另外这地方可能会改入参数，毕竟这里没有激活，不需要额外生成spec
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
        if !does_ability_satisfy_tag_requirements() {
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

fn does_ability_satisfy_tag_requirements() -> bool {
    true
}
