#[cfg(test)]
mod tests {
    use crate::layer1::resources::ResourceType;
    use crate::layer2::fleet::{Fleet, FleetOrder, InOrbit};
    use crate::layer2::mining::{
        fleet_mine_order_system, mining_system, FleetCargo, FleetMining, MiningTarget,
    };
    use bevy_ecs::prelude::*;

    fn setup_world() -> World {

        // Register components if needed
        World::new()
    }

    #[test]
    fn test_mining_target_has_resources() {
        let mut world = setup_world();
        let asteroid = world
            .spawn(MiningTarget {
                resource_type: ResourceType::Ore,
                amount: 100.0,
                mining_difficulty: 1.0,
            })
            .id();

        let target = world
            .get::<MiningTarget>(asteroid)
            .expect("Asteroid should have mining target");
        assert_eq!(target.resource_type, ResourceType::Ore);
        assert_eq!(target.amount, 100.0);
    }

    #[test]
    fn test_fleet_cargo_capacity() {
        let mut world = setup_world();
        let fleet = world
            .spawn((
                Fleet,
                FleetCargo {
                    contents: vec![],
                    capacity: 50.0,
                },
            ))
            .id();

        let cargo = world
            .get::<FleetCargo>(fleet)
            .expect("Fleet should have cargo");
        assert_eq!(cargo.capacity, 50.0);
    }

    #[test]
    fn test_order_mining_initiates_process() {
        let mut world = setup_world();

        // Setup Asteroid
        let asteroid = world
            .spawn(MiningTarget {
                resource_type: ResourceType::Ore,
                amount: 100.0,
                mining_difficulty: 1.0,
            })
            .id();

        // Setup Fleet in Orbit of Asteroid
        let fleet = world
            .spawn((
                Fleet,
                InOrbit { parent: asteroid },
                FleetCargo {
                    contents: vec![],
                    capacity: 100.0,
                },
            ))
            .id();

        // Issue Mine Order
        world.entity_mut(fleet).insert(FleetOrder::Mine(asteroid));

        // Run Order System
        let mut schedule = Schedule::default();
        schedule.add_systems(fleet_mine_order_system);
        schedule.run(&mut world);

        // Verify Fleet is now Mining
        assert!(
            world.get::<FleetOrder>(fleet).is_none(),
            "Order should be consumed"
        );
        let mining_state = world
            .get::<FleetMining>(fleet)
            .expect("Fleet should be in mining state");
        assert_eq!(mining_state.target, asteroid);
    }

    #[test]
    fn test_order_mining_fails_if_not_in_orbit() {
        let mut world = setup_world();
        let asteroid = world
            .spawn(MiningTarget {
                resource_type: ResourceType::Ore,
                amount: 100.0,
                mining_difficulty: 1.0,
            })
            .id();
        let planet = world.spawn_empty().id();

        // Fleet in orbit of wrong entity
        let fleet = world
            .spawn((
                Fleet,
                InOrbit { parent: planet },
                FleetCargo {
                    contents: vec![],
                    capacity: 100.0,
                },
            ))
            .id();

        world.entity_mut(fleet).insert(FleetOrder::Mine(asteroid));

        let mut schedule = Schedule::default();
        schedule.add_systems(fleet_mine_order_system);
        schedule.run(&mut world);

        // Order consumed but no mining state
        assert!(world.get::<FleetOrder>(fleet).is_none());
        assert!(world.get::<FleetMining>(fleet).is_none());
    }

    #[test]
    fn test_order_mining_fails_if_full() {
        let mut world = setup_world();
        let asteroid = world
            .spawn(MiningTarget {
                resource_type: ResourceType::Ore,
                amount: 100.0,
                mining_difficulty: 1.0,
            })
            .id();

        // Fleet full
        let fleet = world
            .spawn((
                Fleet,
                InOrbit { parent: asteroid },
                FleetCargo {
                    contents: vec![],
                    capacity: 0.0,
                },
            ))
            .id();

        world.entity_mut(fleet).insert(FleetOrder::Mine(asteroid));

        let mut schedule = Schedule::default();
        schedule.add_systems(fleet_mine_order_system);
        schedule.run(&mut world);

        // Order consumed but no mining state
        assert!(world.get::<FleetOrder>(fleet).is_none());
        assert!(world.get::<FleetMining>(fleet).is_none());
    }

