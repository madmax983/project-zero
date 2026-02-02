use super::needs::Needs;
use super::terrain::{TerrainGrid, TerrainType};
use bevy_ecs::prelude::*;
use rand::Rng;
use ratatui::style::Color;

/// Marker component for pop entities.
#[derive(Component)]
pub struct Pop;

/// Grid position in world space.
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct GridPosition {
    /// The X coordinate.
    pub x: i32,
    /// The Y coordinate.
    pub y: i32,
}

/// Spawn 5 initial pops at random walkable positions.
pub fn spawn_initial_pops(world: &mut World) {
    let mut rng = rand::thread_rng();
    spawn_initial_pops_internal(world, &mut rng);
}

fn spawn_initial_pops_internal<R: Rng>(world: &mut World, rng: &mut R) {
    // Get dimensions first to release borrow
    let (width, height) = {
        let terrain = world.resource::<TerrainGrid>();
        (terrain.width, terrain.height)
    };

    let mut spawned = 0;

    // Safety: we assume there is at least one walkable tile to avoid infinite loop.
    // In a real game we might want a timeout or more robust search.
    while spawned < 5 {
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let x = rng.gen_range(0..width as i32);
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let y = rng.gen_range(0..height as i32);

        // Check terrain type in a separate scope to handle borrowing
        let is_walkable = {
            let terrain = world.resource::<TerrainGrid>();
            #[allow(clippy::cast_sign_loss)]
            terrain
                .get(x as usize, y as usize)
                .is_some_and(|t| t != TerrainType::Water && t != TerrainType::Rock)
        };

        if is_walkable {
            world.spawn((Pop, GridPosition { x, y }, Needs::default()));
            spawned += 1;
        }
    }
}

const HEALTHY_THRESHOLD: f32 = 0.6;
const WARNING_THRESHOLD: f32 = 0.3;

