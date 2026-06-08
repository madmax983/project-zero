#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::clutter::ClutterGrid;
    use crate::layer1::map::GridPosition;
    use crate::layer1::social::empty_room::{
        evaluate_sanctuary_emptiness, apply_sanctuary_stress_relief, SanctuaryZone,
    };
    use crate::layer1::stress::StressTracker;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(ClutterGrid::new(10, 10));
        world
    }

    #[test]
    fn test_sanctuary_active_when_empty() {
        let mut app = App::new();
        app.add_systems(Update, evaluate_sanctuary_emptiness);

        let zone = app.world_mut().spawn((SanctuaryZone { active: false }, GridPosition { x: 0, y: 0 })).id();

        app.update();

        let sanctuary = app.world().get::<SanctuaryZone>(zone).unwrap();
        assert!(sanctuary.active, "Sanctuary should be active when no clutter is present");
    }

    #[test]
    fn test_sanctuary_deactivated_by_clutter() {
        let mut app = App::new();
        app.add_systems(Update, evaluate_sanctuary_emptiness);

        let zone = app.world_mut().spawn((SanctuaryZone { active: true }, GridPosition { x: 0, y: 0 })).id();
        app.world_mut().spawn((Building { building_type: BuildingType::Housing }, GridPosition { x: 0, y: 0 }));

        app.update();

        let sanctuary = app.world().get::<SanctuaryZone>(zone).unwrap();
        assert!(!sanctuary.active, "Sanctuary should deactivate if clutter is inside");
    }

    #[test]
    fn test_stress_relief_in_active_sanctuary() {
        let mut app = App::new();
        app.add_systems(Update, apply_sanctuary_stress_relief);

        let _zone = app.world_mut().spawn((SanctuaryZone { active: true }, GridPosition { x: 0, y: 0 })).id();
        let pop = app.world_mut().spawn((StressTracker { accumulated_stress: 50.0 }, GridPosition { x: 0, y: 0 })).id();

        app.update();

        let stress = app.world().get::<StressTracker>(pop).unwrap();
        assert!(stress.accumulated_stress < 50.0, "Pop should lose stress in an active sanctuary");
    }

    #[test]
    fn test_no_stress_relief_in_inactive_sanctuary() {
        let mut world = setup_world();

        world.spawn((
            Building {
                building_type: BuildingType::Housing,
            },
            GridPosition { x: 0, y: 0 },
        ));

        let pop = world
            .spawn((
                GridPosition { x: 0, y: 0 }, // Inside zone
                StressTracker {
                    accumulated_stress: 50.0,
                },
            ))
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems((
            evaluate_sanctuary_emptiness,
            apply_sanctuary_stress_relief.after(evaluate_sanctuary_emptiness),
        ));
        schedule.run(&mut world);
        let stress = world.get::<StressTracker>(pop).unwrap();
        assert_eq!(stress.accumulated_stress, 50.0, "Pop should not lose stress in an inactive sanctuary");
    }

    #[test]
    fn test_visit_can_spawn_clutter() {
        let mut app = App::new();
        app.world_mut().insert_resource(ClutterGrid::new(10, 10));
        app.add_systems(Update, apply_sanctuary_stress_relief);

        let _zone = app.world_mut().spawn((SanctuaryZone { active: true }, GridPosition { x: 0, y: 0 })).id();
        let _pop = app.world_mut().spawn((StressTracker { accumulated_stress: 50.0 }, GridPosition { x: 0, y: 0 })).id();

        for _ in 0..1000 {
            app.update();
        }

        let clutter_grid = app.world().resource::<ClutterGrid>();
        assert!(clutter_grid.get(0, 0) > 0.0);
    }
}
