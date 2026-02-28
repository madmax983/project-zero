#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::zone::{ZoneGrid, ZoneType};
    use crate::layer1::map::GridPosition;
    use crate::layer1::stress::StressTracker;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::items::Item;
    use crate::layer1::clutter::ClutterGrid;
    use crate::layer1::social::empty_room::{SanctuaryManager, update_sanctuary_system, visit_sanctuary_system};

    fn setup_world() -> World {
        let mut world = World::new();
        let mut zone_grid = ZoneGrid::new(10, 10);
        zone_grid.set(0, 0, ZoneType::Sanctuary);
        zone_grid.set(0, 1, ZoneType::Sanctuary);
        world.insert_resource(zone_grid);
        world.insert_resource(SanctuaryManager::default());
        world.insert_resource(ClutterGrid::new(10, 10));
        world
    }

    #[test]
    fn test_sanctuary_validity_check() {
        let mut world = setup_world();

        let mut schedule = Schedule::default();
        schedule.add_systems(update_sanctuary_system);
        schedule.run(&mut world);

        let manager = world.resource::<SanctuaryManager>();
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
            Building { building_type: BuildingType::Housing },
            GridPosition { x: 0, y: 0 },
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(update_sanctuary_system);
        schedule.run(&mut world);

        let manager = world.resource::<SanctuaryManager>();
        assert_eq!(manager.sanctuaries.len(), 1);
        let sanctuary = &manager.sanctuaries[0];
        assert!(!sanctuary.is_valid);
        assert_eq!(sanctuary.effectiveness, 0.0);
    }

    #[test]
    fn test_visit_reduces_stress() {
        let mut world = setup_world();

        let pop = world.spawn((
            GridPosition { x: 0, y: 0 }, // Inside zone
            StressTracker { accumulated_stress: 50.0, ..Default::default() },
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems((update_sanctuary_system, visit_sanctuary_system.after(update_sanctuary_system)));
        schedule.run(&mut world);

        let mood = world.get::<StressTracker>(pop).unwrap();
        // Base stress 50. Effectiveness is 2.0, so -0.2 stress => 49.8
        assert!(mood.accumulated_stress < 50.0);
        assert!((mood.accumulated_stress - 49.8).abs() < f32::EPSILON);
    }
}
