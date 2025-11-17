use crate::gameplay_ability::components::GameplayAbilitySpec;
use crate::gameplay_ability::states::{AbilityPreActivating, AbilityWaitingActivation};
use bevy::prelude::{Commands, Entity, Query, With};

fn can_activate_ability(
    mut commands: Commands,
    ability_query: Query<(Entity, &GameplayAbilitySpec), With<AbilityWaitingActivation>>,
) {
    // TODO: check ability activation
    // 移除状态标记，添加下一步标记（但是要在允许激活的情况下）
    for (ability_entity, ability_spec) in &ability_query {
        commands
            .entity(ability_entity)
            .remove::<AbilityWaitingActivation>()
            .insert(AbilityPreActivating);
    }
}