    #[test]
    fn test_mining_extracts_resources() {
        let mut world = setup_world();

        let asteroid = world
            .spawn(MiningTarget {
                resource_type: ResourceType::Ore,
                amount: 100.0,
                mining_difficulty: 1.0,
            })
            .id();

        let fleet = world
            .spawn((
                Fleet,
                InOrbit { parent: asteroid },
                FleetCargo {
                    contents: vec![],
                    capacity: 50.0,
                },
                FleetMining {
                    target: asteroid,
                    rate: 10.0, // Mines 10.0 per tick
                },
            ))
            .id();

        // Run Mining System
        let mut schedule = Schedule::default();
        schedule.add_systems(mining_system);
        schedule.run(&mut world);

        // Verify Asteroid lost resources
        let target = world.get::<MiningTarget>(asteroid).unwrap();
        assert!((target.amount - 90.0).abs() < f32::EPSILON);

        // Verify Fleet gained resources
        let cargo = world.get::<FleetCargo>(fleet).unwrap();
        assert_eq!(cargo.contents.len(), 1);
        assert_eq!(cargo.contents[0].resource_type, ResourceType::Ore);
        assert!((cargo.contents[0].amount - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_mining_stops_when_full() {
        let mut world = setup_world();

        let asteroid = world
            .spawn(MiningTarget {
                resource_type: ResourceType::Ore,
                amount: 100.0,
                mining_difficulty: 1.0,
            })
            .id();

        let fleet = world
            .spawn((
                Fleet,
                InOrbit { parent: asteroid },
                FleetCargo {
                    contents: vec![], // Empty
                    capacity: 5.0,    // Small capacity
                },
                FleetMining {
                    target: asteroid,
                    rate: 10.0, // Mines more than capacity
                },
            ))
            .id();

        // Run Mining System
        let mut schedule = Schedule::default();
        schedule.add_systems(mining_system);
        schedule.run(&mut world);

        // Verify Fleet is full (capped at 5.0)
        let cargo = world.get::<FleetCargo>(fleet).unwrap();
        assert!((cargo.contents[0].amount - 5.0).abs() < f32::EPSILON);

        // Verify Asteroid only lost 5.0
        let target = world.get::<MiningTarget>(asteroid).unwrap();
        assert!((target.amount - 95.0).abs() < f32::EPSILON);

        // Verify Mining State removed (completed/stopped)
        assert!(
            world.get::<FleetMining>(fleet).is_none(),
            "Should stop mining when full"
        );
    }

    #[test]
    fn test_mining_stops_when_depleted() {
        let mut world = setup_world();

        let asteroid = world
            .spawn(MiningTarget {
                resource_type: ResourceType::Ore,
                amount: 5.0, // Only 5 left
                mining_difficulty: 1.0,
            })
            .id();

        let fleet = world
            .spawn((
                Fleet,
                InOrbit { parent: asteroid },
                FleetCargo {
                    contents: vec![],
                    capacity: 50.0,
                },
                FleetMining {
                    target: asteroid,
                    rate: 10.0,
                },
            ))
            .id();

        // Run Mining System
        let mut schedule = Schedule::default();
        schedule.add_systems(mining_system);
        schedule.run(&mut world);

        // Verify Asteroid is empty/depleted
        let target = world.get::<MiningTarget>(asteroid).unwrap();
        assert!(target.amount <= 0.0);

        // Verify Fleet gained 5.0
        let cargo = world.get::<FleetCargo>(fleet).unwrap();
        assert!((cargo.contents[0].amount - 5.0).abs() < f32::EPSILON);

        // Verify Mining State removed
        assert!(world.get::<FleetMining>(fleet).is_none());
    }
}
