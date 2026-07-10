use crate::layer1::map::GridPosition;
use crate::layer2::ftl::wakes::SubspaceWakeEvent;
use crate::layer2::generation::ColonyLocation;
use bevy::prelude::*;
use rand::Rng;

pub fn resolve_subspace_wakes_system(
    mut wake_events: EventReader<SubspaceWakeEvent>,
    mut commands: Commands,
    query: Query<(Entity, &GridPosition)>,
    colony_location: Query<Entity, With<ColonyLocation>>,
) {
    let colony_sys = colony_location.get_single().ok();

    for event in wake_events.read() {
        if event.severity > 0.0 {
            // Check if the event's target system is the colony's system
            if Some(event.system) == colony_sys {
                // Find entities in the wake's system and randomly teleport/despawn them
                let mut rng = rand::thread_rng();
                for (entity, pos) in query.iter() {
                    // Only affect a random subset of entities based on severity
                    if rng.gen_bool(event.severity as f64) {
                        if event.severity > 0.5 {
                            // high severity despawns (void teleport)
                            commands.entity(entity).despawn();
                        } else {
                            // lower severity moves
                            commands.entity(entity).insert(GridPosition {
                                x: pos.x + rng.gen_range(-5..=5),
                                y: pos.y + rng.gen_range(-5..=5),
                            });
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(Events::<SubspaceWakeEvent>::default());
        world
    }

    #[test]
    fn test_wake_moves_entities() {
        let mut world = setup_world();
        let colony_sys = world.spawn(ColonyLocation).id();
        // Need to spawn enough entities so at least one is affected (rng is used)
        let mut entities = vec![];
        for _ in 0..100 {
            entities.push(world.spawn(GridPosition { x: 0, y: 0 }).id());
        }

        world
            .resource_mut::<Events<SubspaceWakeEvent>>()
            .send(SubspaceWakeEvent {
                system: colony_sys,
                severity: 0.3,
            });

        let mut schedule = Schedule::default();
        schedule.add_systems(resolve_subspace_wakes_system);
        schedule.run(&mut world);

        // At least one entity should have moved
        let mut moved = false;
        for entity in entities {
            let pos = world.get::<GridPosition>(entity).unwrap();
            if pos.x != 0 || pos.y != 0 {
                moved = true;
                break;
            }
        }
        assert!(moved, "At least one entity should have moved");
    }

    #[test]
    fn test_wake_despawns_entities() {
        let mut world = setup_world();
        let colony_sys = world.spawn(ColonyLocation).id();
        let mut entities = vec![];
        for _ in 0..100 {
            entities.push(world.spawn(GridPosition { x: 0, y: 0 }).id());
        }

        world
            .resource_mut::<Events<SubspaceWakeEvent>>()
            .send(SubspaceWakeEvent {
                system: colony_sys,
                severity: 0.8,
            });

        let mut schedule = Schedule::default();
        schedule.add_systems(resolve_subspace_wakes_system);
        schedule.run(&mut world);

        let mut despawned = false;
        for entity in entities {
            if world.get_entity(entity).is_err() {
                despawned = true;
                break;
            }
        }
        assert!(despawned, "At least one entity should be despawned");
    }

    #[test]
    fn test_zero_severity_wake_does_nothing() {
        let mut world = setup_world();
        let colony_sys = world.spawn(ColonyLocation).id();
        let entity = world.spawn(GridPosition { x: 0, y: 0 }).id();

        world
            .resource_mut::<Events<SubspaceWakeEvent>>()
            .send(SubspaceWakeEvent {
                system: colony_sys,
                severity: 0.0,
            });

        let mut schedule = Schedule::default();
        schedule.add_systems(resolve_subspace_wakes_system);
        schedule.run(&mut world);

        let pos = world.get::<GridPosition>(entity).unwrap();
        assert_eq!(pos.x, 0);
        assert_eq!(pos.y, 0);
    }

    #[test]
    fn test_wake_in_other_system_does_nothing() {
        let mut world = setup_world();
        let _colony_sys = world.spawn(ColonyLocation).id();
        let other_sys = world.spawn_empty().id();

        let entity = world.spawn(GridPosition { x: 0, y: 0 }).id();

        world
            .resource_mut::<Events<SubspaceWakeEvent>>()
            .send(SubspaceWakeEvent {
                system: other_sys,
                severity: 0.8,
            });

        let mut schedule = Schedule::default();
        schedule.add_systems(resolve_subspace_wakes_system);
        schedule.run(&mut world);

        let pos = world.get::<GridPosition>(entity).unwrap();
        assert_eq!(pos.x, 0);
        assert_eq!(pos.y, 0);
    }
}
