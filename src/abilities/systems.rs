//! Ability system implementations.
//!
//! This module contains the observer functions and systems that manage gameplay abilities.
//!
//! Activation flow:
//!   TryActivateAbilityEvent → can_activate check → PendingActivation marker
//!   → spawn_pending_ability_instances_system: spawn AbilitySpecInstance child entity → ReadyToActivate marker
//!   → call_activate_ability_system: pre_activate → activate → CommitAbilityEvent
//!   → on_commit_ability observer: apply costs/cooldowns
//!
//! End flow:
//!   EndAbilityEvent / CancelAbilityEvent → end_ability_internal:
//!       behavior.end → despawn instance entity → decrement AbilityActiveState
//!
//! Instance cleanup on AbilitySpec removal:
//!   Bevy hierarchy automatically despawns child AbilitySpecInstance entities.
//!   An observer on removal of AbilitySpecInstance calls behavior.end.

use super::components::*;
use super::definition::*;
use crate::attributes::{AttributeData, AttributeName};
use crate::core::BlockedAbilityTags;
use crate::core::OwnedTags;
use crate::effects::definition::GameplayEffectRegistry;
use bevy::ecs::relationship::Relationship;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_gameplay_tag::GameplayTagsManager;

// --- SystemParam bundles ---

/// Bundled query parameters for activation checks.
#[derive(SystemParam)]
pub struct ActivationCheckParams<'w, 's> {
    pub effect_registry: Res<'w, GameplayEffectRegistry>,
    pub tags_manager: Res<'w, bevy_gameplay_tag::GameplayTagsManager>,
    pub time: Res<'w, Time>,
    pub tag_containers: Query<'w, 's, &'static mut OwnedTags>,
    pub attributes: Query<
        'w,
        's,
        (
            &'static AttributeData,
            &'static AttributeName,
            &'static ChildOf,
        ),
    >,
}

#[derive(SystemParam)]
pub struct EndAbilityParams<'w, 's> {
    pub ability_registry: Res<'w, AbilityRegistry>,
    pub tags_manager: Res<'w, GameplayTagsManager>,
    pub ability_specs: Query<
        'w,
        's,
        (
            &'static AbilitySpec,
            &'static mut AbilityActiveState,
            &'static AbilityOwner,
        ),
    >,
    pub instances: Query<
        'w,
        's,
        (
            Entity,
            &'static AbilitySpecInstance,
            &'static InstanceControlState,
            &'static ChildOf,
        ),
    >,
    pub tag_containers: Query<'w, 's, &'static mut OwnedTags>,
    pub blocked_ability_tags: Query<'w, 's, &'static mut BlockedAbilityTags>,
}

// --- Events ---

/// Event for trying to activate an ability.
#[derive(Event, Debug, Clone)]
pub struct TryActivateAbilityEvent {
    /// The ability spec entity to activate.
    pub ability_spec: Entity,
    /// The owner entity.
    pub owner: Entity,
    /// Optional activation context (target data, instigator, etc.).
    pub context: Option<super::activation_context::AbilityActivationContext>,
}

impl TryActivateAbilityEvent {
    /// Creates a new activation event with just spec and owner.
    pub fn new(ability_spec: Entity, owner: Entity) -> Self {
        Self {
            ability_spec,
            owner,
            context: None,
        }
    }

    /// Creates an activation event with full context.
    pub fn with_context(
        ability_spec: Entity,
        owner: Entity,
        context: super::activation_context::AbilityActivationContext,
    ) -> Self {
        Self {
            ability_spec,
            owner,
            context: Some(context),
        }
    }
}

/// Event triggered when an ability is successfully activated.
#[derive(Event, Debug, Clone)]
pub struct AbilityActivatedEvent {
    /// The ability spec entity.
    pub ability_spec: Entity,
    /// The owner entity.
    pub owner: Entity,
    /// The spawned instance entity.
    pub instance: Entity,
}

/// Event for requesting ability end.
#[derive(Event, Debug, Clone)]
pub struct EndAbilityEvent {
    /// The instance entity to end. If None, ends all instances of the spec.
    pub instance: Option<Entity>,
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
    /// The instance entity.
    pub instance: Entity,
    /// The owner entity.
    pub owner: Entity,
}

