use crate::layer1::architecture::building::Building;
use crate::layer1::entities::pop::Pop;
use crate::layer2::ftl::SubspaceWakeEvent;
use crate::layer2::generation::ColonyLocation;
use bevy::prelude::*;

// A simple marker for subspace flora
#[derive(Component)]
pub struct SubspaceFlora;

pub fn resolve_subspace_wake_system(
    mut commands: Commands,
    mut wake_events: EventReader<SubspaceWakeEvent>,
    colony_location_query: Query<(Entity, &ColonyLocation)>,
    pop_query: Query<Entity, With<Pop>>,
    building_query: Query<Entity, With<Building>>,
) {
    for event in wake_events.read() {
        // Find if this event affects the current Layer 1 colony
        let is_our_system = colony_location_query
            .iter()
            .any(|(entity, _)| entity == event.system);

        if !is_our_system {
            continue;
        }

        // Randomly teleport/destroy some pops/buildings based on severity
        // Or spawn SubspaceFlora
        if event.severity > 0.8 {
            // High severity: destroy some buildings/pops
            if let Some(pop_entity) = pop_query.iter().next() {
                commands.entity(pop_entity).despawn_recursive();
            }
            if let Some(building_entity) = building_query.iter().next() {
                commands.entity(building_entity).despawn_recursive();
            }
        } else if event.severity > 0.3 {
            // Moderate severity: spawn subspace flora
            commands.spawn(SubspaceFlora);
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
    fn test_wake_destroys_pop_high_severity() {
        let mut world = setup_world();

        let system_entity = world.spawn(ColonyLocation).id();
        let pop_entity = world.spawn(Pop).id();
        let building_entity = world
            .spawn(Building {
                building_type: crate::layer1::architecture::building::BuildingType::Stockpile,
            })
            .id();

        world
            .resource_mut::<Events<SubspaceWakeEvent>>()
            .send(SubspaceWakeEvent {
                system: system_entity,
                severity: 0.9,
            });

        let mut schedule = Schedule::default();
        schedule.add_systems(resolve_subspace_wake_system);
        schedule.run(&mut world);

        assert!(
            world.get_entity(pop_entity).is_err(),
            "Pop should be destroyed"
        );
        assert!(
            world.get_entity(building_entity).is_err(),
            "Building should be destroyed"
        );
    }

    #[test]
    fn test_wake_no_entities_high_severity() {
        let mut world = setup_world();

        let system_entity = world.spawn(ColonyLocation).id();

        world
            .resource_mut::<Events<SubspaceWakeEvent>>()
            .send(SubspaceWakeEvent {
                system: system_entity,
                severity: 0.9,
            });

        let mut schedule = Schedule::default();
        schedule.add_systems(resolve_subspace_wake_system);
        schedule.run(&mut world);

        // Should just complete without panicking
    }

    #[test]
    fn test_wake_spawns_flora_moderate_severity() {
        let mut world = setup_world();

        let system_entity = world.spawn(ColonyLocation).id();

        world
            .resource_mut::<Events<SubspaceWakeEvent>>()
            .send(SubspaceWakeEvent {
                system: system_entity,
                severity: 0.5,
            });

        let mut schedule = Schedule::default();
        schedule.add_systems(resolve_subspace_wake_system);
        schedule.run(&mut world);

        let flora_count = world.query::<&SubspaceFlora>().iter(&world).count();
        assert_eq!(flora_count, 1, "Should spawn one SubspaceFlora");
    }

    #[test]
    fn test_wake_ignored_if_wrong_system() {
        let mut world = setup_world();

        let _system_entity = world.spawn(ColonyLocation).id(); // The colony's actual system
        let other_system_entity = world.spawn_empty().id();

        let pop_entity = world.spawn(Pop).id();

        world
            .resource_mut::<Events<SubspaceWakeEvent>>()
            .send(SubspaceWakeEvent {
                system: other_system_entity, // different system
                severity: 0.9,
            });

        let mut schedule = Schedule::default();
        schedule.add_systems(resolve_subspace_wake_system);
        schedule.run(&mut world);

        assert!(
            world.get_entity(pop_entity).is_ok(),
            "Pop should NOT be destroyed"
        );
    }
}
