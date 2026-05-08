#[cfg(test)]
mod tests {
    use super::*;
    use crate::cues::manager::{ActiveWhileActiveCues, GameplayCueParameters, StaticCueHandlers};
    use crate::cues::notify::GameplayCueNotifyStatic;
    use bevy_gameplay_tag::GameplayTag;
    use std::sync::{Arc, Mutex};

    struct TestWhileActiveCue {
        call_count: Arc<Mutex<usize>>,
    }

    impl GameplayCueNotifyStatic for TestWhileActiveCue {
        fn on_execute(
            &self,
            _target: Entity,
            _params: &GameplayCueParameters,
            _commands: &mut Commands,
        ) {
        }

        fn while_active(
            &self,
            _target: Entity,
            _params: &GameplayCueParameters,
            _commands: &mut Commands,
        ) {
            *self.call_count.lock().unwrap() += 1;
        }
    }

    #[test]
    fn test_while_active_cues_update() {
        let mut app = App::new();
        app.init_resource::<StaticCueHandlers>();

        // Register a test handler
        let call_count = Arc::new(Mutex::new(0));
        let handler = Arc::new(TestWhileActiveCue {
            call_count: call_count.clone(),
        });
        let tag = GameplayTag::new("GameplayCue.Test");

        app.world_mut()
            .resource_mut::<StaticCueHandlers>()
            .register(tag.clone(), handler);

        // Spawn an entity with active cues
        let entity = app
            .world_mut()
            .spawn({
                let mut active_cues = ActiveWhileActiveCues::default();
                active_cues.add(tag, GameplayCueParameters::new());
                active_cues
            })
            .id();

        // Run the system once
        app.world_mut()
            .run_system_once(update_while_active_cues_system);

        // Verify the handler was called
        assert_eq!(*call_count.lock().unwrap(), 1);

        // Run again
        app.world_mut()
            .run_system_once(update_while_active_cues_system);

        // Should be called again
        assert_eq!(*call_count.lock().unwrap(), 2);

        // Verify entity still has the component
        assert!(app
            .world()
            .entity(entity)
            .contains::<ActiveWhileActiveCues>());
    }

    #[test]
    fn test_while_active_cues_cleanup() {
        let mut app = App::new();
        app.init_resource::<StaticCueHandlers>();

        // Spawn an entity with active cues but no handler registered
        let tag = GameplayTag::new("GameplayCue.Missing");
        let entity = app
            .world_mut()
            .spawn({
                let mut active_cues = ActiveWhileActiveCues::default();
                active_cues.add(tag, GameplayCueParameters::new());
                active_cues
            })
            .id();

        // Run the system
        app.world_mut()
            .run_system_once(update_while_active_cues_system);

        // Component should be removed because no cues remain
        assert!(!app
            .world()
            .entity(entity)
            .contains::<ActiveWhileActiveCues>());
    }
}
