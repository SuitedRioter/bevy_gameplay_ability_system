use crate::attributes::AttributePlugin;
use crate::gameplay_ability::GameplayAbilityPlugin;
use crate::gameplay_effect::GameplayEffectPlugin;
use bevy::app::{PluginGroup, PluginGroupBuilder};

pub struct GameplayAbilitySystemPlugin;

impl PluginGroup for GameplayAbilitySystemPlugin {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(AttributePlugin)
            .add(GameplayEffectPlugin)
            .add(GameplayAbilityPlugin)
    }
}
