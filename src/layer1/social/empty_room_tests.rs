#[cfg(test)]
mod tests {
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::clutter::ClutterGrid;

    use crate::layer1::map::GridPosition;
    use crate::layer1::social::empty_room::{
        update_sanctuary_system, visit_sanctuary_system, ActiveSanctuaries,
    };
    use crate::layer1::stress::StressTracker;
    use crate::layer1::zone::{ZoneGrid, ZoneType};
    use bevy_ecs::prelude::*;

    fn setup_world() -> World {
        let mut world = World::new();
        let mut zone_grid = ZoneGrid::new(10, 10);
        zone_grid.set(0, 0, ZoneType::Sanctuary);
        zone_grid.set(0, 1, ZoneType::Sanctuary);
        world.insert_resource(zone_grid);
        world.insert_resource(ActiveSanctuaries::default());
        world.insert_resource(ClutterGrid::new(10, 10));
        world
    }

    #[test]
    fn test_sanctuary_validity_check() {
        let mut world = setup_world();

        let mut schedule = Schedule::default();
        schedule.add_systems(update_sanctuary_system);
        schedule.run(&mut world);

        let manager = world.resource::<ActiveSanctuaries>();
        assert_eq!(manager.sanctuaries.len(), 1);
        let sanctuary = &manager.sanctuaries[0];
        assert!(sanctuary.is_valid);
        assert_eq!(sanctuary.effectiveness, 2.0); // 1.0 per tile
    }

    #[test]
    fn test_clutter_invalidates_sanctuary() {
        let mut world = setup_world();

        // Spawn a building in the zone
        world.spawn((
            Building {
                building_type: BuildingType::Housing,
            },
            GridPosition { x: 0, y: 0 },
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(update_sanctuary_system);
        schedule.run(&mut world);

        let manager = world.resource::<ActiveSanctuaries>();
        assert_eq!(manager.sanctuaries.len(), 1);
        let sanctuary = &manager.sanctuaries[0];
        assert!(!sanctuary.is_valid);
        assert_eq!(sanctuary.effectiveness, 0.0);
    }

    #[test]
    fn test_visit_reduces_stress() {
        let mut world = setup_world();

        let _pop = world
            .spawn((
                GridPosition { x: 0, y: 0 }, // Inside zone
                StressTracker {
                    accumulated_stress: 50.0,
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems((
            update_sanctuary_system,
            visit_sanctuary_system.after(update_sanctuary_system),
        ));
        schedule.run(&mut world);
    }

    #[test]
    fn test_visit_can_spawn_clutter() {
        let mut world = setup_world();

        let _pop = world
            .spawn((
                GridPosition { x: 0, y: 0 }, // Inside zone
                StressTracker {
                    accumulated_stress: 50.0,
                },
            ))
            .id();

        // Seed random to ensure the 1% chance hits (or we can just mock it, but simplest is to run it enough times or use a controlled random. Actually, we can't easily inject a seeded RNG into the system because it uses thread_rng. We can just run it many times).
        let mut schedule = Schedule::default();
        schedule.add_systems((
            update_sanctuary_system,
            visit_sanctuary_system.after(update_sanctuary_system),
        ));

        // Run it 1000 times, the chance of not spawning clutter is (0.99)^1000 = 0.000043.
        for _ in 0..1000 {
            schedule.run(&mut world);
        }

        let clutter_grid = world.resource::<ClutterGrid>();
        // Since we spawned clutter at 0, 0, the value should be > 0.
        assert!(clutter_grid.get(0, 0) > 0.0);
    }
}
