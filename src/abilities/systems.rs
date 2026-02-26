//! Ability system implementations.
//!
//! This module contains the observer functions and systems that manage gameplay abilities.

use super::components::*;
use super::definition::*;
use crate::attributes::{AttributeData, AttributeName, AttributeOwner};
use crate::effects::definition::GameplayEffectRegistry;
use crate::effects::systems::ApplyGameplayEffectEvent;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_gameplay_tag::gameplay_tag_count_container::GameplayTagCountContainer;

/// Mutable ability spec query (for activation writes).
type AbilitySpecMutQuery = Query<
    'static,
    'static,
    (
        &'static mut AbilitySpec,
        &'static AbilityOwner,
        &'static mut AbilityState,
    ),
>;

/// Read-only ability spec query (for cancel scan).
type AbilitySpecReadQuery =
    Query<'static, 'static, (Entity, &'static AbilitySpec, &'static AbilityOwner)>;

/// Bundled query parameters for activation checks (cost/tag validation).
#[derive(SystemParam)]
pub struct ActivationCheckParams<'w, 's> {
    pub effect_registry: Res<'w, GameplayEffectRegistry>,
    pub tags_manager: Res<'w, bevy_gameplay_tag::GameplayTagsManager>,
    pub time: Res<'w, Time>,
    pub tag_containers: Query<'w, 's, &'static mut GameplayTagCountContainer>,
    pub attributes: Query<
        'w,
        's,
        (
            &'static AttributeData,
            &'static AttributeName,
            &'static AttributeOwner,
        ),
    >,
}

/// Bundled query parameters for ending/cancelling abilities.
#[derive(SystemParam)]
pub struct EndAbilityParams<'w, 's> {
    pub ability_specs: Query<
        'w,
        's,
        (
            &'static mut AbilitySpec,
            &'static AbilityOwner,
            &'static mut AbilityState,
        ),
    >,
    pub tag_containers: Query<'w, 's, &'static mut GameplayTagCountContainer>,
    pub active_instances: Query<'w, 's, (Entity, &'static ActiveAbilityInstance)>,
}

// --- Events ---

/// Event for trying to activate an ability.
#[derive(Event, Debug, Clone)]
pub struct TryActivateAbilityEvent {
    /// The ability spec entity to activate.
    pub ability_spec: Entity,
    /// The owner entity.
    pub owner: Entity,
}

/// Event triggered when an ability is successfully activated.
/// Targeted at the owner entity.
#[derive(Event, Debug, Clone)]
pub struct AbilityActivatedEvent {
    /// The ability spec entity.
    pub ability_spec: Entity,
    /// The owner entity.
    pub owner: Entity,
    /// The active instance entity (if instanced).
    pub instance: Option<Entity>,
}

/// Event triggered when an ability ends.
#[derive(Event, Debug, Clone)]
pub struct AbilityEndedEvent {
    /// The ability spec entity.
    pub ability_spec: Entity,
    /// The owner entity.
    pub owner: Entity,
    /// The active instance entity (if instanced).
    pub instance: Option<Entity>,
    /// Whether the ability was cancelled (vs ended normally).
    pub was_cancelled: bool,
}

/// Event for requesting ability end.
#[derive(Event, Debug, Clone)]
pub struct EndAbilityEvent {
    /// The ability spec entity.
    pub ability_spec: Entity,
    /// The owner entity.
    pub owner: Entity,
}

/// Event for committing an ability (applying costs and cooldowns).
#[derive(Event, Debug, Clone)]
pub struct CommitAbilityEvent {
    /// The ability spec entity.
    pub ability_spec: Entity,
    /// The owner entity.
    pub owner: Entity,
}

/// Event for canceling an ability.
#[derive(Event, Debug, Clone)]
pub struct CancelAbilityEvent {
    /// The ability spec entity.
    pub ability_spec: Entity,
    /// The owner entity.
    pub owner: Entity,
}

/// Event triggered when ability activation fails.
#[derive(Event, Debug, Clone)]
pub struct AbilityActivationFailedEvent {
    /// The ability spec entity.
    pub ability_spec: Entity,
    /// The owner entity.
    pub owner: Entity,
    /// The reason activation failed.
    pub reason: ActivationFailureReason,
}

/// Event triggered with the result of committing an ability.
#[derive(Event, Debug, Clone)]
pub struct CommitAbilityResultEvent {
    /// The ability spec entity.
    pub ability_spec: Entity,
    /// The owner entity.
    pub owner: Entity,
    /// Whether the commit succeeded.
    pub success: bool,
}

// --- Enums ---

/// Reason why ability activation failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActivationFailureReason {
    /// Ability is already active (NonInstanced/InstancedPerActor).
    AlreadyActive,
    /// Ability is on cooldown.
    OnCooldown,
    /// Owner doesn't have enough resources for the cost.
    InsufficientCost,
    /// Owner is missing required tags.
    MissingRequiredTags,
    /// Owner has tags that block activation.
    BlockedByTags,
}

