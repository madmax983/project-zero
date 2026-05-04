use bevy::prelude::*;

#[derive(Resource)]
pub struct PlanetCurvature {
    pub horizon_distance_base: f32,
}

#[derive(Component)]
pub struct Elevation(pub f32);

pub fn has_line_of_sight(world: &World, observer: Entity, target: Entity) -> bool {
    let curvature = world.get_resource::<PlanetCurvature>().unwrap();
    let observer_pos = world
        .get::<crate::layer1::core::map::GridPosition>(observer)
        .unwrap();
    let observer_elev = world.get::<Elevation>(observer).map(|e| e.0).unwrap_or(0.0);

    let target_pos = world
        .get::<crate::layer1::core::map::GridPosition>(target)
        .unwrap();
    let target_elev = world.get::<Elevation>(target).map(|e| e.0).unwrap_or(0.0);

    let dx = observer_pos.x - target_pos.x;
    let dy = observer_pos.y - target_pos.y;
    let dist = ((dx * dx + dy * dy) as f32).sqrt();

    // Simple horizon extension formula: base + elevation factor
    let effective_horizon =
        curvature.horizon_distance_base + (observer_elev * 1.0) + (target_elev * 1.0);

    dist <= effective_horizon
}

#[allow(clippy::module_inception)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_planetary_curvature_limits_los() {
        let mut app = App::new();
        // Setup mock curvature
        app.insert_resource(PlanetCurvature {
            horizon_distance_base: 10.0,
        });

        let observer = app
            .world_mut()
            .spawn((
                crate::layer1::core::map::GridPosition { x: 0, y: 0 },
                Elevation(0.0),
            ))
            .id();
        let target = app
            .world_mut()
            .spawn((
                crate::layer1::core::map::GridPosition { x: 15, y: 0 },
                Elevation(0.0),
            ))
            .id();

        // At ground level, distance 15 is > horizon 10, should not be visible
        assert!(!has_line_of_sight(app.world(), observer, target));
    }

    #[test]
    fn test_elevation_extends_horizon() {
        let mut app = App::new();
        app.insert_resource(PlanetCurvature {
            horizon_distance_base: 10.0,
        });

        // Observer is elevated, pushing horizon to e.g., 20
        let observer = app
            .world_mut()
            .spawn((
                crate::layer1::core::map::GridPosition { x: 0, y: 0 },
                Elevation(10.0),
            ))
            .id();
        let target = app
            .world_mut()
            .spawn((
                crate::layer1::core::map::GridPosition { x: 15, y: 0 },
                Elevation(0.0),
            ))
            .id();

        // Now visible
        assert!(has_line_of_sight(app.world(), observer, target));
    }
}
