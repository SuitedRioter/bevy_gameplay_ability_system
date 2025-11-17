use crate::gameplay_ability::components::GameplayAbilitySpec;
use crate::gameplay_ability::states::{AbilityCooldown, AbilityEnding};
use bevy::prelude::{Commands, Entity, Query, With};

fn end_ability(
    mut commands: Commands,
    ability_query: Query<(Entity, &GameplayAbilitySpec), With<AbilityEnding>>,
) {
    for (ability_entity, ability_spec) in &ability_query {
        commands
            .entity(ability_entity)
            .remove::<AbilityEnding>()
            .insert(AbilityCooldown);
    }
}
