#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap
)]

use crate::layer1::beauty::BeautyGrid;
use crate::layer1::lighting::LightMap;
use crate::layer1::map::GridPosition;
use crate::layer1::needs::Needs;
use crate::layer1::terrain::TerrainGrid;
use bevy_ecs::prelude::*;
use rand::Rng;

/// Marker component for a ghost entity.
#[derive(Component, Default, Debug)]
pub struct Ghost;

/// Health component for a ghost.
///
/// Represents "spiritual stability".
#[derive(Component, Debug)]
pub struct Ectoplasm {
    /// Current stability. <= 0 means exorcism.
    pub current: f32,
    /// Max stability.
    pub max: f32,
}

impl Default for Ectoplasm {
    fn default() -> Self {
        Self {
            current: 100.0,
            max: 100.0,
        }
    }
}

/// Moves ghosts randomly. Ghosts ignore terrain collision (they float).
pub fn ghost_movement_system(
    mut query: Query<&mut GridPosition, With<Ghost>>,
    terrain: Res<TerrainGrid>,
) {
    let mut rng = rand::thread_rng();
    let width = i32::try_from(terrain.width).unwrap_or(i32::MAX);
    let height = i32::try_from(terrain.height).unwrap_or(i32::MAX);

    // Simple random walk: 10% chance to move each tick to keep them floaty and slow.
    for mut pos in &mut query {
        if rng.gen_bool(0.1) {
            let dx = rng.gen_range(-1..=1);
            let dy = rng.gen_range(-1..=1);

            let new_x = pos.x + dx;
            let new_y = pos.y + dy;

            if new_x >= 0 && new_y >= 0 && new_x < width && new_y < height {
                pos.x = new_x;
                pos.y = new_y;
            }
        }
    }
}

