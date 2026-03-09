//! Effect system implementations.
//!
//! This module contains the observer functions and systems that manage gameplay effects.

use super::components::*;
use super::definition::*;
use crate::attributes::{AttributeData, AttributeName};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::ecs::relationship::Relationship;
use bevy_gameplay_tag::GameplayTagsManager;
use bevy_gameplay_tag::gameplay_tag_count_container::GameplayTagCountContainer;
use string_cache::DefaultAtom as Atom;

/// Bundled query parameters for applying gameplay effects.
#[derive(SystemParam)]
pub struct ApplyEffectParams<'w, 's> {
    pub tag_containers: Query<'w, 's, &'static mut GameplayTagCountContainer>,
    pub attributes: Query<
        'w,
        's,
        (
            &'static mut AttributeData,
            &'static AttributeName,
            &'static ChildOf,
        ),
    >,
    pub existing_effects: Query<
        'w,
        's,
        (
            Entity,
            &'static ActiveGameplayEffect,
            &'static EffectTarget,
            Option<&'static mut EffectDuration>,
        ),
    >,
}

/// Event for applying a gameplay effect.
#[derive(Event, Debug, Clone)]
pub struct ApplyGameplayEffectEvent {
    /// The effect definition ID to apply.
    pub effect_id: Atom,
    /// The target entity.
    pub target: Entity,
    /// The instigator entity (optional).
    pub instigator: Option<Entity>,
    /// The level at which to apply the effect.
    pub level: i32,
}

/// Event triggered when an effect is applied.
#[derive(Event, Debug, Clone)]
pub struct GameplayEffectAppliedEvent {
    /// The effect entity (None for instant effects that modify base_value directly).
    pub effect: Entity,
    /// The target entity.
    pub target: Entity,
    /// The effect definition ID.
    pub effect_id: Atom,
}

/// Event triggered when an effect is removed.
#[derive(Event, Debug, Clone)]
pub struct GameplayEffectRemovedEvent {
    /// The effect entity.
    pub effect: Entity,
    /// The target entity.
    pub target: Entity,
    /// The effect definition ID.
    pub effect_id: Atom,
}