/// Event for canceling an ability.
#[derive(Event, Debug, Clone)]
pub struct CancelAbilityEvent {
    /// The instance entity to cancel. If None, cancels all instances.
    pub instance: Option<Entity>,
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
    /// The instance entity.
    pub instance: Entity,
    /// The owner entity.
    pub owner: Entity,
    /// Whether the commit succeeded.
    pub success: bool,
}

/// Entity event triggered on instance entity when ended.
#[derive(EntityEvent, Debug, Clone)]
pub struct OnGameplayAbilityEnded {
    #[event_target]
    pub ability_instance: Entity,
    pub was_cancelled: bool,
}

// --- Enums ---

/// Reason why ability activation failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActivationFailureReason {
    /// Ability is on cooldown.
    OnCooldown,
    /// Owner doesn't have enough resources for the cost.
    InsufficientCost,
    /// Owner is missing required tags.
    MissingRequiredTags,
    /// Owner has tags that block activation.
    BlockedByTags,
}

// --- Pending activation ---

/// Marker component inserted by the observer when activation checks pass.
/// The first system picks this up to spawn the instance entity.
#[derive(Component, Debug, Clone)]
pub struct PendingActivation {
    pub owner: Entity,
    pub activation_info: super::activation_info::AbilityActivationInfo,
}

/// Marker component inserted after instance spawn, before activation.
/// The second system picks this up to call behavior methods.
#[derive(Component, Debug, Clone)]
pub struct ReadyToActivate {
    pub owner: Entity,
    pub instance: Entity,
    pub activation_info: super::activation_info::AbilityActivationInfo,
}

/// First system: spawns AbilitySpecInstance entities for pending activations based on instancing policy.
///
/// For each AbilitySpec with PendingActivation:
/// 1. Resolves the ability definition from registry
/// 2. Based on instancing policy:
///    - NonInstanced: No instance entity, uses Entity::PLACEHOLDER
///    - InstancedPerActor: Reuses existing instance or creates new one
///    - InstancedPerExecution: Always creates new instance (default)
/// 3. Adds ReadyToActivate marker for the next system
pub fn spawn_pending_ability_instances_system(
    mut commands: Commands,
    registry: Res<AbilityRegistry>,
    pending_query: Query<(Entity, &PendingActivation, &AbilitySpec), With<PendingActivation>>,
    existing_instances: Query<(Entity, &AbilitySpecInstance, &ChildOf)>,
) {
    for (spec_entity, pending, spec) in pending_query.iter() {
        let Some(def) = registry.get(&spec.definition_id) else {
            // Invalid definition, remove marker.
            commands.entity(spec_entity).remove::<PendingActivation>();
            continue;
        };

        let instance_entity = match def.instancing_policy {
            super::definition::InstancingPolicy::NonInstanced => {
                // No instance entity - logic executes directly from definition
                Entity::PLACEHOLDER
            }
            super::definition::InstancingPolicy::InstancedPerActor => {
                // Check if an instance already exists for this spec
                let existing = existing_instances
                    .iter()
                    .find(
                        |(_, instance, child_of): &(Entity, &AbilitySpecInstance, &ChildOf)| {
                            child_of.get() == spec_entity
                                && instance.definition_id == spec.definition_id
                        },
                    )
                    .map(|(entity, _, _)| entity);

                if let Some(existing_entity) = existing {
                    // Reuse existing instance
                    existing_entity
                } else {
                    // Create new instance (first activation)
                    commands
                        .spawn((
                            AbilitySpecInstance {
                                definition_id: spec.definition_id.clone(),
                                level: spec.level,
                                behavior: def.behavior.clone(),
                                owner: pending.owner,
                                instigator: Some(pending.activation_info.instigator),
                                target_data: Some(pending.activation_info.target_data.clone()),
                            },
                            InstanceControlState {
                                is_active: true,
                                is_blocking_other_abilities: def.default_blocks_other_abilities,
                                is_cancelable: def.default_is_cancelable,
                            },
                        ))
                        .set_parent_in_place(spec_entity)
                        .id()
                }
            }
            super::definition::InstancingPolicy::InstancedPerExecution => {
                // Always create new instance (current default behavior)
                commands
                    .spawn((
                        AbilitySpecInstance {
                            definition_id: spec.definition_id.clone(),
                            level: spec.level,
                            behavior: def.behavior.clone(),
                            owner: pending.owner,
                            instigator: Some(pending.activation_info.instigator),
                            target_data: Some(pending.activation_info.target_data.clone()),
                        },
                        InstanceControlState {
                            is_active: true,
                            is_blocking_other_abilities: def.default_blocks_other_abilities,
                            is_cancelable: def.default_is_cancelable,
                        },
                    ))
                    .set_parent_in_place(spec_entity)
                    .id()
            }
        };

        // Mark as ready for activation in next system.
        commands.entity(spec_entity).insert(ReadyToActivate {
            owner: pending.owner,
            instance: instance_entity,
            activation_info: pending.activation_info.clone(),
        });

        // Remove pending marker.
        commands.entity(spec_entity).remove::<PendingActivation>();
    }
}