// --- Helper functions ---

/// Check if the ability is on cooldown by looking at the cooldown effect's granted_tags
/// on the owner's GameplayTagCountContainer.
fn is_on_cooldown(
    cooldown_effect_id: Option<&str>,
    effect_registry: &GameplayEffectRegistry,
    owner_tags: &GameplayTagCountContainer,
) -> bool {
    let Some(cd_id) = cooldown_effect_id else {
        return false;
    };
    let Some(cd_def) = effect_registry.get(cd_id) else {
        return false;
    };
    owner_tags.has_any_matching_gameplay_tags(&cd_def.granted_tags)
}

/// Check if the owner can afford the cost effect by pre-evaluating modifiers.
fn can_afford_cost(
    cost_effect_id: Option<&str>,
    effect_registry: &GameplayEffectRegistry,
    owner: Entity,
    attributes: &Query<(&AttributeData, &AttributeName, &AttributeOwner)>,
) -> bool {
    let Some(cost_id) = cost_effect_id else {
        return true;
    };
    let Some(cost_def) = effect_registry.get(cost_id) else {
        return true;
    };
    for modifier in &cost_def.modifiers {
        let magnitude = modifier.magnitude.evaluate(1, None);
        for (attr_data, attr_name, attr_owner) in attributes.iter() {
            if attr_owner.0 == owner
                && attr_name.as_str() == modifier.attribute_name
                && attr_data.current_value + magnitude < 0.0
            {
                return false;
            }
        }
    }
    true
}

// --- Observer functions ---

