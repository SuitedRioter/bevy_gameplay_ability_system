use crate::gameplay_ability::components::{GameplayAbilityActorInfo, GameplayAbilitySpec};
use crate::gameplay_ability::states::{AbilityActivated, AbilityPreActivating};
use bevy::ecs::system::Res;
use bevy::ecs::world::World;
use bevy::prelude::{Commands, Entity, Query, With};
use bevy_gameplay_tag::gameplay_tag_count_container::GameplayTagCountContainer;
use bevy_gameplay_tag::gameplay_tags_manager::GameplayTagsManager;

#[expect(dead_code)]
fn call_activate_ability(
    world: &mut World,
    commands: &mut Commands,
    tags_manager: &Res<GameplayTagsManager>,
    ability_query: Query<
        (Entity, &GameplayAbilitySpec, &GameplayAbilityActorInfo),
        With<AbilityPreActivating>,
    >,
) {
    for (ability_entity, ability_spec, actor_info) in &ability_query {
        // TODO: pre的逻辑
        commands
            .entity(ability_entity)
            .remove::<AbilityPreActivating>()
            .insert(AbilityActivated);
        //获得actor_info.owner的GameplayTagCountContainer组件
        if let Some(mut gameplay_tag_count_container) =
            world.get_mut::<GameplayTagCountContainer>(actor_info.owner)
        {
            gameplay_tag_count_container.update_tag_container_count(
                &ability_spec.ability.activation_owned_tags,
                1,
                tags_manager,
                commands,
                actor_info.owner,
            );
        } else {
            // 创建新的 GameplayTagCountContainer 并初始化，这个分支判断完全是防止使用者遗漏了给角色添加这个组件而加，后期可能要想下有没有更好的方式，不然每个地方都要判断了
            let mut new_container = GameplayTagCountContainer::default(); // 或使用适当的构造函数
            new_container.update_tag_container_count(
                &ability_spec.ability.activation_owned_tags,
                1,
                tags_manager,
                commands,
                actor_info.owner,
            );
            commands.entity(actor_info.owner).insert(new_container);
        }
    }
}