/// Returns the character and color for rendering a pop.
#[must_use]
pub fn pop_display(needs: &Needs) -> (char, Color) {
    let health = needs.worst();
    if health > HEALTHY_THRESHOLD {
        ('☺', Color::Yellow)
    } else if health > WARNING_THRESHOLD {
        ('☻', Color::Rgb(255, 165, 0))
    } else {
        ('☹', Color::Red)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::needs::Needs;
    use crate::layer1::terrain::{TerrainGrid, TerrainType, generate_terrain};
    use ratatui::style::Color;

    #[test]
    fn test_pop_component_exists() {
        let mut world = World::new();
        let entity = world.spawn(Pop).id();

        assert!(world.get::<Pop>(entity).is_some());
    }

    #[test]
    fn test_grid_position_creation() {
        let pos = GridPosition { x: 5, y: 10 };
        assert_eq!(pos.x, 5);
        assert_eq!(pos.y, 10);
    }

    #[test]
    fn test_grid_position_is_copy() {
        let pos1 = GridPosition { x: 3, y: 7 };
        let pos2 = pos1; // Should copy, not move
        assert_eq!(pos1.x, pos2.x);
        assert_eq!(pos1.y, pos2.y);
    }

    #[test]
    fn test_grid_position_negative_coords() {
        let pos = GridPosition { x: -5, y: -10 };
        assert_eq!(pos.x, -5);
        assert_eq!(pos.y, -10);
    }

    #[test]
    fn test_spawn_initial_pops_count() {
        let mut world = World::new();
        let terrain = generate_terrain(80, 50);
        world.insert_resource(terrain);

        spawn_initial_pops(&mut world);

        let count = world.query::<&Pop>().iter(&world).count();
        assert_eq!(count, 5, "Should spawn exactly 5 pops");
    }

    #[test]
    fn test_spawn_initial_pops_have_positions() {
        let mut world = World::new();
        let terrain = generate_terrain(80, 50);
        world.insert_resource(terrain);

        spawn_initial_pops(&mut world);

        let mut query = world.query::<(&Pop, &GridPosition)>();
        let all_have_positions = query.iter(&world).count() == 5;
        assert!(all_have_positions, "All pops should have GridPosition");
    }

    #[test]
    fn test_spawn_only_on_walkable_terrain() {
        let mut world = World::new();
        let terrain = generate_terrain(80, 50);
        world.insert_resource(terrain);

        spawn_initial_pops(&mut world);

        // Initialize query before borrowing resource to avoid conflict
        let mut query = world.query::<(&Pop, &GridPosition)>();
        let terrain = world.resource::<TerrainGrid>();

        for (_, pos) in query.iter(&world) {
            let x = usize::try_from(pos.x).expect("Pop x should be non-negative");
            let y = usize::try_from(pos.y).expect("Pop y should be non-negative");
            if let Some(tile_type) = terrain.get(x, y) {
                assert_ne!(tile_type, TerrainType::Water, "Pop spawned on water");
                assert_ne!(tile_type, TerrainType::Rock, "Pop spawned on rock");
            }
        }
    }

    #[test]
    fn test_spawn_within_terrain_bounds() {
        let mut world = World::new();
        let terrain = generate_terrain(80, 50);
        world.insert_resource(terrain);

        spawn_initial_pops(&mut world);

        for (_, pos) in world.query::<(&Pop, &GridPosition)>().iter(&world) {
            assert!(pos.x >= 0 && pos.x < 80, "Pop x out of bounds");
            assert!(pos.y >= 0 && pos.y < 50, "Pop y out of bounds");
        }
    }

    #[test]
    fn test_pop_char_and_color() {
        // Test helper function for rendering pops
        let needs = Needs::default();
        let (ch, color) = pop_display(&needs);
        assert_eq!(ch, '☺');
        assert_eq!(color, Color::Yellow);
    }

    #[test]
    fn test_spawn_initial_pops_retries() {
        let mut world = World::new();
        let width = 10;
        let height = 10;
        let mut tiles = vec![TerrainType::Water; width * height];
        // Only one walkable tile
        tiles[0] = TerrainType::Grass;
        let terrain = TerrainGrid {
            width,
            height,
            tiles,
        };
        world.insert_resource(terrain);

        // Mock RNG could be used here, but for simplicity we rely on the fact
        // that with only 1/100 walkable tiles, the random generator WILL fail many times
        // before succeeding 5 times. This ensures the loop and 'if is_walkable' false path
        // are exercised.
        // We use a seeded RNG for determinism if possible, but standard RNG is fine for coverage.
        // To be safer and deterministic, we can use a SeedableRng if we import it,
        // but `rand::rngs::StdRng` requires a feature. `rand::rngs::mock::StepRng` isn't available.
        // We'll just run it. The probability of finding 5 spots in 5 tries on 1/100 map is 10^-10.
        // So retries are guaranteed.

        spawn_initial_pops(&mut world);

        let count = world.query::<&Pop>().iter(&world).count();
        assert_eq!(count, 5);

        // All pops should be at (0,0)
        for (_, pos) in world.query::<(&Pop, &GridPosition)>().iter(&world) {
            assert_eq!(pos.x, 0);
            assert_eq!(pos.y, 0);
        }
    }

    #[test]
    fn test_spawn_initial_pops_internal_with_custom_rng() {
        use rand::SeedableRng;
        use rand::rngs::StdRng;

        let mut world = World::new();
        let terrain = generate_terrain(80, 50);
        world.insert_resource(terrain);

        // Use seeded RNG for determinism
        let mut rng = StdRng::seed_from_u64(42);
        spawn_initial_pops_internal(&mut world, &mut rng);

        let count = world.query::<&Pop>().iter(&world).count();
        assert_eq!(count, 5, "Should spawn exactly 5 pops");
    }

    #[test]
    fn test_grid_position_debug() {
        let pos = GridPosition { x: 10, y: -5 };
        let debug_str = format!("{pos:?}");
        assert!(debug_str.contains("GridPosition"));
        assert!(debug_str.contains("10"));
    }

    #[test]
    fn test_grid_position_clone() {
        let pos1 = GridPosition { x: 7, y: 14 };
        #[allow(clippy::clone_on_copy)]
        let pos2 = pos1.clone();
        assert_eq!(pos1.x, pos2.x);
        assert_eq!(pos1.y, pos2.y);
    }

    #[test]
    fn test_spawn_with_mixed_terrain() {
        let mut world = World::new();
        let width = 10;
        let height = 10;
        let mut tiles = vec![TerrainType::Grass; width * height];

        // Add some non-walkable tiles to force retries
        tiles[5] = TerrainType::Water;
        tiles[15] = TerrainType::Rock;
        tiles[25] = TerrainType::Water;
        tiles[35] = TerrainType::Rock;

        let terrain = TerrainGrid {
            width,
            height,
            tiles,
        };
        world.insert_resource(terrain);

        spawn_initial_pops(&mut world);

        let count = world.query::<&Pop>().iter(&world).count();
        assert_eq!(count, 5);

        // Verify no pops on water or rock
        let mut query = world.query::<(&Pop, &GridPosition)>();
        let terrain = world.resource::<TerrainGrid>();
        for (_, pos) in query.iter(&world) {
            let x = usize::try_from(pos.x).expect("x should be non-negative");
            let y = usize::try_from(pos.y).expect("y should be non-negative");
            if let Some(tile) = terrain.get(x, y) {
                assert_ne!(tile, TerrainType::Water);
                assert_ne!(tile, TerrainType::Rock);
            }
        }
    }

    #[test]
    fn test_spawn_internal_multiple_attempts() {
        use rand::SeedableRng;
        use rand::rngs::StdRng;

        let mut world = World::new();
        let width = 20;
        let height = 20;
        let mut tiles = vec![TerrainType::Water; width * height];

        // Create a sparse walkable area (only 20 out of 400 tiles)
        for i in 0..20 {
            tiles[i * 20] = TerrainType::Grass;
        }

        let terrain = TerrainGrid {
            width,
            height,
            tiles,
        };
        world.insert_resource(terrain);

        let mut rng = StdRng::seed_from_u64(123);
        spawn_initial_pops_internal(&mut world, &mut rng);

        let count = world.query::<&Pop>().iter(&world).count();
        assert_eq!(count, 5);
    }

    #[test]
    fn test_spawn_on_grass() {
        let mut world = World::new();
        let tiles = vec![TerrainType::Grass; 100];
        let terrain = TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        };
        world.insert_resource(terrain);

        spawn_initial_pops(&mut world);

        let count = world.query::<&Pop>().iter(&world).count();
        assert_eq!(count, 5);
    }

    #[test]
    fn test_spawn_on_dirt() {
        let mut world = World::new();
        let tiles = vec![TerrainType::Dirt; 100];
        let terrain = TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        };
        world.insert_resource(terrain);

        spawn_initial_pops(&mut world);

        let count = world.query::<&Pop>().iter(&world).count();
        assert_eq!(count, 5);
    }

    #[test]
    fn test_grid_position_fields() {
        let pos = GridPosition { x: 42, y: -7 };
        assert_eq!(pos.x, 42);
        assert_eq!(pos.y, -7);

        let pos2 = GridPosition { x: 0, y: 0 };
        assert_eq!(pos2.x, 0);
        assert_eq!(pos2.y, 0);
    }

    #[test]
    fn test_pop_component_on_entity() {
        let mut world = World::new();
        let entity = world.spawn((Pop, GridPosition { x: 1, y: 2 })).id();

        assert!(world.get::<Pop>(entity).is_some());
        let pos = world.get::<GridPosition>(entity).unwrap();
        assert_eq!(pos.x, 1);
        assert_eq!(pos.y, 2);
    }
}
