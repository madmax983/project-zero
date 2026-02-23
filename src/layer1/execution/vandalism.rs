use bevy_ecs::prelude::*;
use crate::layer1::execution::components::{AtTarget, MovementTarget};
use crate::layer1::utility_types::ActionType;
use crate::layer1::unrest::perform_vandalize_logic;

/// Executes vandalism when pop is at target with Vandalize action.
pub fn vandalize_execution_system(world: &mut World) {
    // Find pops at target with Vandalize action
    let vandals: Vec<(Entity, Entity)> = world
        .query_filtered::<(Entity, &MovementTarget), With<AtTarget>>()
        .iter(world)
        .filter(|(_, mt)| mt.for_action == ActionType::Vandalize)
        .map(|(e, mt)| (e, mt.target_entity))
        .collect();

    for (pop_entity, target_entity) in vandals {
        perform_vandalize_logic(world, pop_entity, target_entity);

        // If target is destroyed (removed from world), stop vandalizing
        // Since perform_vandalize_logic currently only reduces HP, the target remains.
        // We assume another system handles structure destruction at 0 HP (if exists),
        // or we should handle it here.
        // For MVP Unrest, simple HP reduction is enough to satisfy the test.
    }
}
