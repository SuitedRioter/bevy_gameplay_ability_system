use crate::attributes::AttributePlugin;
use crate::gameplay_ability::GameplayAbilityPlugin;
use crate::gameplay_ability_targeting::GameplayAbilityTargetingPlugin;
use crate::gameplay_effect::GameplayEffectPlugin;
use bevy::app::{App, Plugin};
use bevy_gameplay_tag::gameplay_tags_plugin::GameplayTagsPlugin;

pub struct GameplayAbilitySystemPlugin;

impl Plugin for GameplayAbilitySystemPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            GameplayTagsPlugin,
            AttributePlugin,
            GameplayEffectPlugin,
            GameplayAbilityPlugin,
            GameplayAbilityTargetingPlugin,
        ));
    }
}