/// Observer for TryActivateAbilityEvent.
pub fn on_try_activate_ability(
    ev: On<TryActivateAbilityEvent>,
    mut commands: Commands,
    ability_registry: Res<AbilityRegistry>,
    mut spec_set: ParamSet<(AbilitySpecMutQuery, AbilitySpecReadQuery)>,
    mut params: ActivationCheckParams,
) {
    let event = ev.event();
    let spec_entity = event.ability_spec;
    let owner = event.owner;

    // Read spec data and definition via p0
    let (definition_id, is_active, instancing_policy, cancel_tags, owned_tags, block_tags);
    {
        let specs = spec_set.p0();
        let Ok((spec, _ability_owner, _state)) = specs.get(spec_entity) else {
            return;
        };
        definition_id = spec.definition_id.clone();
        is_active = spec.is_active;
    }

    let Some(definition) = ability_registry.get(&definition_id) else {
        return;
    };

    let Ok(owner_tags) = params.tag_containers.get(owner) else {
        return;
    };

    // --- CanActivate checks ---

    // 1. Already active? (NonInstanced/InstancedPerActor)
    if is_active
        && matches!(
            definition.instancing_policy,
            InstancingPolicy::NonInstanced | InstancingPolicy::InstancedPerActor
        )
    {
        commands.trigger(AbilityActivationFailedEvent {
            ability_spec: spec_entity,
            owner,
            reason: ActivationFailureReason::AlreadyActive,
        });
        return;
    }

    // 2. Cooldown?
    if is_on_cooldown(
        definition.cooldown_effect.as_deref(),
        &params.effect_registry,
        owner_tags,
    ) {
        commands.trigger(AbilityActivationFailedEvent {
            ability_spec: spec_entity,
            owner,
            reason: ActivationFailureReason::OnCooldown,
        });
        return;
    }

    // 3. Cost?
    if !can_afford_cost(
        definition.cost_effect.as_deref(),
        &params.effect_registry,
        owner,
        &params.attributes,
    ) {
        commands.trigger(AbilityActivationFailedEvent {
            ability_spec: spec_entity,
            owner,
            reason: ActivationFailureReason::InsufficientCost,
        });
        return;
    }

    // 4. Required tags?
    if !definition.activation_required_tags.is_empty()
        && !owner_tags.has_all_matching_gameplay_tags(&definition.activation_required_tags)
    {
        commands.trigger(AbilityActivationFailedEvent {
            ability_spec: spec_entity,
            owner,
            reason: ActivationFailureReason::MissingRequiredTags,
        });
        return;
    }

    // 5. Blocked tags?
    if owner_tags.has_any_matching_gameplay_tags(&definition.activation_blocked_tags) {
        commands.trigger(AbilityActivationFailedEvent {
            ability_spec: spec_entity,
            owner,
            reason: ActivationFailureReason::BlockedByTags,
        });
        return;
    }

    // --- PreActivate ---

    instancing_policy = definition.instancing_policy;
    owned_tags = definition.activation_owned_tags.clone();
    block_tags = definition.block_abilities_with_tags.clone();
    cancel_tags = definition.cancel_abilities_with_tags.clone();

    // 1. Set active state
    {
        let mut specs = spec_set.p0();
        let Ok((mut spec, _, mut state)) = specs.get_mut(spec_entity) else {
            return;
        };
        spec.is_active = true;
        spec.active_count += 1;
        *state = AbilityState::Active;
    }

    // 2. Add activation_owned_tags to owner
    if let Ok(mut owner_tag_container) = params.tag_containers.get_mut(owner) {
        owner_tag_container.update_tag_container_count(
            &owned_tags,
            1,
            &params.tags_manager,
            &mut commands,
            owner,
        );

        // 3. Add block_abilities_with_tags to owner
        owner_tag_container.update_tag_container_count(
            &block_tags,
            1,
            &params.tags_manager,
            &mut commands,
            owner,
        );
    }

    // 4. Cancel other active abilities matching cancel_abilities_with_tags
    if !cancel_tags.is_empty() {
        let all_specs = spec_set.p1();
        for (other_spec_entity, other_spec, other_owner) in all_specs.iter() {
            if other_spec_entity == spec_entity || !other_spec.is_active || other_owner.0 != owner {
                continue;
            }
            if let Some(other_def) = ability_registry.get(&other_spec.definition_id)
                && other_def.ability_tags.has_any(&cancel_tags)
            {
                commands.trigger(CancelAbilityEvent {
                    ability_spec: other_spec_entity,
                    owner,
                });
            }
        }
    }

    // Spawn instance if needed
    let instance = match instancing_policy {
        InstancingPolicy::InstancedPerExecution => {
            let instance_entity = commands
                .spawn(ActiveAbilityInstance::new(
                    spec_entity,
                    params.time.elapsed_secs(),
                ))
                .id();
            Some(instance_entity)
        }
        _ => None,
    };

    // Trigger AbilityActivatedEvent
    commands.trigger(AbilityActivatedEvent {
        ability_spec: spec_entity,
        owner,
        instance,
    });
}

