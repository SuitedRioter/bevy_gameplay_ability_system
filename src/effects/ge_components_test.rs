#[cfg(test)]
mod immunity_tests {
    use super::*;
    use crate::effects::definition::{DurationPolicy, GameplayEffectDefinition, ModifierInfo};
    use crate::effects::components::ModifierOperation;
    use crate::effects::definition::MagnitudeCalculation;
    use crate::effects::query::GameplayEffectQuery;
    use crate::effects::systems::ApplyGameplayEffectEvent;
    use crate::core::{BlockedAbilityTags, OwnedTags};
    use bevy_gameplay_tag::{GameplayTag, GameplayTagsManager, GameplayTagsPlugin};
    use std::sync::Arc;

    #[test]
    fn test_immunity_component_blocks_matching_effects() {
        let mut app = App::new();
        app.add_plugins(GameplayTagsPlugin::with_data_path(
            "assets/gameplay_tags.json".to_string(),
        ));
        app.add_plugins(crate::effects::EffectPlugin);
        app.update(); // Load tags

        // Register immunity effect
        let tags_manager = app.world().resource::<GameplayTagsManager>();
        let poison_query = GameplayEffectQuery::new()
            .with_definition_id("poison_damage");

        let mut immunity_effect = GameplayEffectDefinition::new("poison_immunity")
            .with_duration_policy(DurationPolicy::Infinite)
            .add_component(Arc::new(ImmunityComponent::new(vec![poison_query])));

        app.world_mut()
            .resource_mut::<crate::effects::definition::GameplayEffectRegistry>()
            .register(immunity_effect);

        // Register poison effect
        let poison_effect = GameplayEffectDefinition::new("poison_damage")
            .with_duration_policy(DurationPolicy::HasDuration)
            .with_duration(5.0)
            .add_modifier(ModifierInfo::new(
                "Health",
                ModifierOperation::Add,
                MagnitudeCalculation::ScalableFloat {
                    base_value: -10.0,
                    level_multiplier: 1.0,
                },
            ));

        app.world_mut()
            .resource_mut::<crate::effects::definition::GameplayEffectRegistry>()
            .register(poison_effect);

        // Spawn player with attribute set
        let player = app
            .world_mut()
            .spawn((
                Name::new("Player"),
                OwnedTags::default(),
                BlockedAbilityTags::default(),
            ))
            .id();

        // Apply immunity effect
        app.world_mut().commands().trigger(ApplyGameplayEffectEvent {
            effect_id: "poison_immunity".into(),
            target: player,
            source: None,
            level: 1,
            set_by_caller_magnitudes: None,
        });

        app.update();

        // Verify immunity component was added
        assert!(app
            .world()
            .get::<ActiveImmunityEffects>(player)
            .is_some());

        // Try to apply poison - should be blocked
        app.world_mut().commands().trigger(ApplyGameplayEffectEvent {
            effect_id: "poison_damage".into(),
            target: player,
            source: None,
            level: 1,
            set_by_caller_magnitudes: None,
        });

        app.update();

        // Verify poison was NOT applied (no active effect with poison_damage ID)
        let poison_applied = app
            .world()
            .query::<&crate::effects::components::ActiveGameplayEffect>()
            .iter(app.world())
            .any(|effect| effect.definition_id.as_ref() == "poison_damage");

        assert!(!poison_applied, "Poison should have been blocked by immunity");
    }
}