/// Observer for ApplyGameplayEffectEvent.
pub fn on_apply_gameplay_effect(
    ev: On<ApplyGameplayEffectEvent>,
    mut commands: Commands,
    registry: Res<GameplayEffectRegistry>,
    tags_manager: Res<GameplayTagsManager>,
    time: Res<Time>,
    mut params: ApplyEffectParams,
) {
    let event = ev.event();
    let target = event.target;
    let effect_id = &event.effect_id;
    let level = event.level;

    let Some(definition) = registry.get(effect_id) else {
        warn!("Effect definition not found: {}", effect_id);
        return;
    };

    // Check application_tag_requirements
    if let Ok(owner_tags) = params.tag_containers.get(target)
        && !definition
            .application_tag_requirements
            .requirements_met(&owner_tags.explicit_tags)
    {
        return;
    }

    // Handle stacking
    match definition.stacking_policy {
        StackingPolicy::RefreshDuration => {
            // Find existing effect and refresh its duration
            for (effect_entity, active_effect, effect_target, duration) in
                params.existing_effects.iter_mut()
            {
                if effect_target.0 == target && active_effect.definition_id == *effect_id {
                    if let Some(mut dur) = duration {
                        dur.remaining = definition.duration_magnitude;
                    }
                    // Trigger applied event for the existing effect
                    commands.trigger(GameplayEffectAppliedEvent {
                        effect: effect_entity,
                        target,
                        effect_id: effect_id.clone(),
                    });
                    return;
                }
            }
            // Fall through to spawn new if no existing found
        }
        StackingPolicy::StackCount { max_stacks } => {
            // Find existing effect and increment stack count
            for (effect_entity, active_effect, effect_target, _) in params.existing_effects.iter() {
                if effect_target.0 == target && active_effect.definition_id == *effect_id {
                    if active_effect.stack_count < max_stacks {
                        // Need to increment stack count - we'll spawn a new modifier set
                        // by letting it fall through, but first update the existing effect
                        commands.entity(effect_entity).insert(ActiveGameplayEffect {
                            definition_id: active_effect.definition_id.clone(),
                            level: active_effect.level,
                            start_time: active_effect.start_time,
                            stack_count: active_effect.stack_count + 1,
                        });
                        commands.trigger(GameplayEffectAppliedEvent {
                            effect: effect_entity,
                            target,
                            effect_id: effect_id.clone(),
                        });
                    }
                    return;
                }
            }
            // Fall through to spawn new if no existing found
        }
        StackingPolicy::Independent => {
            // Always spawn a new effect entity
        }
    }

    match definition.duration_policy {
        DurationPolicy::Instant => {
            // Directly modify attribute base_value, no entity spawn
            for modifier in &definition.modifiers {
                let magnitude = modifier.magnitude.evaluate(level, None);
                for (mut attr_data, attr_name, attr_owner) in params.attributes.iter_mut() {
                    if attr_owner.0 == target && attr_name.0 == modifier.attribute_name {
                        match modifier.operation {
                            ModifierOperation::AddBase => {
                                attr_data.base_value += magnitude;
                                attr_data.current_value = attr_data.base_value;
                            }
                            ModifierOperation::AddCurrent => {
                                attr_data.base_value += magnitude;
                                attr_data.current_value = attr_data.base_value;
                            }
                            ModifierOperation::MultiplyAdditive
                            | ModifierOperation::MultiplyMultiplicative => {
                                attr_data.base_value *= 1.0 + magnitude;
                                attr_data.current_value = attr_data.base_value;
                            }
                            ModifierOperation::Override => {
                                attr_data.base_value = magnitude;
                                attr_data.current_value = magnitude;
                            }
                        }
                    }
                }
            }

            // Add granted_tags to target (even for instant, they'll be removed when the "instant" is done)
            // For instant effects, granted_tags are typically not used, but we support it
            if !definition.granted_tags.is_empty()
                && let Ok(mut target_tags) = params.tag_containers.get_mut(target)
            {
                target_tags.update_tag_container_count(
                    &definition.granted_tags,
                    1,
                    &tags_manager,
                    &mut commands,
                    target,
                );
            }

            // Use PLACEHOLDER since no entity is spawned for instant effects
            commands.trigger(GameplayEffectAppliedEvent {
                effect: Entity::PLACEHOLDER,
                target,
                effect_id: effect_id.clone(),
            });
        }
        DurationPolicy::HasDuration | DurationPolicy::Infinite => {
            // Spawn effect entity with components
            let mut effect_entity_commands = commands.spawn((
                ActiveGameplayEffect::new(effect_id.clone(), level, time.elapsed_secs()),
                EffectTarget(target),
                EffectInstigator(event.instigator),
            ));

            // Add duration component for HasDuration
            if definition.duration_policy == DurationPolicy::HasDuration {
                effect_entity_commands.insert(EffectDuration::new(definition.duration_magnitude));
            }

            // Add periodic component if needed
            if definition.period > 0.0 {
                effect_entity_commands.insert(PeriodicEffect::new(definition.period));
            }

            // Add granted tags component
            if !definition.granted_tags.is_empty() {
                effect_entity_commands.insert(EffectGrantedTags {
                    tags: definition.granted_tags.clone(),
                });
            }

            let effect_entity = effect_entity_commands.id();

            // Add granted_tags to target's GameplayTagCountContainer
            if !definition.granted_tags.is_empty()
                && let Ok(mut target_tags) = params.tag_containers.get_mut(target)
            {
                target_tags.update_tag_container_count(
                    &definition.granted_tags,
                    1,
                    &tags_manager,
                    &mut commands,
                    target,
                );
            }

            commands.trigger(GameplayEffectAppliedEvent {
                effect: effect_entity,
                target,
                effect_id: effect_id.clone(),
            });
        }
    }
}

