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
        let mut app = bevy::prelude::App::new();
        app.add_plugins(bevy::prelude::MinimalPlugins);
        // Setup Pressure Grid: (1,0) is High Pressure, (3,0) is Vacuum
        let mut grid = PressureGrid::new(5, 1);
        grid.set(1, 0, 1.0);
        app.insert_resource(grid);

        // Spawn Airlock at (2,0) set to OPEN
        app.world_mut().spawn((
            Building {
                building_type: BuildingType::Airlock,
            },
            GridPosition { x: 2, y: 0 },
            DoorControl {
                state: DoorState::Open,
            }, // Forced Open
            Structure::default(),
        ));

        app.add_systems(bevy::prelude::Update, update_pressure_system);

        // Run pressure update multiple times to allow diffusion
        for _ in 0..10 {
            // Replenish source to fight vacuum decay at edges
            app.world_mut()
                .resource_mut::<PressureGrid>()
                .set(1, 0, 1.0);
            app.update();
        }

        let grid = app.world().resource::<PressureGrid>();
        // Pressure should diffuse past the airlock because it is open
        assert!(
            grid.get(3, 0) > 0.05,
            "Pressure should vent through OPEN airlock"
        );
    }

    #[test]
    fn test_locked_airlock_maintains_pressure() {
        let mut app = bevy::prelude::App::new();
        app.add_plugins(bevy::prelude::MinimalPlugins);
        let mut grid = PressureGrid::new(5, 1);
        grid.set(1, 0, 1.0);
        app.insert_resource(grid);

        // Spawn Airlock at (2,0) set to LOCKED (Closed)
        app.world_mut().spawn((
            Building {
                building_type: BuildingType::Airlock,
            },
            GridPosition { x: 2, y: 0 },
            DoorControl {
                state: DoorState::Locked,
            },
            Structure::default(),
        ));

        app.add_systems(bevy::prelude::Update, update_pressure_system);

        // Run pressure update multiple times
        for _ in 0..10 {
            app.update();
        }

        let grid = app.world().resource::<PressureGrid>();
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

    #[test]
    fn test_emergency_venting_extinguishes_fire() {
        use crate::layer1::building::{Building, BuildingType};
        use crate::layer1::control::{DoorControl, DoorState};
        use crate::layer1::fire::Fire;
        use crate::layer1::map::GridPosition;
        use crate::layer1::pressure::PressureGrid;

        use crate::layer1::physics::pressure::process_door_venting_system;
        use bevy::prelude::{App, Update};
        let mut app = App::new();
        app.add_plugins(bevy::prelude::MinimalPlugins);

        let mut grid = PressureGrid::new(5, 5);
        grid.fill(1.0); // fully pressurized room
        app.insert_resource(grid);

        // Act: Force the DoorControl of the Airlock to DoorState::Open to vent the room.
        app.world_mut().spawn((
            Building {
                building_type: BuildingType::Airlock,
            },
            DoorControl {
                state: DoorState::Open,
            },
            GridPosition { x: 2, y: 2 },
        ));

        let fire = app
            .world_mut()
            .spawn((
                Fire {
                    intensity: 1.0,
                    lifetime: 100,
                    ..Default::default()
                },
                GridPosition { x: 2, y: 3 }, // adjacent to airlock
            ))
            .id();

        app.add_systems(Update, process_door_venting_system);
        app.add_systems(
            Update,
            crate::layer1::fire::fire_pressure_check_system.after(process_door_venting_system),
        );

        for _ in 0..10 {
            app.update();
        }

        // Assert: Verify that the Fire entity is despawned or extinguished due to lack of pressure.
        assert!(
            app.world().get_entity(fire).is_err(),
            "Fire should be extinguished due to emergency venting"
        );
    }

    #[test]
    fn test_emergency_venting_causes_vacuum_damage() {
        use crate::layer1::building::{Building, BuildingType};
        use crate::layer1::control::{DoorControl, DoorState};
        use crate::layer1::health::Health;
        use crate::layer1::map::GridPosition;
        use crate::layer1::pop::Pop;
        use crate::layer1::pressure::{pressure_damage_system, PressureGrid};

        use crate::layer1::physics::pressure::process_door_venting_system;
        use bevy::prelude::{App, Update};
        let mut app = App::new();
        app.add_plugins(bevy::prelude::MinimalPlugins);
        app.init_resource::<bevy::ecs::event::Events<crate::layer1::chronicle::AddChronicleEvent>>(
        );

        let mut grid = PressureGrid::new(5, 5);
        grid.fill(1.0); // fully pressurized room
        app.insert_resource(grid);

        app.world_mut().spawn((
            Building {
                building_type: BuildingType::Airlock,
            },
            DoorControl {
                state: DoorState::Open,
            },
            GridPosition { x: 2, y: 2 },
        ));

        let pop = app
            .world_mut()
            .spawn((
                Pop,
                Health {
                    current: 100.0,
                    max: 100.0,
                },
                GridPosition { x: 2, y: 3 }, // adjacent to airlock
            ))
            .id();

        app.add_systems(Update, process_door_venting_system);
        app.add_systems(
            Update,
            pressure_damage_system.after(process_door_venting_system),
        );

        for _ in 0..10 {
            app.update();
        }

        // Assert: Verify the Pop takes damage from lack of pressure/suffocation.
        let health = app.world().get::<Health>(pop).unwrap();
        assert!(health.current < 100.0, "Pop should take vacuum damage");
    }

    #[test]
    fn test_emergency_venting_pulls_unanchored_items() {
        use crate::layer1::building::{Building, BuildingType};
        use crate::layer1::control::{DoorControl, DoorState};
        use crate::layer1::items::{Item, ItemType};
        use crate::layer1::map::GridPosition;
        use crate::layer1::physics::pressure::process_door_venting_system;
        use crate::layer1::pressure::PressureGrid;
        use crate::layer1::suction::suction_system;
        use bevy::prelude::{App, Update};

        let mut app = App::new();
        app.add_plugins(bevy::prelude::MinimalPlugins);

        let mut grid = PressureGrid::new(5, 5);
        grid.fill(1.0);
        app.insert_resource(grid);

        // Act: Force the Airlock open.
        app.world_mut().spawn((
            Building {
                building_type: BuildingType::Airlock,
            },
            DoorControl {
                state: DoorState::Open,
            },
            GridPosition { x: 2, y: 2 },
        ));

        let item = app
            .world_mut()
            .spawn((
                Item {
                    item_type: ItemType::Rations,
                },
                GridPosition { x: 2, y: 4 }, // further away
            ))
            .id();

        app.add_systems(Update, process_door_venting_system);
        app.add_systems(Update, suction_system.after(process_door_venting_system));

        for _ in 0..10 {
            app.update();
        }

        // Assert: The unanchored item's position changes, moving towards the venting airlock (suction effect).
        let pos = app.world().get::<GridPosition>(item).unwrap();
        assert!(
            pos.x == 2 && pos.y < 4,
            "Item should be sucked toward the airlock"
        );
    }
}
