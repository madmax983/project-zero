use crate::layer2::fleet::Fleet;
use crate::layer2::nebulae::{MovementSpeed, SpatialVolume};
use bevy::prelude::*;

#[derive(Component)]
pub struct GravitationalDoldrums {
    pub penalty_multiplier: f32,
}

#[derive(Component)]
pub struct TugShip {
    pub tow_capacity: u32,
}

#[derive(Component)]
pub struct TowedBy {
    pub tug_entity: Entity,
}

#[allow(clippy::type_complexity)]
pub fn doldrums_effects_system(
    doldrums_query: Query<(&GravitationalDoldrums, &SpatialVolume, &Transform)>,
    mut fleet_query: Query<
        (
            &Transform,
            Option<&mut MovementSpeed>,
            Option<&TugShip>,
            Option<&TowedBy>,
        ),
        With<Fleet>,
    >,
    tug_query: Query<&TugShip>,
) {
    for (fleet_transform, maybe_speed, maybe_tug, maybe_towed) in fleet_query.iter_mut() {
        let mut in_doldrums = false;
        let mut min_multiplier = 1.0;

        for (doldrums, volume, doldrums_transform) in doldrums_query.iter() {
            let distance = fleet_transform
                .translation
                .distance(doldrums_transform.translation);
            if distance <= volume.radius {
                in_doldrums = true;
                if doldrums.penalty_multiplier < min_multiplier {
                    min_multiplier = doldrums.penalty_multiplier;
                }
            }
        }

        if let Some(mut speed) = maybe_speed {
            if in_doldrums {
                let is_tug = maybe_tug.is_some();
                let is_towed_by_valid_tug = if let Some(towed) = maybe_towed {
                    tug_query.contains(towed.tug_entity)
                } else {
                    false
                };

                if !is_tug && !is_towed_by_valid_tug {
                    speed.current = speed.base * min_multiplier;
                } else {
                    speed.current = speed.base;
                }
            } else {
                speed.current = speed.base;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doldrums_reduce_movement_speed() {
        let mut app = App::new();

        let _doldrums = app
            .world_mut()
            .spawn((
                GravitationalDoldrums {
                    penalty_multiplier: 0.1,
                },
                SpatialVolume { radius: 10.0 },
                Transform::from_translation(Vec3::ZERO),
            ))
            .id();

        let ship = app
            .world_mut()
            .spawn((
                Fleet,
                Transform::from_translation(Vec3::ZERO),
                MovementSpeed {
                    base: 100.0,
                    current: 100.0,
                },
            ))
            .id();

        // Let's add the system to test its behavior properly
        app.add_systems(Update, doldrums_effects_system);
        app.update();

        let speed = app.world().get::<MovementSpeed>(ship).unwrap();
        assert_eq!(speed.current, 10.0);
    }

    #[test]
    fn test_tug_ships_ignore_doldrums() {
        let mut app = App::new();

        app.world_mut().spawn((
            GravitationalDoldrums {
                penalty_multiplier: 0.1,
            },
            SpatialVolume { radius: 10.0 },
            Transform::from_translation(Vec3::ZERO),
        ));

        let ship = app
            .world_mut()
            .spawn((
                Fleet,
                TugShip { tow_capacity: 100 },
                Transform::from_translation(Vec3::ZERO),
                MovementSpeed {
                    base: 100.0,
                    current: 100.0,
                },
            ))
            .id();

        app.add_systems(Update, doldrums_effects_system);
        app.update();

        let speed = app.world().get::<MovementSpeed>(ship).unwrap();
        assert_eq!(speed.current, 100.0);
    }

    #[test]
    fn test_towed_fleets_move_at_tug_speed() {
        let mut app = App::new();

        app.world_mut().spawn((
            GravitationalDoldrums {
                penalty_multiplier: 0.1,
            },
            SpatialVolume { radius: 10.0 },
            Transform::from_translation(Vec3::ZERO),
        ));

        let tug = app
            .world_mut()
            .spawn((
                Fleet,
                TugShip { tow_capacity: 100 },
                Transform::from_translation(Vec3::ZERO),
            ))
            .id();

        let towed_ship = app
            .world_mut()
            .spawn((
                Fleet,
                TowedBy { tug_entity: tug },
                Transform::from_translation(Vec3::ZERO),
                MovementSpeed {
                    base: 100.0,
                    current: 100.0,
                },
            ))
            .id();

        app.add_systems(Update, doldrums_effects_system);
        app.update();

        let speed = app.world().get::<MovementSpeed>(towed_ship).unwrap();
        assert_eq!(speed.current, 100.0);
    }
}
