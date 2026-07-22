use crate::layer1::Pop;
use bevy::prelude::*;
use bevy::utils::HashSet;

#[derive(Component)]
pub struct AdminHub;

#[derive(Component)]
pub struct SubOffice;

#[derive(Component)]
pub struct ArchivalNode;

#[derive(Component, Debug, PartialEq)]
pub struct BuildingLocation(pub i32, pub i32);

#[derive(Event, Default)]
pub struct LogicCascadeEvent;

#[derive(Component, Default)]
pub struct PopStatus {
    pub is_confused: bool,
}

#[allow(dead_code)]
pub fn fractal_bureaucracy_validation_system(
    hubs: Query<&BuildingLocation, With<AdminHub>>,
    offices: Query<&BuildingLocation, With<SubOffice>>,
    nodes: Query<&BuildingLocation, With<ArchivalNode>>,
    mut events: EventWriter<LogicCascadeEvent>,
) {
    let mut cascade = false;

    // Performance: Use spatial hashing for faster lookup
    let mut office_map = HashSet::new();
    for office_loc in offices.iter() {
        office_map.insert((office_loc.0, office_loc.1));
    }

    let mut node_map = HashSet::new();
    for node_loc in nodes.iter() {
        node_map.insert((node_loc.0, node_loc.1));
    }

    for hub_loc in hubs.iter() {
        let mut has_adjacent_office = false;

        let adjacent_offsets = [(-1, 0), (1, 0), (0, -1), (0, 1)];

        for offset in adjacent_offsets.iter() {
            let office_x = hub_loc.0 + offset.0;
            let office_y = hub_loc.1 + offset.1;

            if office_map.contains(&(office_x, office_y)) {
                has_adjacent_office = true;
                let mut has_adjacent_node = false;

                for node_offset in adjacent_offsets.iter() {
                    let node_x = office_x + node_offset.0;
                    let node_y = office_y + node_offset.1;
                    if node_map.contains(&(node_x, node_y)) {
                        has_adjacent_node = true;
                        break;
                    }
                }

                if !has_adjacent_node {
                    cascade = true;
                    break;
                }
            }
        }

        if !has_adjacent_office || cascade {
            cascade = true;
            break;
        }
    }

    if cascade {
        events.send(LogicCascadeEvent);
    }
}

#[allow(dead_code)]
pub fn apply_logic_cascade_system(
    mut events: EventReader<LogicCascadeEvent>,
    mut pops: Query<&mut PopStatus, With<Pop>>,
) {
    if !events.is_empty() {
        events.clear();
        for mut status in pops.iter_mut() {
            status.is_confused = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    fn spawn_test_world() -> World {
        World::new()
    }

    #[test]
    fn test_logic_cascade_on_broken_fractal() {
        let mut world = spawn_test_world();
        world.insert_resource(Events::<LogicCascadeEvent>::default());

        let pop_entity = world.spawn_empty().id();
        world
            .entity_mut(pop_entity)
            .insert((Pop, PopStatus { is_confused: false }));

        // Spawn a Hub without the required Sub-Offices and Nodes
        let hub_entity = world.spawn_empty().id();
        world
            .entity_mut(hub_entity)
            .insert((AdminHub, BuildingLocation(0, 0)));

        // Act
        let mut schedule = Schedule::default();
        schedule.add_systems(
            (
                fractal_bureaucracy_validation_system,
                apply_logic_cascade_system,
            )
                .chain(),
        );
        schedule.run(&mut world);

        // Assert: Pop should be confused due to broken chain
        let status = world.get::<PopStatus>(pop_entity).unwrap();
        assert!(
            status.is_confused,
            "Broken fractal chain must cause Logic Cascade (confusion) on Pops"
        );
    }

    #[test]
    fn test_logic_cascade_with_adjacency() {
        let mut world = spawn_test_world();
        world.insert_resource(Events::<LogicCascadeEvent>::default());

        let pop_entity = world.spawn_empty().id();
        world
            .entity_mut(pop_entity)
            .insert((Pop, PopStatus { is_confused: false }));

        // Correct chain: Hub at (0,0) -> Office at (1,0) -> Node at (2,0)
        world.spawn((AdminHub, BuildingLocation(0, 0)));
        world.spawn((SubOffice, BuildingLocation(1, 0)));
        world.spawn((ArchivalNode, BuildingLocation(2, 0)));

        let mut schedule = Schedule::default();
        schedule.add_systems(
            (
                fractal_bureaucracy_validation_system,
                apply_logic_cascade_system,
            )
                .chain(),
        );
        schedule.run(&mut world);

        // Assert: Pop should NOT be confused
        let status = world.get::<PopStatus>(pop_entity).unwrap();
        assert!(
            !status.is_confused,
            "Correct fractal chain should not cause Logic Cascade"
        );

        // Break chain (Node moved too far away)
        world.spawn((AdminHub, BuildingLocation(10, 10)));
        world.spawn((SubOffice, BuildingLocation(11, 10)));
        world.spawn((ArchivalNode, BuildingLocation(13, 10)));

        schedule.run(&mut world);
        let status = world.get::<PopStatus>(pop_entity).unwrap();
        assert!(
            status.is_confused,
            "Broken fractal chain should cause Logic Cascade"
        );
    }
}
