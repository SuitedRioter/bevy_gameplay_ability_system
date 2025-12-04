use bevy::ecs::{entity::Entity, observer::On, system::Query};

use crate::{
    attributes::types::AttributeSet,
    gameplay_effect::{
        components::{GameplayEffectSpec, Pending},
        events::OnAddModifierEvaluatedData,
    },
};

///处理spec应用前的校验，调用can_apply等方法
fn apply_gameplay_effect_spec_Validate(
    effect_spec: Query<(Entity, &GameplayEffectSpec, &Pending)>,
) {
}

fn internal_execute_mod<T: AttributeSet>(event: On<OnAddModifierEvaluatedData<T>>) {}
