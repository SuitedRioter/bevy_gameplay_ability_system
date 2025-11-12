use bevy::prelude::{Component, Entity};
use bevy_gameplay_tag::gameplay_tag_container::GameplayTagContainer;

#[derive(Component)]
pub struct GameplayAbilitySpecContainer{
    #[entities]
    pub abilities: Vec<Entity>,
}

#[derive(Component)]
pub struct GameplayAbilitySpec{
    pub level: i32,
    pub ability: GameplayAbility,
}

#[derive(Clone, Debug)]
pub struct GameplayAbility{
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