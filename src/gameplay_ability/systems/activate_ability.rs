use crate::gameplay_ability::components::GameplayAbilitySpec;
use crate::gameplay_ability::states::{AbilityActivated, AbilityPreActivating};
use bevy::prelude::{Commands, Entity, Query, With};

#[expect(dead_code)]
fn call_activate_ability(
    mut commands: Commands,
    ability_query: Query<(Entity, &GameplayAbilitySpec), With<AbilityPreActivating>>,
) {
    for (ability_entity, ability_spec) in &ability_query {
        // TODO: pre的逻辑

        commands
            .entity(ability_entity)
            .remove::<AbilityPreActivating>()
            .insert(AbilityActivated);
    }
}