/// Second system: calls behavior lifecycle methods and triggers events.
///
/// For each AbilitySpec with ReadyToActivate:
/// 1. Increments AbilityActiveState
/// 2. Adds activation_owned_tags to owner's OwnedTags
/// 3. Adds block_abilities_with_tags to owner's BlockedAbilityTags
/// 4. Calls pre_activate → activate on the behavior
/// 5. Triggers CommitAbilityEvent and AbilityActivatedEvent
pub fn call_activate_ability_system(
    mut commands: Commands,
    ability_registry: Res<AbilityRegistry>,
    tags_manager: Res<GameplayTagsManager>,
    mut ready_query: Query<
        (
            Entity,
            &ReadyToActivate,
            &AbilitySpec,
            &mut AbilityActiveState,
        ),
        With<ReadyToActivate>,
    >,
    instances: Query<&AbilitySpecInstance>,
    mut tag_containers: Query<&mut OwnedTags>,
    mut blocked_ability_tags: Query<&mut BlockedAbilityTags>,
) {
    for (spec_entity, ready, spec, mut active_state) in ready_query.iter_mut() {
        let Ok(instance) = instances.get(ready.instance) else {
            // Instance not found, skip.
            commands.entity(spec_entity).remove::<ReadyToActivate>();
            continue;
        };

        let Some(definition) = ability_registry.get(&spec.definition_id) else {
            commands.entity(spec_entity).remove::<ReadyToActivate>();
            continue;
        };

        // Increment active state.
        active_state.increment();

        // Call behavior lifecycle methods.
        let b: &dyn super::traits::AbilityBehavior = match instance.behavior.as_deref() {
            Some(b) => b,
            None => &super::traits::DefaultAbilityBehavior,
        };

        b.pre_activate(
            &mut commands,
            ready.instance,
            spec_entity,
            ready.owner,
            definition,
            &tags_manager,
            &mut tag_containers,
            &mut blocked_ability_tags,
        );
        b.activate(
            &mut commands,
            ready.instance,
            spec_entity,
            &ready.activation_info,
        );

        // Trigger events.
        commands.trigger(CommitAbilityEvent {
            ability_spec: spec_entity,
            instance: ready.instance,
            owner: ready.owner,
        });

        commands.trigger(AbilityActivatedEvent {
            ability_spec: spec_entity,
            owner: ready.owner,
            instance: ready.instance,
        });

        info!(
            "Ability {:?} activated: spec={:?} instance={:?}",
            instance.definition_id, spec_entity, ready.instance
        );

        // Remove ready marker.
        commands.entity(spec_entity).remove::<ReadyToActivate>();
    }
}

