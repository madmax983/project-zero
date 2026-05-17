use crate::layer1::biology::health::Health;
use crate::layer1::map::GridPosition;
use crate::layer1::nature::terrain::{TerrainGrid, TerrainType};
use crate::layer2::syzygy::TidalForce;
use bevy_ecs::prelude::*;

pub fn process_crustal_tides(
    mut terrain: ResMut<TerrainGrid>,
    tidal_force: Res<TidalForce>,
    mut buildings: Query<(&GridPosition, &mut Health)>,
) {
    let high_tide = tidal_force.current > 0.7;
    let low_tide = tidal_force.current < 0.3;

    for y in 0..terrain.height {
        for x in 0..terrain.width {
            if let Some(TerrainType::FaultLine(is_open)) = terrain.get(x, y) {
                if high_tide && !is_open {
                    terrain.set(x, y, TerrainType::FaultLine(true));
                } else if low_tide && is_open {
                    terrain.set(x, y, TerrainType::FaultLine(false));

                    // Damage buildings on the closing fissure
                    for (pos, mut health) in buildings.iter_mut() {
                        if pos.x == (x as i32) && pos.y == (y as i32) {
                            health.take_damage(50.0);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::biology::health::Health;
    use crate::layer1::nature::terrain::{generate_terrain, TerrainGrid, TerrainType};
    use crate::layer2::syzygy::TidalForce;
    use bevy_app::prelude::*;

    #[test]
    fn test_high_tide_opens_fissures() {
        let mut app = App::new();
        // app.add_plugins(MinimalPlugins); // Need to see if this is needed.

        let mut grid = generate_terrain(10, 10);
        // Set up a fault line
        grid.set(5, 5, TerrainType::FaultLine(false)); // false = closed
        app.insert_resource(grid);
        app.insert_resource(TidalForce {
            current: 0.8,
            base: 0.5,
        }); // High tide threshold > 0.7

        app.add_systems(Update, process_crustal_tides);
        app.update();

        let grid = app.world().resource::<TerrainGrid>();
        assert_eq!(grid.get(5, 5), Some(TerrainType::FaultLine(true))); // true = open (magma exposed)
    }

    #[test]
    fn test_low_tide_closes_fissures_and_damages_buildings() {
        let mut app = App::new();

        let mut grid = generate_terrain(10, 10);
        grid.set(5, 5, TerrainType::FaultLine(true)); // Start open
        app.insert_resource(grid);
        app.insert_resource(TidalForce {
            current: 0.2,
            base: 0.5,
        }); // Low tide threshold < 0.3

        let building = app
            .world_mut()
            .spawn((
                Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
                crate::layer1::map::GridPosition { x: 5, y: 5 },
            ))
            .id();

        app.add_systems(Update, process_crustal_tides);
        app.update();

        // Terrain should close
        let grid = app.world().resource::<TerrainGrid>();
        assert_eq!(grid.get(5, 5), Some(TerrainType::FaultLine(false)));

        // Building should take crush damage
        let health = app.world().get::<Health>(building).unwrap();
        assert!(
            health.current < 100.0,
            "Building should take damage when fissure closes"
        );
    }
}
