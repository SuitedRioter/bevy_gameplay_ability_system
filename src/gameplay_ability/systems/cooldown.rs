use crate::gameplay_ability::components::GameplayAbilitySpec;
use crate::gameplay_ability::states::AbilityCooldown;
use bevy::prelude::{Commands, Entity, Query, With};

#[expect(dead_code)]
fn ability_cooldown(
    mut commands: Commands,
    ability_query: Query<(Entity, &GameplayAbilitySpec), With<AbilityCooldown>>,
) {
    for (ability_entity, ability_spec) in ability_query {
        commands.entity(ability_entity).remove::<AbilityCooldown>();
    }
}
