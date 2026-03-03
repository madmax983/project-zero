#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::zone::{ZoneGrid, ZoneType};
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop;
    use crate::layer1::stress::StressTracker;
    use crate::layer1::social::empty_room::{update_sanctuary_system, visit_sanctuary_system, SanctuaryTracker};
    use crate::layer1::building::{Building, BuildingType};

    #[test]
    fn test_sanctuary_validity_check() {
        let mut world = World::new();
        let mut zone_grid = ZoneGrid::new(10, 10);

        // Create Sanctuary Zone covering (0,0) and (0,1)
        zone_grid.set(0, 0, ZoneType::Sanctuary);
        zone_grid.set(0, 1, ZoneType::Sanctuary);
        world.insert_resource(zone_grid);
        world.insert_resource(SanctuaryTracker::default());

        // Run system - should be valid as no objects exist
        let mut schedule = Schedule::default();
        schedule.add_systems(update_sanctuary_system);
        schedule.run(&mut world);

        let tracker = world.get_resource::<SanctuaryTracker>().unwrap();
        // Since both tiles are connected, it could be 1 room with size 2
        let effectiveness = tracker.get_effectiveness(GridPosition { x: 0, y: 0 });
        assert_eq!(effectiveness, 2.0); // 1.0 per tile
        assert_eq!(tracker.get_effectiveness(GridPosition { x: 0, y: 1 }), 2.0);
    }

    #[test]
    fn test_clutter_invalidates_sanctuary() {
        let mut world = World::new();
        let mut zone_grid = ZoneGrid::new(10, 10);
        zone_grid.set(0, 0, ZoneType::Sanctuary);
        world.insert_resource(zone_grid);
        world.insert_resource(SanctuaryTracker::default());

        // Spawn a building in the zone
        world.spawn((
            Building { building_type: BuildingType::Housing },
            GridPosition { x: 0, y: 0 },
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(update_sanctuary_system);
        schedule.run(&mut world);

        let tracker = world.get_resource::<SanctuaryTracker>().unwrap();
        assert_eq!(tracker.get_effectiveness(GridPosition { x: 0, y: 0 }), 0.0);
    }

    #[test]
    fn test_visit_reduces_stress() {
        let mut world = World::new();
        let mut zone_grid = ZoneGrid::new(10, 10);
        zone_grid.set(5, 5, ZoneType::Sanctuary);
        world.insert_resource(zone_grid);
        world.insert_resource(SanctuaryTracker::default());

        let pop = world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 }, // Inside zone
            StressTracker { accumulated_stress: 50.0 },
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems((update_sanctuary_system, visit_sanctuary_system.after(update_sanctuary_system)));
        schedule.run(&mut world);

        let stress = world.get::<StressTracker>(pop).unwrap();
        assert!(stress.accumulated_stress < 50.0);
    }
}
