use bevy::ecs::{component::Component, entity::Entity};

#[derive(Component, Debug)]
#[expect(dead_code)]
pub struct ActiveEffectsContainer {
    pub active_effects: Vec<Entity>,
}

#[derive(Component, Debug)]
#[expect(dead_code)]
pub struct ActiveGameplayEffect {}
