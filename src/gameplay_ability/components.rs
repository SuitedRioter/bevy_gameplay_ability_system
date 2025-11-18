use bevy::prelude::{Component, Entity};
use bevy_gameplay_tag::gameplay_tag_container::GameplayTagContainer;

#[derive(Component, Default)]
#[expect(dead_code)]
pub struct GameplayAbilitySpecContainer {
    #[entities]
    pub abilities: Vec<Entity>,
}

#[derive(Component, Default)]
#[expect(dead_code)]
pub struct GameplayAbilitySpec {
    pub level: i32,
    pub ability: GameplayAbility,
}

///存储能力系统需要的角色相关信息，后续需要考虑自动spawn，而不用让使用者来spawn。
///也就是使用者不用感知到这个组件。
#[derive(Component)]
pub struct GameplayAbilityActorInfo {
    /// 拥有能力系统的实体(Owner)
    #[entities]
    pub owner: Entity,

    /// 实际执行能力的实体(Avatar)
    /// 在某些情况下可能与 owner 不同
    #[entities]
    pub avatar: Option<Entity>,

    ///与此Actor关联的PlayerController，这通常是null！
    #[entities]
    pub controller: Option<Entity>,
}

#[derive(Clone, Debug, Default)]
#[expect(dead_code)]
pub struct GameplayAbility {
    pub cancel_abilities_with_tag: GameplayTagContainer,
    pub block_abilities_with_tag: GameplayTagContainer,
    pub activation_owned_tags: GameplayTagContainer,
    pub activation_required_tags: GameplayTagContainer,
    pub activation_blocked_tags: GameplayTagContainer,
    pub source_required_tags: GameplayTagContainer,
    pub source_blocked_tags: GameplayTagContainer,
    pub target_required_tags: GameplayTagContainer,
    pub target_blocked_tags: GameplayTagContainer,
}