/// System that creates modifier entities for active effects.
pub fn create_effect_modifiers_system(
    mut commands: Commands,
    registry: Res<GameplayEffectRegistry>,
    new_effects: Query<
        (
            Entity,
            &ActiveGameplayEffect,
            &EffectTarget,
            Option<&EffectInstigator>,
        ),
        Added<ActiveGameplayEffect>,
    >,
) {
    for (effect_entity, active_effect, target, _instigator) in new_effects.iter() {
        let Some(definition) = registry.get(&active_effect.definition_id) else {
            continue;
        };

        for modifier_info in &definition.modifiers {
            let source_value = None;
            let magnitude = modifier_info
                .magnitude
                .evaluate(active_effect.level, source_value);

            commands.spawn((
                AttributeModifier {
                    target_entity: target.0,
                    target_attribute: modifier_info.attribute_name.clone(),
                    operation: modifier_info.operation,
                    magnitude,
                },
                ModifierSource(effect_entity),
            ));
        }
    }
}

/// System that aggregates attribute modifiers and applies them to attributes.
pub fn aggregate_attribute_modifiers_system(
    mut commands: Commands,
    mut attributes: Query<(Entity, &mut AttributeData, &AttributeName, &ChildOf)>,
    modifiers: Query<&AttributeModifier>,
) {
    for (attr_entity, mut attr_data, attr_name, child_of) in attributes.iter_mut() {
        let owner = child_of.get();
        let mut applicable_modifiers: Vec<_> = modifiers
            .iter()
            .filter(|m| m.target_entity == owner && m.target_attribute == attr_name.0)
            .collect();

        applicable_modifiers.sort_by_key(|m| m.operation.priority());

        // Check for Override first (short-circuit)
        if let Some(override_mod) = applicable_modifiers
            .iter()
            .find(|m| matches!(m.operation, ModifierOperation::Override))
        {
            let old_value = attr_data.current_value;
            let new_value = override_mod.magnitude;
            if (old_value - new_value).abs() > f32::EPSILON {
                attr_data.current_value = new_value;
                commands.trigger(crate::attributes::systems::AttributeChangedEvent {
                    owner,
                    attribute: attr_entity,
                    attribute_name: attr_name.as_str().to_string(),
                    old_value,
                    new_value,
                });
            }
            continue;
        }

        let mut current = attr_data.base_value;

        // AddCurrent
        for modifier in applicable_modifiers.iter().filter(|m| matches!(m.operation, ModifierOperation::AddCurrent)) {
            current += modifier.magnitude;
        }

        // MultiplyAdditive: (1 + sum)
        let additive_multiplier: f32 = applicable_modifiers
            .iter()
            .filter(|m| matches!(m.operation, ModifierOperation::MultiplyAdditive))
            .map(|m| m.magnitude)
            .sum();
        current *= 1.0 + additive_multiplier;

        // MultiplyMultiplicative: prod(1 + m)
        for modifier in applicable_modifiers.iter().filter(|m| matches!(m.operation, ModifierOperation::MultiplyMultiplicative)) {
            current *= 1.0 + modifier.magnitude;
        }

        let old_value = attr_data.current_value;
        if (current - old_value).abs() > f32::EPSILON {
            attr_data.current_value = current;
            commands.trigger(crate::attributes::systems::AttributeChangedEvent {
                owner,
                attribute: attr_entity,
                attribute_name: attr_name.as_str().to_string(),
                old_value,
                new_value: current,
            });
        }
    }
}

/// System that updates effect durations.
pub fn update_effect_durations_system(mut effects: Query<&mut EffectDuration>, time: Res<Time>) {
    for mut duration in effects.iter_mut() {
        duration.tick(time.delta_secs());
    }
}

