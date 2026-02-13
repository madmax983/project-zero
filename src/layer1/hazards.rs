use crate::layer1::health::Health;
use crate::layer1::utility_types::ActionType;
use crate::shared::log::MessageLog;
use bevy_ecs::prelude::*;
use rand::Rng;

/// Checks for workplace accidents and applies damage if one occurs.
pub fn handle_workplace_hazards(world: &mut World, pop_entity: Entity, action_type: ActionType) {
    let danger = action_type.danger_level();

    // Danger is probability 0.0 to 1.0
    if danger <= 0.0 {
        return;
    }

    let mut rng = rand::thread_rng();

    if rng.gen_bool(danger) {
        let damage = action_type.accident_damage();

        // Apply damage if pop has Health
        if let Some(mut health) = world.get_mut::<Health>(pop_entity) {
            health.take_damage(damage);

            // Log accident
            if let Some(mut log) = world.get_resource_mut::<MessageLog>() {
                log.add(format!(
                    "ACCIDENT: Worker injured while {action_type:?}! (-{damage} HP)"
                ));
            }
        }
    }
}
