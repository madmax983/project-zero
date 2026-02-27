use bevy_ecs::prelude::*;

#[derive(Component, Debug, Clone)]
pub struct Governor {
    pub pop_entity: Entity,
    pub assigned_at: u64,
}

pub fn assign_governor(world: &mut World, planet: Entity, pop: Entity) {
    // Basic assignment logic for Spec 241 dependency
    // In a real implementation (Spec 209), this might check eligibility, remove old governor, etc.
    world.entity_mut(planet).insert(Governor {
        pop_entity: pop,
        assigned_at: 0, // Placeholder
    });
}
