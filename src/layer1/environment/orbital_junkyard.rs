use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct ScrapPile {
    pub amount: u32,
}

#[derive(Component)]
pub struct FallingDebris {
    pub damage: f32,
}

#[derive(Resource)]
pub struct DebrisRainChance(pub f32);

#[allow(clippy::cast_possible_wrap)]
pub fn process_debris_rain(
    mut commands: Commands,
    grid: Option<Res<crate::layer1::nature::terrain::TerrainGrid>>,
    chance: Option<Res<DebrisRainChance>>,
) {
    if let (Some(chance), Some(grid)) = (chance, grid) {
        if rand::random::<f32>() < chance.0 && grid.width > 0 && grid.height > 0 {
            let x = rand::random::<usize>() % grid.width;
            let y = rand::random::<usize>() % grid.height;

            // Spawn falling debris which resolves next tick
            commands.spawn((
                crate::layer1::core::map::GridPosition {
                    x: x as i32,
                    y: y as i32,
                },
                FallingDebris { damage: 50.0 }, // Base random damage
            ));
        }
    }
}

pub fn process_falling_debris(
    mut commands: Commands,
    debris_query: Query<(
        Entity,
        &crate::layer1::core::map::GridPosition,
        &FallingDebris,
    )>,
    mut structure_query: Query<
        (
            Entity,
            &crate::layer1::core::map::GridPosition,
            &mut crate::layer1::biology::health::Health,
        ),
        With<crate::layer1::architecture::structure::Structure>,
    >,
) {
    for (debris_entity, debris_pos, debris) in debris_query.iter() {
        let mut hit_structure = false;

        for (struct_entity, struct_pos, mut health) in structure_query.iter_mut() {
            if struct_pos.x == debris_pos.x && struct_pos.y == debris_pos.y {
                health.take_damage(debris.damage);
                hit_structure = true;

                if !health.is_alive() {
                    commands.entity(struct_entity).despawn();
                }
            }
        }

        if !hit_structure {
            // Spawn scrap on empty tiles
            commands.spawn((
                crate::layer1::core::map::GridPosition {
                    x: debris_pos.x,
                    y: debris_pos.y,
                },
                ScrapPile { amount: 10 },
            ));
        }

        // Remove the falling debris hazard
        commands.entity(debris_entity).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::architecture::structure::Structure;
    use crate::layer1::biology::health::Health;
    use crate::layer1::core::map::GridPosition;
    use crate::layer1::nature::terrain::{TerrainGrid, TerrainType};
    use bevy::prelude::*;

    #[test]
    fn test_debris_rain_spawn() {
        let mut app = App::new();
        app.add_systems(
            Update,
            (process_debris_rain, process_falling_debris).chain(),
        );

        app.world_mut().insert_resource(DebrisRainChance(1.0)); // Always rain
        app.world_mut().insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Dirt; 100],
        });

        app.update();

        let mut query = app.world_mut().query::<&ScrapPile>();
        let scrap_count = query.iter(app.world()).count();
        assert!(
            scrap_count > 0,
            "Debris should have spawned a scrap pile on the grid"
        );
    }

    #[test]
    fn test_debris_crushes_building() {
        let mut app = App::new();
        app.add_systems(Update, process_falling_debris);

        let building_pos = GridPosition { x: 5, y: 5 };
        let building = app
            .world_mut()
            .spawn((
                building_pos,
                Structure {
                    current_hp: 100.0,
                    max_hp: 100.0,
                },
                Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
            ))
            .id();

        let _debris = app
            .world_mut()
            .spawn((
                GridPosition { x: 5, y: 5 },
                FallingDebris { damage: 150.0 }, // Fatal damage
            ))
            .id();

        app.update();

        assert!(
            app.world().get::<Health>(building).is_none(),
            "Building should have been destroyed by falling debris"
        );
    }

    #[test]
    fn test_debris_damages_building_but_not_destroyed() {
        let mut app = App::new();
        app.add_systems(Update, process_falling_debris);

        let building_pos = GridPosition { x: 5, y: 5 };
        let building = app
            .world_mut()
            .spawn((
                building_pos,
                Structure {
                    current_hp: 100.0,
                    max_hp: 100.0,
                },
                Health {
                    current: 100.0,
                    max: 100.0,
                    has_rust_lung: false,
                },
            ))
            .id();

        let _debris = app
            .world_mut()
            .spawn((
                GridPosition { x: 5, y: 5 },
                FallingDebris { damage: 50.0 }, // Non-fatal damage
            ))
            .id();

        app.update();

        let health = app.world().get::<Health>(building).unwrap();
        assert_eq!(health.current, 50.0, "Building should have taken 50 damage");

        let mut scrap_query = app.world_mut().query::<&ScrapPile>();
        assert_eq!(
            scrap_query.iter(app.world()).count(),
            0,
            "Should not spawn scrap on hit"
        );
    }

    #[test]
    fn test_no_rain_when_chance_is_zero() {
        let mut app = App::new();
        app.add_systems(Update, process_debris_rain);

        app.world_mut().insert_resource(DebrisRainChance(0.0)); // Never rain
        app.world_mut().insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Dirt; 100],
        });

        app.update();

        let mut query = app.world_mut().query::<&FallingDebris>();
        let debris_count = query.iter(app.world()).count();
        assert_eq!(debris_count, 0, "No debris should spawn when chance is 0.0");
    }

    #[test]
    fn test_zero_size_grid_handled() {
        let mut app = App::new();
        app.add_systems(Update, process_debris_rain);

        app.world_mut().insert_resource(DebrisRainChance(1.0)); // Always rain
        app.world_mut().insert_resource(TerrainGrid {
            width: 0,
            height: 0,
            tiles: vec![],
        });

        app.update();

        let mut query = app.world_mut().query::<&FallingDebris>();
        let debris_count = query.iter(app.world()).count();
        assert_eq!(debris_count, 0, "No debris should spawn on an empty grid");
    }
}
