use bevy::math::Vec3;
use bevy::prelude::Transform;
use bevy_ecs::prelude::*;

/// Marker component for ships.
#[derive(Component, Debug, Clone, Copy)]
pub struct Ship;

/// Marker component for fleets.
#[derive(Component, Debug, Clone, Copy)]
pub struct Fleet;

/// Radius within which a ship can detect other ships.
#[derive(Component, Debug, Clone, Copy)]
pub struct SensorRange {
    pub radius: f32,
}

/// Status of visibility on the system map.
#[derive(Component, Debug, Clone, Copy)]
pub struct VisibilityStatus {
    pub is_visible: bool,
}

/// Celestial body tag.
#[derive(Component, Debug, Clone, Copy)]
pub struct CelestialBody;

/// Radius of a celestial body used for line-of-sight occlusion.
#[derive(Component, Debug, Clone, Copy)]
pub struct CelestialRadius {
    pub radius: f32,
}

#[allow(clippy::type_complexity)]
pub fn sensor_occlusion_system(
    scanner_query: Query<(&Transform, &SensorRange)>,
    mut target_query: Query<(&Transform, &mut VisibilityStatus)>,
    celestial_query: Query<(&Transform, &CelestialRadius), With<CelestialBody>>,
) {
    for (target_transform, mut visibility) in target_query.iter_mut() {
        let target_pos = target_transform.translation;
        let mut is_visible_to_any = false;

        for (scanner_transform, sensor_range) in scanner_query.iter() {
            let scanner_pos = scanner_transform.translation;
            let dist_to_target = scanner_pos.distance(target_pos);

            // Out of sensor range
            if dist_to_target > sensor_range.radius {
                continue;
            }

            // Check occlusion by celestial bodies
            let mut occluded = false;
            let dir = (target_pos - scanner_pos).normalize_or_zero();

            if dir != Vec3::ZERO {
                for (body_transform, body_radius) in celestial_query.iter() {
                    let body_pos = body_transform.translation;
                    let to_body = body_pos - scanner_pos;

                    // Project to_body onto the direction vector
                    let t = to_body.dot(dir);

                    if t > 0.0 && t < dist_to_target {
                        let closest_point = scanner_pos + dir * t;
                        let dist_to_line = closest_point.distance(body_pos);

                        if dist_to_line <= body_radius.radius {
                            occluded = true;
                            break;
                        }
                    }
                }
            }

            if !occluded {
                is_visible_to_any = true;
                break; // If any scanner can see the target, no need to check others
            }
        }

        visibility.is_visible = is_visible_to_any;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_app::App;
    use bevy_app::Update;

    #[test]
    fn test_celestial_body_blocks_line_of_sight() {
        let mut app = App::new();
        app.add_systems(Update, sensor_occlusion_system);

        // Arrange: Spawn scanning ship
        let _scanner_ship = app
            .world_mut()
            .spawn((
                Ship,
                Transform::from_translation(Vec3::new(100.0, 0.0, 0.0)),
                SensorRange { radius: 500.0 },
            ))
            .id();

        // Arrange: Spawn target ship (behind the planet)
        let target_ship = app
            .world_mut()
            .spawn((
                Ship,
                Transform::from_translation(Vec3::new(-100.0, 0.0, 0.0)),
                VisibilityStatus { is_visible: true },
            ))
            .id();

        // Arrange: Spawn planet in the middle blocking LoS
        app.world_mut().spawn((
            CelestialBody,
            Transform::from_translation(Vec3::ZERO),
            CelestialRadius { radius: 50.0 },
        ));

        // Act: Run the occlusion system
        app.update();

        // Assert: The target ship should no longer be visible to the scanner
        let visibility = app.world().get::<VisibilityStatus>(target_ship).unwrap();
        assert!(
            !visibility.is_visible,
            "Target ship should be occluded by the celestial body."
        );
    }

    #[test]
    fn test_unobstructed_line_of_sight() {
        let mut app = App::new();
        app.add_systems(Update, sensor_occlusion_system);

        // Arrange: Spawn scanning ship
        let _scanner_ship = app
            .world_mut()
            .spawn((
                Ship,
                Transform::from_translation(Vec3::new(100.0, 100.0, 0.0)),
                SensorRange { radius: 500.0 },
            ))
            .id();

        // Arrange: Spawn target ship
        let target_ship = app
            .world_mut()
            .spawn((
                Ship,
                Transform::from_translation(Vec3::new(-100.0, -100.0, 0.0)),
                VisibilityStatus { is_visible: false },
            ))
            .id();

        // Arrange: Spawn planet off to the side (not blocking)
        app.world_mut().spawn((
            CelestialBody,
            Transform::from_translation(Vec3::new(100.0, -100.0, 0.0)),
            CelestialRadius { radius: 50.0 },
        ));

        // Act: Run the occlusion system
        app.update();

        // Assert: The target ship should become visible
        let visibility = app.world().get::<VisibilityStatus>(target_ship).unwrap();
        assert!(
            visibility.is_visible,
            "Target ship should be visible when unobstructed."
        );
    }
}
