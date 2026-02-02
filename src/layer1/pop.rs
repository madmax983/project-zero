use super::terrain::{TerrainGrid, TerrainType};
use bevy_ecs::prelude::*;
use rand::Rng;
use ratatui::style::Color;

/// Marker component for pop entities.
#[derive(Component)]
pub struct Pop;

/// Grid position in world space.
#[derive(Component, Clone, Copy, Debug)]
pub struct GridPosition {
    /// The X coordinate.
    pub x: i32,
    /// The Y coordinate.
    pub y: i32,
}

/// Spawn 5 initial pops at random walkable positions.
pub fn spawn_initial_pops(world: &mut World) {
    // Get dimensions first to release borrow
    let (width, height) = {
        let terrain = world.resource::<TerrainGrid>();
        (terrain.width, terrain.height)
    };

    let mut rng = rand::thread_rng();
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
            world.spawn((Pop, GridPosition { x, y }));
            spawned += 1;
        }
    }
}

/// Returns the character and color for rendering a pop.
#[must_use]
pub const fn pop_display() -> (char, Color) {
    ('☺', Color::Yellow)
}

#[cfg(test)]
mod tests {
    use super::*;
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
        let (ch, color) = pop_display();
        assert_eq!(ch, '☺');
        assert_eq!(color, Color::Yellow);
    }
}