/// Observer for TryActivateAbilityEvent.
pub fn on_try_activate_ability(
    ev: On<TryActivateAbilityEvent>,
    mut commands: Commands,
    ability_registry: Res<AbilityRegistry>,
    ability_specs: Query<&AbilitySpec>,
    tags_manager: Res<bevy_gameplay_tag::GameplayTagsManager>,
    world: &World,
) {
    let event = ev.event();
    let spec_entity = event.ability_spec;
    let owner = event.owner;

    let Ok(spec) = ability_specs.get(spec_entity) else {
        return;
    };

    let Some(definition) = ability_registry.get(&spec.definition_id) else {
        return;
    };

    // Check if already pending activation (prevent duplicate activation in same frame).
    if world.get::<PendingActivation>(spec_entity).is_some() {
        return;
    }

    let behavior = definition
        .behavior
        .as_ref()
        .map(|b| b.as_ref() as &dyn super::traits::AbilityBehavior)
        .unwrap_or(&super::traits::DefaultAbilityBehavior);

    // Check if can activate
    if let Err(failure) = behavior.can_activate(world, spec_entity, owner, &tags_manager) {
        use super::traits::ActivationCheckFailure;
        let reason = match failure {
            ActivationCheckFailure::OnCooldown(_) => ActivationFailureReason::OnCooldown,
            ActivationCheckFailure::SourceMissingRequiredTags(_)
            | ActivationCheckFailure::TargetMissingRequiredTags(_) => {
                ActivationFailureReason::MissingRequiredTags
            }
            ActivationCheckFailure::SourceHasBlockedTags(_)
            | ActivationCheckFailure::TargetHasBlockedTags(_) => {
                ActivationFailureReason::BlockedByTags
            }
            ActivationCheckFailure::MissingComponents => return,
        };
        commands.trigger(AbilityActivationFailedEvent {
            ability_spec: spec_entity,
            owner,
            reason,
        });
        return;
    }

    // Mark for deferred activation.
    let activation_info = if let Some(ctx) = &event.context {
        // Convert AbilityActivationContext to AbilityActivationInfo
        super::activation_info::AbilityActivationInfo {
            owner: ctx.owner,
            instigator: ctx.activator,
            target_data: ctx
                .target_data
                .clone()
                .unwrap_or_else(super::target_data::GameplayAbilityTargetData::empty),
            level: ctx.level,
            event_payload: None,
        }
    } else {
        // No context provided, create minimal activation info
        super::activation_info::AbilityActivationInfo::new(
            owner,
            super::target_data::GameplayAbilityTargetData::empty(),
        )
    };

    commands.entity(spec_entity).insert(PendingActivation {
        owner,
        activation_info,
    });
}

/// Observer for CommitAbilityEvent.
pub fn on_commit_ability(
    ev: On<CommitAbilityEvent>,
    mut commands: Commands,
    ability_registry: Res<AbilityRegistry>,
    ability_specs: Query<&AbilitySpec>,
    tags_manager: Res<bevy_gameplay_tag::GameplayTagsManager>,
    world: &World,
) {
    let event = ev.event();
    let spec_entity = event.ability_spec;
    let instance_entity = event.instance;
    let owner = event.owner;

    let Ok(spec) = ability_specs.get(spec_entity) else {
        return;
    };

    let Some(definition) = ability_registry.get(&spec.definition_id) else {
        return;
    };

    let behavior = definition
        .behavior
        .as_ref()
        .map(|b| b.as_ref() as &dyn super::traits::AbilityBehavior)
        .unwrap_or(&super::traits::DefaultAbilityBehavior);

    if behavior
        .commit(world, &mut commands, definition, spec, owner, &tags_manager)
        .is_err()
    {
        commands.trigger(CommitAbilityResultEvent {
            ability_spec: spec_entity,
            instance: instance_entity,
            owner,
            success: false,
        });
        return;
    }

    commands.trigger(CommitAbilityResultEvent {
        ability_spec: spec_entity,
        instance: instance_entity,
        owner,
        success: true,
    });
}

/// Observer for EndAbilityEvent.
pub fn on_end_ability(
    ev: On<EndAbilityEvent>,
    mut commands: Commands,
    mut params: EndAbilityParams,
) {
    let event = ev.event();
    end_ability_internal(
        event.instance,
        event.ability_spec,
        event.owner,
        false,
        &mut commands,
        &mut params,
    );
}

/// Observer for CancelAbilityEvent.
pub fn on_cancel_ability(
    ev: On<CancelAbilityEvent>,
    mut commands: Commands,
    mut params: EndAbilityParams,
) {
    let event = ev.event();
    end_ability_internal(
        event.instance,
        event.ability_spec,
        event.owner,
        true,
        &mut commands,
        &mut params,
    );
}

