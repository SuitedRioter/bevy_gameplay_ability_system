use crate::gameplay_ability::components::GameplayAbilitySpec;
use crate::gameplay_ability::states::{AbilityActivated, AbilityEnding};
use bevy::prelude::{Commands, Entity, Query, With};

#[expect(dead_code)]
fn commit_ability(
    mut commands: Commands,
    ability_query: Query<(Entity, &GameplayAbilitySpec), With<AbilityActivated>>,
) {
    for (ability_entity, ability_spec) in &ability_query {
        commands
            .entity(ability_entity)
            .remove::<AbilityActivated>()
            .insert(AbilityEnding);
    }
}
