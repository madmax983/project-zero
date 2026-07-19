use crate::layer1::map::GridPosition;
use crate::layer2::ftl::wakes::SubspaceWakeEvent;
use bevy::prelude::*;
use rand::Rng;

#[derive(Component)]
pub struct SubspaceFlora;

#[derive(Component)]
pub struct Displaced;

pub fn resolve_subspace_wakes_system(
    mut commands: Commands,
    mut wake_events: EventReader<SubspaceWakeEvent>,
    entities: Query<(Entity, &GridPosition), Without<SubspaceFlora>>,
) {
    let mut rng = rand::thread_rng();
    for wake in wake_events.read() {
        let max_displaced = (wake.severity * 5.0).ceil() as usize;
        let mut count = 0;

        for (entity, _) in entities.iter() {
            if count >= max_displaced {
                break;
            }
            // Displace entities with a probability based on wake severity
            if rng.gen_range(0.0..1.0) < wake.severity {
                commands.entity(entity).insert(Displaced);
                count += 1;
            }
        }

        // Spawn flora
        if wake.severity > 0.5 {
            let num_flora = (wake.severity * 3.0).ceil() as usize;
            for _ in 0..num_flora {
                commands.spawn((
                    SubspaceFlora,
                    GridPosition {
                        x: rng.gen_range(0..100),
                        y: rng.gen_range(0..100),
                    },
                ));
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
    fn test_wake_teleports_entities() {
        let mut world = setup_world();
        let target_system = world.spawn_empty().id();
        let _entity = world.spawn(GridPosition { x: 5, y: 5 }).id();

        world
            .resource_mut::<Events<SubspaceWakeEvent>>()
            .send(SubspaceWakeEvent {
                system: target_system,
                severity: 1.0, // Guaranteed displacement for testing
            });

        let mut schedule = Schedule::default();
        schedule.add_systems(resolve_subspace_wakes_system);
        schedule.run(&mut world);

        // Assert some displacement occurred
        let mut q = world.query::<&Displaced>();
        assert!(
            q.iter(&world).count() > 0,
            "Entities should be displaced by severe wakes"
        );
    }

    #[test]
    fn test_wake_spawns_flora() {
        let mut world = setup_world();
        let target_system = world.spawn_empty().id();

        world
            .resource_mut::<Events<SubspaceWakeEvent>>()
            .send(SubspaceWakeEvent {
                system: target_system,
                severity: 1.0,
            });

        let mut schedule = Schedule::default();
        schedule.add_systems(resolve_subspace_wakes_system);
        schedule.run(&mut world);

        // Assert some flora was spawned
        let mut q = world.query::<&SubspaceFlora>();
        assert!(
            q.iter(&world).count() > 0,
            "Subspace flora should be spawned by severe wakes"
        );
    }
}
