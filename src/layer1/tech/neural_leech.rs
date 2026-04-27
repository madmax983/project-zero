use crate::layer1::map::GridPosition;
use crate::layer1::pop::Pop;
use crate::layer1::stress::StressTracker;
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

pub const LINK_RADIUS: f32 = 20.0;

/// Applies neural link buffs/status.
///
/// ⚡ Bolt Optimization: Removed intermediate `.collect::<Vec<_>>()` allocation of hubs
/// and replaced O(N) linear scan `hubs.iter().find(...)` with an O(1) query lookup.
#[allow(clippy::type_complexity)]
pub fn apply_neural_link_buffs_system(
    mut commands: Commands,
    hub_query: Query<(Entity, &GridPosition), With<NeuralHub>>,
    worker_query: Query<
        (Entity, &GridPosition),
        (With<Pop>, Without<NeuralHub>, Without<NeuralLinked>),
    >,
    linked_query: Query<(Entity, &GridPosition, &NeuralLinked), (With<Pop>, Without<NeuralHub>)>,
) {
    // 1. Link workers who are close to a hub
    for (hub_entity, hub_tf) in hub_query.iter() {
        for (worker_entity, worker_tf) in worker_query.iter() {
            // Simplified distance check (Manhattan distance)
            let dist = (hub_tf.x as f32 - worker_tf.x as f32).abs()
                + (hub_tf.y as f32 - worker_tf.y as f32).abs();

            if dist <= LINK_RADIUS {
                commands
                    .entity(worker_entity)
                    .insert(NeuralLinked { hub_entity });
            }
        }
    }

    // 2. Unlink workers who are too far from their specific hub, or if the hub no longer exists/moved
    for (worker_entity, worker_tf, link) in linked_query.iter() {
        if let Ok((_, hub_tf)) = hub_query.get(link.hub_entity) {
            let dist = (hub_tf.x as f32 - worker_tf.x as f32).abs()
                + (hub_tf.y as f32 - worker_tf.y as f32).abs();

            if dist > LINK_RADIUS {
                commands.entity(worker_entity).remove::<NeuralLinked>();
            }
        } else {
            // Hub might have died or lost GridPosition, wait for death event to trigger shock.
            // But if it just lost GridPosition, maybe we should unlink. We'll leave it to handle_hub_death_system.
        }
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

pub fn process_neural_hub_decay_system(mut query: Query<&mut StressTracker, With<NeuralHub>>) {
    for mut stress in query.iter_mut() {
        stress.accumulated_stress = 100.0;
    }
}

pub fn neural_hub_death_bridge_system(
    mut events: EventWriter<NeuralHubDeathEvent>,
    mut removed_hubs: RemovedComponents<NeuralHub>,
) {
    for entity in removed_hubs.read() {
        events.send(NeuralHubDeathEvent { hub_entity: entity });
    }
}