/// System that removes expired effects and cleans up granted tags.
pub fn remove_expired_effects_system(
    mut commands: Commands,
    tags_manager: Res<GameplayTagsManager>,
    effects: Query<(
        Entity,
        &EffectDuration,
        &ActiveGameplayEffect,
        &EffectTarget,
        Option<&EffectGrantedTags>,
    )>,
    modifiers: Query<(Entity, &ModifierSource)>,
    mut tag_containers: Query<&mut GameplayTagCountContainer>,
) {
    for (effect_entity, duration, active_effect, target, granted_tags) in effects.iter() {
        if duration.is_expired() {
            // Remove granted_tags from target's GameplayTagCountContainer
            if let Some(granted) = granted_tags
                && let Ok(mut target_tags) = tag_containers.get_mut(target.0)
            {
                target_tags.update_tag_container_count(
                    &granted.tags,
                    -1,
                    &tags_manager,
                    &mut commands,
                    target.0,
                );
            }

            // Remove all modifiers created by this effect
            for (modifier_entity, source) in modifiers.iter() {
                if source.0 == effect_entity {
                    commands.entity(modifier_entity).despawn();
                }
            }

            // Trigger removal event
            commands.trigger(GameplayEffectRemovedEvent {
                effect: effect_entity,
                target: target.0,
                effect_id: active_effect.definition_id.clone(),
            });

            // Remove the effect
            commands.entity(effect_entity).despawn();
        }
    }
}

/// System that executes periodic effects.
pub fn execute_periodic_effects_system(
    mut effects: Query<(&mut PeriodicEffect, &ActiveGameplayEffect, &EffectTarget)>,
    time: Res<Time>,
) {
    for (mut periodic, _active_effect, _target) in effects.iter_mut() {
        let executions = periodic.tick(time.delta_secs());
        for _ in 0..executions {
            // TODO: Trigger periodic execution
        }
    }
}

/// System that removes instant effects after they've been applied.
pub fn remove_instant_effects_system(
    mut commands: Commands,
    registry: Res<GameplayEffectRegistry>,
    instant_effects: Query<(Entity, &ActiveGameplayEffect), Added<ActiveGameplayEffect>>,
) {
    for (effect_entity, active_effect) in instant_effects.iter() {
        if let Some(definition) = registry.get(&active_effect.definition_id)
            && definition.duration_policy == DurationPolicy::Instant
        {
            commands.entity(effect_entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Resource, Default)]
    struct ReceivedApplyEvents(Vec<ApplyGameplayEffectEvent>);

    #[derive(Resource, Default)]
    struct ReceivedAppliedEvents(Vec<GameplayEffectAppliedEvent>);

    #[test]
    fn test_apply_effect_event() {
        let mut app = App::new();
        app.add_plugins(bevy_gameplay_tag::GameplayTagsPlugin::with_data_path(
            "assets/gameplay_tags.json".to_string(),
        ));
        app.init_resource::<ReceivedApplyEvents>();
        app.init_resource::<ReceivedAppliedEvents>();
        app.init_resource::<GameplayEffectRegistry>();
        app.init_resource::<Time>();
        app.add_observer(on_apply_gameplay_effect);
        app.update();

        app.add_observer(
            |ev: On<ApplyGameplayEffectEvent>, mut received: ResMut<ReceivedApplyEvents>| {
                received.0.push(ev.event().clone());
            },
        );
        app.add_observer(
            |ev: On<GameplayEffectAppliedEvent>, mut received: ResMut<ReceivedAppliedEvents>| {
                received.0.push(ev.event().clone());
            },
        );

        let effect = GameplayEffectDefinition::new("test_effect").with_duration(5.0);
        app.world_mut()
            .resource_mut::<GameplayEffectRegistry>()
            .register(effect);

        let target = app
            .world_mut()
            .spawn(GameplayTagCountContainer::default())
            .id();

        app.world_mut().trigger(ApplyGameplayEffectEvent {
            effect_id: Atom::from("test_effect"),
            target,
            instigator: None,
            level: 1,
        });

        app.update();

        let apply_events = app.world().resource::<ReceivedApplyEvents>();
        assert_eq!(apply_events.0.len(), 1);
        assert_eq!(apply_events.0[0].effect_id, Atom::from("test_effect"));
        assert_eq!(apply_events.0[0].target, target);
    }
}