/// Ghosts take damage from light.
pub fn ghost_light_damage_system(
    mut commands: Commands,
    light_map: Res<LightMap>,
    mut ghosts: Query<(Entity, &GridPosition, &mut Ectoplasm), With<Ghost>>,
) {
    for (entity, pos, mut ectoplasm) in &mut ghosts {
        if let (Ok(x), Ok(y)) = (u32::try_from(pos.x), u32::try_from(pos.y)) {
            let light_level = light_map.get(x, y);

            // Threshold: Light > 0.2 causes damage.
            if light_level > 0.2 {
                // Damage scales with light intensity.
                // e.g., 1.0 light -> 1.0 damage per tick.
                let damage = light_level * 1.0;
                ectoplasm.current -= damage;

                if ectoplasm.current <= 0.0 {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

/// Ghosts make the world ugly.
pub fn apply_ghost_beauty_system(
    mut beauty_grid: ResMut<BeautyGrid>,
    ghosts: Query<&GridPosition, With<Ghost>>,
) {
    // This runs AFTER beauty grid is cleared and populated by buildings.
    for pos in &ghosts {
        if let (Ok(x), Ok(y)) = (usize::try_from(pos.x), usize::try_from(pos.y)) {
            let current = beauty_grid.get(x, y);
            // Strong negative beauty (-10 is like a Landfill)
            beauty_grid.set(x, y, current - 10.0);

            // Simple 3x3 aura
            for dy in -1..=1 {
                for dx in -1..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }

                    // Safe bounds check
                    let nx = x as isize + dx;
                    let ny = y as isize + dy;

                    if nx >= 0
                        && ny >= 0
                        && nx < beauty_grid.width as isize
                        && ny < beauty_grid.height as isize
                    {
                        let nx = nx as usize;
                        let ny = ny as usize;
                        let current = beauty_grid.get(nx, ny);
                        beauty_grid.set(nx, ny, current - 5.0);
                    }
                }
            }
        }
    }
}

/// Scares nearby pops, reducing their leisure and rest.
pub fn ghost_scare_system(
    ghosts: Query<&GridPosition, With<Ghost>>,
    mut pops: Query<(&GridPosition, &mut Needs), Without<Ghost>>,
) {
    // Collect ghost positions first to avoid O(N*M) checks if possible,
    // but N (ghosts) and M (pops) are likely small.
    // However, Query iteration inside Query iteration is fine for small numbers.
    let ghost_positions: Vec<GridPosition> = ghosts.iter().copied().collect();

    if ghost_positions.is_empty() {
        return;
    }

    for (pop_pos, mut needs) in &mut pops {
        for ghost_pos in &ghost_positions {
            let dist = (pop_pos.x - ghost_pos.x).abs() + (pop_pos.y - ghost_pos.y).abs();
            if dist < 4 {
                // Spooky range!
                // Reduce leisure (stress)
                needs.leisure = (needs.leisure - 0.005).max(0.0);
                // Reduce rest (disturbed)
                needs.rest = (needs.rest - 0.002).max(0.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    #[test]
    fn test_ghost_components() {
        let _ghost = Ghost;
        let ecto = Ectoplasm::default();
        assert_eq!(ecto.current, 100.0);
    }

    #[test]
    fn test_ghost_light_damage() {
        let mut world = World::new();
        let mut light_map = LightMap::new(10, 10);
        light_map.set(5, 5, 1.0); // Bright light
        world.insert_resource(light_map);

        let ghost = world
            .spawn((Ghost, Ectoplasm::default(), GridPosition { x: 5, y: 5 }))
            .id();

        // Run system
        world.run_system_once(ghost_light_damage_system).unwrap();

        // Should exist but be damaged
        if let Some(ecto) = world.get::<Ectoplasm>(ghost) {
            assert!(ecto.current < 100.0);
        } else {
            panic!("Ghost should not be despawned yet");
        }
    }

    #[test]
    fn test_ghost_exorcism() {
        let mut world = World::new();
        let mut light_map = LightMap::new(10, 10);
        light_map.set(5, 5, 1.0);
        world.insert_resource(light_map);

        let ghost = world
            .spawn((
                Ghost,
                Ectoplasm {
                    current: 0.5,
                    max: 100.0,
                }, // Almost dead
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        world.run_system_once(ghost_light_damage_system).unwrap();

        // Should be despawned
        assert!(world.get_entity(ghost).is_err());
    }

    #[test]
    fn test_ghost_beauty_aura() {
        let mut world = World::new();
        let beauty_grid = BeautyGrid::new(10, 10);
        world.insert_resource(beauty_grid);

        world.spawn((Ghost, GridPosition { x: 5, y: 5 }));

        world.run_system_once(apply_ghost_beauty_system).unwrap();

        let grid = world.resource::<BeautyGrid>();
        assert!(grid.get(5, 5) <= -10.0); // Center
        assert!(grid.get(4, 5) <= -5.0); // Neighbor
        assert_eq!(grid.get(0, 0), 0.0); // Far away
    }

    #[test]
    fn test_ghost_scare_system() {
        let mut world = World::new();

        // Spawn Ghost
        world.spawn((Ghost, GridPosition { x: 5, y: 5 }));

        // Spawn Pop nearby (3 tiles away: 2+1=3 < 4)
        let pop = world
            .spawn((
                GridPosition { x: 7, y: 6 },
                Needs {
                    leisure: 0.5,
                    rest: 0.5,
                    ..Default::default()
                },
            ))
            .id();

        // Spawn Pop far away
        let safe_pop = world
            .spawn((
                GridPosition { x: 0, y: 0 },
                Needs {
                    leisure: 0.5,
                    rest: 0.5,
                    ..Default::default()
                },
            ))
            .id();

        world.run_system_once(ghost_scare_system).unwrap();

        // Check scared pop
        let needs = world.get::<Needs>(pop).unwrap();
        assert!(needs.leisure < 0.5);
        assert!(needs.rest < 0.5);

        // Check safe pop
        let safe_needs = world.get::<Needs>(safe_pop).unwrap();
        assert!((safe_needs.leisure - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_ghost_movement() {
        use crate::layer1::terrain::{TerrainGrid, TerrainType};

        let mut world = World::new();
        // Setup small terrain
        let terrain = TerrainGrid {
            width: 3,
            height: 3,
            tiles: vec![TerrainType::Grass; 9],
        };
        world.insert_resource(terrain);

        // Spawn ghost at edge (2, 2)
        let ghost = world.spawn((Ghost, GridPosition { x: 2, y: 2 })).id();

        // Run system multiple times to try to force movement
        let mut schedule = Schedule::default();
        schedule.add_systems(ghost_movement_system);

        for _ in 0..50 {
            schedule.run(&mut world);
            // Check bounds
            let pos = world.get::<GridPosition>(ghost).unwrap();
            assert!(pos.x >= 0 && pos.x < 3);
            assert!(pos.y >= 0 && pos.y < 3);
        }
    }

    #[test]
    fn test_death_spawns_ghost() {
        use crate::layer1::health::{Health, death_system};
        use crate::layer1::map::GridPosition;

        let mut world = World::new();
        // Add necessary resources for death_system
        world.insert_resource(crate::shared::time::SimulationTime::default());
        world.insert_resource(crate::shared::log::MessageLog::default());

        // Spawn dying pop
        let pop = world
            .spawn((
                Health {
                    current: -10.0,
                    max: 100.0,
                },
                GridPosition { x: 10, y: 10 },
            ))
            .id();

        // Run death system
        death_system(&mut world);

        // Check if pop is gone
        assert!(world.get_entity(pop).is_err());

        // Check if ghost appeared at (10, 10)
        let mut query = world.query::<(&Ghost, &GridPosition)>();
        let ghosts: Vec<_> = query.iter(&world).collect();
        assert_eq!(ghosts.len(), 1);
        assert_eq!(ghosts[0].1.x, 10);
        assert_eq!(ghosts[0].1.y, 10);
    }
}
