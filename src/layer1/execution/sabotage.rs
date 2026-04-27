use bevy_ecs::prelude::*;
use crate::layer1::architecture::Structure;
use crate::layer1::utility_types::{ActionType, PopAction};
use crate::layer1::execution::components::MovementTarget;

pub fn sabotage_action_system(
    mut commands: Commands,
    mut pops: Query<(Entity, &mut PopAction, &MovementTarget)>,
    mut structures: Query<&mut Structure>,
) {
    for (pop_entity, mut action, mt) in pops.iter_mut() {
        if action.current == ActionType::Sabotage {
            let target = mt.target_entity;
            if let Ok(mut structure) = structures.get_mut(target) {
                structure.current_hp -= 10.0;
                if structure.current_hp < 0.0 {
                    structure.current_hp = 0.0;
                }
            }
            action.current = ActionType::Idle;
            commands.entity(pop_entity).remove::<MovementTarget>();
        }
    }
}
