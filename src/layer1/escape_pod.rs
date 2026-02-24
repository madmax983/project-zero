// src/layer1/escape_pod.rs

use bevy_ecs::prelude::*;
use crate::shared::log::MessageLog;

/// Component representing an Escape Pod.
#[derive(Component, Default)]
pub struct EscapePod {
    /// Maximum number of occupants.
    pub capacity: usize,
    /// Whether the pod has launched.
    pub launched: bool,
    /// List of occupants inside the pod.
    pub occupants: Vec<Entity>,
}

/// Component tagging an entity (Pop) as inside an Escape Pod.
#[derive(Component)]
pub struct Evacuee {
    /// The pod the evacuee is inside.
    pub pod_entity: Entity,
}

/// Attempts to assign a pop to an escape pod.
///
/// Returns `true` if successful.
pub fn try_assign_pop(world: &mut World, pod_entity: Entity, pop_entity: Entity) -> bool {
    // Check capacity first (read-only access to EscapePod)
    let can_fit = if let Some(pod) = world.get::<EscapePod>(pod_entity) {
        pod.occupants.len() < pod.capacity
    } else {
        return false;
    };

    if can_fit {
        // Modify occupants
        if let Some(mut pod) = world.get_mut::<EscapePod>(pod_entity) {
            pod.occupants.push(pop_entity);
        }
        // Add Evacuee component
        world.entity_mut(pop_entity).insert(Evacuee { pod_entity });
        true
    } else {
        false
    }
}

/// System to handle launching escape pods.
///
/// Despawns the pod and its occupants if `launched` is true.
pub fn launch_pod_system(
    mut commands: Commands,
    query: Query<(Entity, &EscapePod)>,
    mut log: Option<ResMut<MessageLog>>,
) {
    for (pod_entity, pod) in &query {
        if pod.launched {
            if let Some(ref mut log) = log {
                log.add(format!("Escape Pod launched with {} occupants!", pod.occupants.len()));
            }

            // Despawn occupants
            for &occupant in &pod.occupants {
                commands.entity(occupant).despawn();
            }

            // Despawn pod
            commands.entity(pod_entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::pop::Pop;
    use crate::shared::log::MessageLog;

    #[test]
    fn test_escape_pod_capacity() {
        let mut world = World::new();
        let pod = world.spawn((
            Building { building_type: BuildingType::EscapePod },
            EscapePod { capacity: 3, ..Default::default() },
        )).id();

        let pop1 = world.spawn(Pop).id();
        let pop2 = world.spawn(Pop).id();
        let pop3 = world.spawn(Pop).id();
        let pop4 = world.spawn(Pop).id();

        // Simulate assigning pops to pod (logic usually in a system, but helper here)
        assert!(try_assign_pop(&mut world, pod, pop1));
        assert!(try_assign_pop(&mut world, pod, pop2));
        assert!(try_assign_pop(&mut world, pod, pop3));
        assert!(!try_assign_pop(&mut world, pod, pop4), "Should fail when full");
    }

    #[test]
    fn test_launch_removes_pod_and_occupants() {
        let mut world = World::new();
        world.insert_resource(MessageLog::default()); // Required for logging

        let pod = world.spawn((
            Building { building_type: BuildingType::EscapePod },
            EscapePod { capacity: 3, launched: false, ..Default::default() },
        )).id();

        let pop = world.spawn((
            Pop,
            Evacuee { pod_entity: pod }, // Component indicating they are inside
        )).id();

        // Manually assign pop to pod occupants for the test setup
        world.get_mut::<EscapePod>(pod).unwrap().occupants.push(pop);

        // Mark for launch
        world.get_mut::<EscapePod>(pod).unwrap().launched = true;

        // Run launch system
        let mut schedule = Schedule::default();
        // Ensure commands are applied by chaining apply_deferred
        schedule.add_systems((launch_pod_system, apply_deferred).chain());
        schedule.run(&mut world);

        // Verify entities are despawned (or marked dead/safe)
        // In this version of Bevy/ECS, get_entity returns Result, so Err means not found.
        assert!(world.get_entity(pod).is_err(), "Pod should be despawned (launched)");
        assert!(world.get_entity(pop).is_err(), "Pop should be despawned (safe)");

        // Verify log
        let log = world.resource::<MessageLog>();
        assert!(!log.messages.is_empty());
        assert!(log.messages[0].text.contains("Escape Pod launched"));
    }
}
