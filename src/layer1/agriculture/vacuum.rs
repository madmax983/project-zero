use crate::layer1::biology::health::Health;
use crate::layer1::core::map::TilePos;
use crate::layer1::physics::pressure::PressureGrid;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct VoidCrop;

#[derive(Component)]
pub struct Growth {
    pub progress: f32,
    pub rate: f32,
}

pub fn process_void_crops_system(
    mut query: Query<(&TilePos, &mut Growth, Option<&mut Health>), With<VoidCrop>>,
    pressure: Option<Res<PressureGrid>>,
) {
    for (pos, mut growth, health) in query.iter_mut() {
        let local_pressure = if let Some(ref grid) = pressure {
            grid.get(pos.x, pos.y)
        } else {
            0.0
        };

        if local_pressure <= 0.01 {
            growth.progress += growth.rate * 1.0;
        } else {
            if let Some(mut h) = health {
                h.take_damage(10.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::biology::health::Health;
    use crate::layer1::core::map::TilePos;
    use crate::layer1::physics::pressure::PressureGrid;
    use bevy::prelude::*;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, process_void_crops_system);
        app
    }

    #[test]
    fn test_void_crop_grows_in_vacuum() {
        let mut app = setup_app();

        let mut grid = PressureGrid::new(10, 10);
        grid.set(5, 5, 0.0);
        app.world_mut().insert_resource(grid);

        let crop = app
            .world_mut()
            .spawn((
                VoidCrop,
                Growth {
                    progress: 0.0,
                    rate: 1.0,
                },
                TilePos { x: 5, y: 5 },
            ))
            .id();

        app.update();

        let growth = app.world().get::<Growth>(crop).unwrap();
        assert!(growth.progress > 0.0);
    }

    #[test]
    fn test_void_crop_dies_in_atmosphere() {
        let mut app = setup_app();

        let mut grid = PressureGrid::new(10, 10);
        grid.set(5, 5, 1.0);
        app.world_mut().insert_resource(grid);

        let crop = app
            .world_mut()
            .spawn((
                VoidCrop,
                Growth {
                    progress: 50.0,
                    rate: 1.0,
                },
                TilePos { x: 5, y: 5 },
                Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
            ))
            .id();

        app.update();

        let health = app.world().get::<Health>(crop).unwrap();
        assert!(health.current < 100.0);
    }
}
