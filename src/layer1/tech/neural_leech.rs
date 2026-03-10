use crate::layer1::pop::Pop;
use crate::layer1::GridPosition;
use crate::layer1::StressTracker;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct NeuralHub;

#[derive(Component)]
pub struct NeuralLinked {
    pub hub_entity: Entity,
}

#[derive(Component)]
pub struct NeuralShock; // Causes mental break

#[derive(Event)]
pub struct NeuralHubDeathEvent {
    pub hub_entity: Entity,
}

pub const LINK_RADIUS: i32 = 20;

#[allow(clippy::type_complexity)]
pub fn apply_neural_link_buffs_system(
    mut commands: Commands,
    hub_query: Query<(Entity, &GridPosition), With<NeuralHub>>,
    worker_query: Query<
        (Entity, &GridPosition),
        (With<Pop>, Without<NeuralHub>, Without<NeuralLinked>),
    >,
) {
    for (hub_entity, hub_pos) in hub_query.iter() {
        for (worker_entity, worker_pos) in worker_query.iter() {
            // Manhattan distance for grid
            let dist = (hub_pos.x - worker_pos.x).abs() + (hub_pos.y - worker_pos.y).abs();

            if dist <= LINK_RADIUS {
                commands
                    .entity(worker_entity)
                    .insert(NeuralLinked { hub_entity });
            }
        }
    }
}

pub fn process_neural_hub_decay_system(mut hub_query: Query<&mut StressTracker, With<NeuralHub>>) {
    for mut stress in hub_query.iter_mut() {
        stress.accumulated_stress = 100.0; // Pegged to max
    }
}

pub fn handle_hub_death_system(
    mut commands: Commands,
    mut events: EventReader<NeuralHubDeathEvent>,
    linked_query: Query<(Entity, &NeuralLinked)>,
) {
    for event in events.read() {
        for (entity, link) in linked_query.iter() {
            if link.hub_entity == event.hub_entity {
                commands.entity(entity).remove::<NeuralLinked>();
                commands.entity(entity).insert(NeuralShock);
            }
        }
    }
}