/// Observer for CommitAbilityEvent.
pub fn on_commit_ability(
    ev: On<CommitAbilityEvent>,
    mut commands: Commands,
    ability_registry: Res<AbilityRegistry>,
    effect_registry: Res<GameplayEffectRegistry>,
    ability_specs: Query<(&AbilitySpec, &AbilityOwner)>,
    tag_containers: Query<&GameplayTagCountContainer>,
    attributes: Query<(&AttributeData, &AttributeName, &AttributeOwner)>,
) {
    let event = ev.event();
    let spec_entity = event.ability_spec;
    let owner = event.owner;

    let Ok((spec, _)) = ability_specs.get(spec_entity) else {
        return;
    };

    let Some(definition) = ability_registry.get(&spec.definition_id) else {
        return;
    };

    // --- CommitCheck ---
    if let Ok(owner_tags) = tag_containers.get(owner)
        && is_on_cooldown(
            definition.cooldown_effect.as_deref(),
            &effect_registry,
            owner_tags,
        )
    {
        commands.trigger(CommitAbilityResultEvent {
            ability_spec: spec_entity,
            owner,
            success: false,
        });
        return;
    }

    if !can_afford_cost(
        definition.cost_effect.as_deref(),
        &effect_registry,
        owner,
        &attributes,
    ) {
        commands.trigger(CommitAbilityResultEvent {
            ability_spec: spec_entity,
            owner,
            success: false,
        });
        return;
    }

    // --- CommitExecute ---

    if let Some(cost_id) = &definition.cost_effect {
        commands.trigger(ApplyGameplayEffectEvent {
            effect_id: cost_id.clone(),
            target: owner,
            instigator: Some(owner),
            level: spec.level,
        });
    }

    if let Some(cd_id) = &definition.cooldown_effect {
        commands.trigger(ApplyGameplayEffectEvent {
            effect_id: cd_id.clone(),
            target: owner,
            instigator: Some(owner),
            level: spec.level,
        });
    }

    commands.trigger(CommitAbilityResultEvent {
        ability_spec: spec_entity,
        owner,
        success: true,
    });
}

/// Observer for EndAbilityEvent.
pub fn on_end_ability(
    ev: On<EndAbilityEvent>,
    mut commands: Commands,
    ability_registry: Res<AbilityRegistry>,
    tags_manager: Res<bevy_gameplay_tag::GameplayTagsManager>,
    mut params: EndAbilityParams,
) {
    let event = ev.event();
    end_ability_internal(
        event.ability_spec,
        event.owner,
        false,
        &mut commands,
        &ability_registry,
        &tags_manager,
        &mut params,
    );
}

/// Observer for CancelAbilityEvent.
pub fn on_cancel_ability(
    ev: On<CancelAbilityEvent>,
    mut commands: Commands,
    ability_registry: Res<AbilityRegistry>,
    tags_manager: Res<bevy_gameplay_tag::GameplayTagsManager>,
    mut params: EndAbilityParams,
) {
    let event = ev.event();
    end_ability_internal(
        event.ability_spec,
        event.owner,
        true,
        &mut commands,
        &ability_registry,
        &tags_manager,
        &mut params,
    );
}

/// Shared logic for ending/cancelling an ability.
fn end_ability_internal(
    spec_entity: Entity,
    owner: Entity,
    was_cancelled: bool,
    commands: &mut Commands,
    ability_registry: &AbilityRegistry,
    tags_manager: &Res<bevy_gameplay_tag::GameplayTagsManager>,
    params: &mut EndAbilityParams,
) {
    let Ok((mut spec, _, mut state)) = params.ability_specs.get_mut(spec_entity) else {
        return;
    };

    if !spec.is_active {
        return;
    }

    let Some(definition) = ability_registry.get(&spec.definition_id) else {
        return;
    };

    let owned_tags = definition.activation_owned_tags.clone();
    let block_tags = definition.block_abilities_with_tags.clone();
    let instancing_policy = definition.instancing_policy;

    // 1. Remove activation_owned_tags from owner
    if let Ok(mut owner_tag_container) = params.tag_containers.get_mut(owner) {
        owner_tag_container.update_tag_container_count(
            &owned_tags,
            -1,
            tags_manager,
            commands,
            owner,
        );

        // 2. Remove block_abilities_with_tags from owner
        owner_tag_container.update_tag_container_count(
            &block_tags,
            -1,
            tags_manager,
            commands,
            owner,
        );
    }

    // 3. Update spec state
    spec.active_count = spec.active_count.saturating_sub(1);
    if spec.active_count == 0 {
        spec.is_active = false;
    }
    *state = AbilityState::Ready;

    // 4. Despawn ActiveAbilityInstance (if InstancedPerExecution)
    if instancing_policy == InstancingPolicy::InstancedPerExecution {
        for (instance_entity, instance) in params.active_instances.iter() {
            if instance.spec_entity == spec_entity {
                commands.entity(instance_entity).despawn();
            }
        }
    }

    // 5. Trigger AbilityEndedEvent
    commands.trigger(AbilityEndedEvent {
        ability_spec: spec_entity,
        owner,
        instance: None,
        was_cancelled,
    });
}

