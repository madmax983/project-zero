#[cfg(test)]
mod tests {
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::fire::Fire;
    use crate::layer1::map::GridPosition;
    use crate::layer1::pressure::{update_pressure_system, PressureGrid};
    use crate::layer1::structure::Structure;
    use bevy_ecs::prelude::*;

    // New Component
    use crate::layer1::control::{DoorControl, DoorState};

    #[test]
    fn test_door_control_defaults_to_auto() {
        let control = DoorControl::default();
        assert_eq!(control.state, DoorState::Auto);
    }

    #[test]
    fn test_open_airlock_vents_pressure() {
        let mut world = World::new();
        // Setup Pressure Grid: (1,0) is High Pressure, (3,0) is Vacuum
        let mut grid = PressureGrid::new(5, 1);
        grid.set(1, 0, 1.0);
        world.insert_resource(grid);

        // Spawn Airlock at (2,0) set to OPEN
        world.spawn((
            Building {
                building_type: BuildingType::Airlock,
            },
            GridPosition { x: 2, y: 0 },
            DoorControl {
                state: DoorState::Open,
            }, // Forced Open
            Structure::default(),
        ));

        // Run pressure update multiple times to allow diffusion
        for _ in 0..10 {
            // Replenish source to fight vacuum decay at edges
            world.resource_mut::<PressureGrid>().set(1, 0, 1.0);
            update_pressure_system(&mut world);
        }

        let grid = world.resource::<PressureGrid>();
        // Pressure should diffuse past the airlock because it is open
        assert!(
            grid.get(3, 0) > 0.05,
            "Pressure should vent through OPEN airlock"
        );
    }

    #[test]
    fn test_locked_airlock_maintains_pressure() {
        let mut world = World::new();
        let mut grid = PressureGrid::new(5, 1);
        grid.set(1, 0, 1.0);
        world.insert_resource(grid);

        // Spawn Airlock at (2,0) set to LOCKED (Closed)
        world.spawn((
            Building {
                building_type: BuildingType::Airlock,
            },
            GridPosition { x: 2, y: 0 },
            DoorControl {
                state: DoorState::Locked,
            },
            Structure::default(),
        ));

        // Run pressure update multiple times
        for _ in 0..10 {
            update_pressure_system(&mut world);
        }

        let grid = world.resource::<PressureGrid>();
        assert!(
            grid.get(3, 0) < 0.01,
            "Pressure should NOT vent through LOCKED airlock"
        );
    }

    #[test]
    fn test_fire_extinguishes_in_vacuum() {
        let mut world = World::new();

        // Setup Pressure Grid with Vacuum at (5,5)
        let mut grid = PressureGrid::new(10, 10);
        grid.set(5, 5, 0.0); // Vacuum
        world.insert_resource(grid);

        // Spawn Fire at (5,5)
        let fire_entity = world
            .spawn((
                Fire {
                    lifetime: 50,
                    ..Default::default()
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // Run fire damage/update system
        // Note: You may need to update fire_damage_system or create a new fire_pressure_system
        crate::layer1::fire::fire_pressure_check_system(&mut world);

        // Fire should be extinguished (despawned) immediately due to lack of oxygen/pressure
        assert!(
            world.get_entity(fire_entity).is_err(),
            "Fire should die in vacuum"
        );
    }
}