/// Shared logic for ending/cancelling an ability.
///
/// If `instance` is Some, only that instance is ended. Otherwise all active
/// instances under the spec are ended.
fn end_ability_internal(
    instance: Option<Entity>,
    spec_entity: Entity,
    owner: Entity,
    was_cancelled: bool,
    commands: &mut Commands,
    params: &mut EndAbilityParams,
) {
    let Ok((spec, _, _)) = params.ability_specs.get(spec_entity) else {
        return;
    };

    let Some(definition) = params.ability_registry.get(&spec.definition_id) else {
        return;
    };

    let owned_tags = definition.activation_owned_tags.clone();
    let block_tags = definition.block_abilities_with_tags.clone();

    // Collect (entity, behavior) pairs for instances to end.
    let instances_to_end: Vec<(
        Entity,
        Option<std::sync::Arc<dyn super::traits::AbilityBehavior>>,
    )> = if let Some(inst) = instance {
        // End a specific instance.
        let Ok((_, inst_comp, ctrl, _)) = params.instances.get(inst) else {
            return;
        };
        if !ctrl.is_active || (was_cancelled && !ctrl.is_cancelable) {
            return;
        }
        vec![(inst, inst_comp.behavior.clone())]
    } else {
        // End all active instances that are children of this spec.
        params
            .instances
            .iter()
            .filter_map(|(inst_entity, inst_comp, ctrl, child_of)| {
                if child_of.get() != spec_entity || !ctrl.is_active {
                    return None;
                }
                if was_cancelled && !ctrl.is_cancelable {
                    return None;
                }
                Some((inst_entity, inst_comp.behavior.clone()))
            })
            .collect()
    };

    for (inst_entity, behavior) in &instances_to_end {
        // Call behavior.end.
        let b: &dyn super::traits::AbilityBehavior = match behavior.as_deref() {
            Some(b) => b,
            None => &super::traits::DefaultAbilityBehavior,
        };
        b.end(commands, *inst_entity, was_cancelled);

        // Remove activation_owned_tags from owner.
        if let Ok(mut owner_tags) = params.tag_containers.get_mut(owner) {
            owner_tags.0.update_tag_container_count(
                &owned_tags,
                -1,
                &params.tags_manager,
                commands,
                owner,
            );
        }

        // Remove block_abilities_with_tags from owner's BlockedAbilityTags.
        if let Ok(mut blocked_tags) = params.blocked_ability_tags.get_mut(owner) {
            blocked_tags.0.update_tag_container_count(
                &block_tags,
                -1,
                &params.tags_manager,
                commands,
                owner,
            );
        }

        // Despawn the instance entity.
        commands.entity(*inst_entity).despawn();

        // Decrement active state on the spec.
        if let Ok((_, mut active_state, _)) = params.ability_specs.get_mut(spec_entity) {
            active_state.decrement();
        }
    }
}

/// Observer that fires behavior.end when an AbilitySpecInstance is removed
/// (e.g., when the parent AbilitySpec entity is despawned via hierarchy cleanup).
pub fn on_instance_removed(
    ev: On<Remove, AbilitySpecInstance>,
    instances: Query<&AbilitySpecInstance>,
    mut commands: Commands,
) {
    let entity = ev.event_target();
    if let Ok(instance) = instances.get(entity) {
        let b: &dyn super::traits::AbilityBehavior = match instance.behavior.as_deref() {
            Some(b) => b,
            None => &super::traits::DefaultAbilityBehavior,
        };
        b.end(&mut commands, entity, true);
    }
}

// --- Helper functions ---

/// Check if abilities can be activated based on tag requirements.
pub fn check_ability_activation_requirements(
    ability_def: &AbilityDefinition,
    tags: &OwnedTags,
) -> bool {
    if !tags
        .0
        .explicit_tags
        .has_all(&ability_def.activation_required_tags)
    {
        return false;
    }

    if tags
        .0
        .explicit_tags
        .has_any(&ability_def.activation_blocked_tags)
    {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use bevy_gameplay_tag::gameplay_tag::GameplayTag;
    use bevy_gameplay_tag::{GameplayTagsManager, GameplayTagsPlugin};
    use string_cache::DefaultAtom as Atom;

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

                assert_eq!(ability.id, Atom::from("test"));
                assert_eq!(ability.activation_required_tags.gameplay_tags.len(), 1);
                assert_eq!(ability.activation_blocked_tags.gameplay_tags.len(), 1);
            })
            .expect("System should run successfully");
    }
}