// --- Kept systems ---

/// Check if abilities can be activated based on tag requirements.
pub fn check_ability_activation_requirements(
    ability_def: &AbilityDefinition,
    tags: &GameplayTagCountContainer,
) -> bool {
    if !tags
        .explicit_tags
        .has_all(&ability_def.activation_required_tags)
    {
        return false;
    }

    if tags
        .explicit_tags
        .has_any(&ability_def.activation_blocked_tags)
    {
        return false;
    }

    true
}

/// System that cancels abilities based on tags.
pub fn cancel_abilities_by_tags_system(
    mut commands: Commands,
    registry: Res<AbilityRegistry>,
    ability_specs: Query<(Entity, &AbilitySpec, &AbilityOwner)>,
    tag_containers: Query<&GameplayTagCountContainer>,
) {
    for (spec_entity, spec, owner) in ability_specs.iter() {
        if !spec.is_active {
            continue;
        }

        let Some(definition) = registry.get(&spec.definition_id) else {
            continue;
        };

        let Ok(tags) = tag_containers.get(owner.0) else {
            continue;
        };

        if tags.has_any_matching_gameplay_tags(&definition.cancel_on_tags_added) {
            commands.trigger(CancelAbilityEvent {
                ability_spec: spec_entity,
                owner: owner.0,
            });
        }
    }
}

/// System that updates ability states based on cooldowns and tags.
pub fn update_ability_states_system(
    mut ability_specs: Query<(&AbilitySpec, &AbilityOwner, &mut AbilityState)>,
    tag_containers: Query<&GameplayTagCountContainer>,
    registry: Res<AbilityRegistry>,
    effect_registry: Res<GameplayEffectRegistry>,
) {
    for (spec, owner, mut state) in ability_specs.iter_mut() {
        if spec.is_active {
            *state = AbilityState::Active;
            continue;
        }

        let Some(definition) = registry.get(&spec.definition_id) else {
            continue;
        };

        let Ok(tags) = tag_containers.get(owner.0) else {
            continue;
        };

        if is_on_cooldown(
            definition.cooldown_effect.as_deref(),
            &effect_registry,
            tags,
        ) {
            *state = AbilityState::Cooldown;
            continue;
        }

        if !check_ability_activation_requirements(definition, tags) {
            *state = AbilityState::Blocked;
            continue;
        }

        *state = AbilityState::Ready;
    }
}

/// System that updates ability cooldowns.
pub fn update_ability_cooldowns_system(
    mut commands: Commands,
    mut cooldowns: Query<(Entity, &mut AbilityCooldown)>,
    time: Res<Time>,
) {
    for (entity, mut cooldown) in cooldowns.iter_mut() {
        cooldown.tick(time.delta_secs());

        if cooldown.is_expired() {
            commands.entity(entity).remove::<AbilityCooldown>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use bevy_gameplay_tag::gameplay_tag::GameplayTag;
    use bevy_gameplay_tag::{GameplayTagsManager, GameplayTagsPlugin};

    #[test]
    fn test_check_activation_requirements() {
        let mut app = App::new();
        app.add_plugins(GameplayTagsPlugin::with_data_path(
            "assets/gameplay_tags.json".to_string(),
        ));
        app.update();

        app.world_mut()
            .run_system_once(|tags_manager: Res<GameplayTagsManager>| {
                let ability = AbilityDefinition::new("test")
                    .add_activation_required_tag(GameplayTag::new("State.Alive"), &tags_manager)
                    .add_activation_blocked_tag(GameplayTag::new("State.Stunned"), &tags_manager);

                assert_eq!(ability.id, "test");
                assert_eq!(ability.activation_required_tags.gameplay_tags.len(), 1);
                assert_eq!(ability.activation_blocked_tags.gameplay_tags.len(), 1);
            })
            .expect("System should run successfully");
    }
}
