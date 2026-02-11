//! System sets for organizing GAS systems.
//!
//! This module defines the system sets used to organize and order
//! the various systems in the Gameplay Ability System.

use bevy::prelude::*;

/// System sets for the Gameplay Ability System.
///
/// These sets define the execution order of GAS systems within a frame.
/// Systems are organized into logical groups that run in sequence.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GasSystemSet {
    /// Input handling and ability activation requests.
    ///
    /// This runs first to capture player input and trigger ability activations.
    Input,

    /// Attribute updates and clamping.
    ///
    /// This runs after input to ensure attributes are up-to-date before
    /// checking ability costs and requirements.
    Attributes,

    /// Effect application and duration updates.
    ///
    /// This runs after attributes to apply new effects and update existing ones.
    Effects,

    /// Ability activation, commitment, and cancellation.
    ///
    /// This runs after effects to handle ability logic with current state.
    Abilities,

    /// Gameplay cue execution.
    ///
    /// This runs last to provide visual/audio feedback for all changes.
    Cues,

    /// Cleanup and finalization.
    ///
    /// This runs at the end to clean up expired effects, ended abilities, etc.
    Cleanup,
}

/// System sets for attribute systems.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AttributeSystemSet {
    /// Clamp attribute values to their min/max bounds.
    Clamp,
    /// Trigger attribute change events.
    Events,
}

/// System sets for effect systems.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EffectSystemSet {
    /// Apply new gameplay effects.
    Apply,
    /// Create modifiers for active effects.
    CreateModifiers,
    /// Aggregate modifiers and update attribute CurrentValues.
    Aggregate,
    /// Update effect durations.
    UpdateDurations,
    /// Execute periodic effects.
    ExecutePeriodic,
    /// Remove expired effects.
    RemoveExpired,
    /// Remove instant effects (they apply once then despawn).
    RemoveInstant,
}

/// System sets for ability systems.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AbilitySystemSet {
    /// Try to activate abilities.
    TryActivate,
    /// Commit abilities (apply costs and cooldowns).
    Commit,
    /// End active abilities.
    End,
    /// Cancel abilities based on tags.
    Cancel,
    /// Update ability states (Ready, Active, Cooldown, Blocked).
    UpdateStates,
    /// Update ability cooldowns.
    UpdateCooldowns,
}

/// System sets for cue systems.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CueSystemSet {
    /// Handle cue trigger events.
    Handle,
    /// Route cues to appropriate handlers.
    Route,
    /// Execute static cues.
    ExecuteStatic,
    /// Manage cue actors.
    ManageActors,
    /// Clean up finished cues.
    Cleanup,
    /// Update WhileActive cues.
    UpdateWhileActive,
}

/// Helper function to configure GAS system ordering.
///
/// This sets up the correct execution order for all GAS systems.
pub fn configure_gas_system_sets(app: &mut App) {
    app.configure_sets(
        Update,
        (
            GasSystemSet::Input,
            GasSystemSet::Attributes,
            GasSystemSet::Effects,
            GasSystemSet::Abilities,
            GasSystemSet::Cues,
            GasSystemSet::Cleanup,
        )
            .chain(),
    );

    // Configure attribute system ordering
    app.configure_sets(
        Update,
        (AttributeSystemSet::Clamp, AttributeSystemSet::Events)
            .chain()
            .in_set(GasSystemSet::Attributes),
    );

    // Configure effect system ordering
    app.configure_sets(
        Update,
        (
            EffectSystemSet::Apply,
            EffectSystemSet::CreateModifiers,
            EffectSystemSet::Aggregate,
            EffectSystemSet::UpdateDurations,
            EffectSystemSet::ExecutePeriodic,
            EffectSystemSet::RemoveExpired,
            EffectSystemSet::RemoveInstant,
        )
            .chain()
            .in_set(GasSystemSet::Effects),
    );

    // Configure ability system ordering
    app.configure_sets(
        Update,
        (
            AbilitySystemSet::TryActivate,
            AbilitySystemSet::Commit,
            AbilitySystemSet::End,
            AbilitySystemSet::Cancel,
            AbilitySystemSet::UpdateStates,
            AbilitySystemSet::UpdateCooldowns,
        )
            .chain()
            .in_set(GasSystemSet::Abilities),
    );

    // Configure cue system ordering
    app.configure_sets(
        Update,
        (
            CueSystemSet::Handle,
            CueSystemSet::Route,
            CueSystemSet::ExecuteStatic,
            CueSystemSet::ManageActors,
            CueSystemSet::Cleanup,
            CueSystemSet::UpdateWhileActive,
        )
            .chain()
            .in_set(GasSystemSet::Cues),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_sets_exist() {
        // Just verify the system sets can be created
        let _input = GasSystemSet::Input;
        let _attributes = GasSystemSet::Attributes;
        let _effects = GasSystemSet::Effects;
        let _abilities = GasSystemSet::Abilities;
        let _cues = GasSystemSet::Cues;
        let _cleanup = GasSystemSet::Cleanup;
    }

    #[test]
    fn test_configure_system_sets() {
        let mut app = App::new();
        configure_gas_system_sets(&mut app);
        // If this doesn't panic, the configuration is valid
    }
}
