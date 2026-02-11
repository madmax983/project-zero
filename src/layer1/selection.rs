use crate::layer1::{
    GridPosition, TerrainGrid, Viewport, building::Building, health::Health, needs::Needs, pop::Pop,
};
use crate::shared::selection::Selection;
use bevy_ecs::prelude::*;
use std::fmt::Write;

use crate::platform::input::GameMouseEvent;

/// Generate info panel text for a selected tile.
#[must_use]
#[allow(clippy::cast_sign_loss)] // Coordinate checking handles negative values
pub fn inspect_tile(world: &World, x: i32, y: i32) -> String {
    let terrain = world.resource::<TerrainGrid>();

    if x < 0 || y < 0 {
        return String::from("Empty space\n(outside map)");
    }

    terrain.get(x as usize, y as usize).map_or_else(
        || String::from("Empty space\n(outside map)"),
        |tile| {
            format!(
                "Tile ({x}, {y})\n\nTerrain: {}\n\n(Click entity for details)",
                tile.name()
            )
        },
    )
}

/// Generate info panel text for a selected entity.
#[must_use]
pub fn inspect_entity(world: &World, entity: Entity) -> String {
    // Check if entity exists
    if !world.entities().contains(entity) {
        return String::from("Entity not found");
    }

    let entity_type = if world.get::<Pop>(entity).is_some() {
        "Pop"
    } else if let Some(b) = world.get::<Building>(entity) {
        b.building_type.label()
    } else {
        "Entity"
    };

    // Try to get Position
    if let Some(pos) = world.get::<GridPosition>(entity) {
        let mut info = format!("{entity_type}\nPosition: ({}, {})\n\n", pos.x, pos.y);

        if let Some(health) = world.get::<Health>(entity) {
            let _ = writeln!(info, "Health: {:.0}%", health.current);
        }

        if let Some(needs) = world.get::<Needs>(entity) {
            let _ = write!(
                info,
                "Hunger: {:.0}%\nRest: {:.0}%\n",
                needs.hunger * 100.0,
                needs.rest * 100.0
            );
        }

        return info;
    }

    String::from("Unknown entity")
}

/// Convert screen coordinates to world coordinates.
#[must_use]
pub const fn screen_to_world(screen_x: u16, screen_y: u16, viewport: &Viewport) -> (i32, i32) {
    let world_x = viewport.x.wrapping_add(screen_x as i32);
    let world_y = viewport.y.wrapping_add(screen_y as i32);
    (world_x, world_y)
}

/// Handle mouse click for selection.
pub fn handle_selection_click(world: &mut World, mouse: GameMouseEvent, viewport: &Viewport) {
    let (world_x, world_y) = screen_to_world(mouse.x, mouse.y, viewport);

    // Find all entities at position
    let mut candidates = Vec::new();
    let mut query = world.query::<(Entity, &GridPosition)>();
    for (entity, pos) in query.iter(world) {
        if pos.x == world_x && pos.y == world_y {
            candidates.push(entity);
        }
    }

    // Prioritize: Pop > Building > Any
    let mut selected_entity = None;

    // Check for Pop
    for &entity in &candidates {
        if world.get::<Pop>(entity).is_some() {
            selected_entity = Some(entity);
            break;
        }
    }

    // If no Pop, check for Building
    if selected_entity.is_none() {
        for &entity in &candidates {
            if world.get::<Building>(entity).is_some() {
                selected_entity = Some(entity);
                break;
            }
        }
    }

    // Fallback to first candidate
    if selected_entity.is_none() {
        selected_entity = candidates.first().copied();
    }

    // Update selection
    let mut selection = world.resource_mut::<Selection>();
    if let Some(entity) = selected_entity {
        selection.select_entity(entity);
    } else {
        selection.select_tile(world_x, world_y);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::{GridPosition, Needs, Pop, TerrainGrid, TerrainType};

    #[test]
    fn test_inspect_tile_with_terrain() {
        let mut world = World::new();
        let terrain = TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        };
        world.insert_resource(terrain);

        let info = inspect_tile(&world, 5, 5);

        assert!(info.contains("Grass"));
        assert!(info.contains('5'));
    }

    #[test]
    fn test_inspect_tile_out_of_bounds() {
        let mut world = World::new();
        let terrain = TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        };
        world.insert_resource(terrain);

        let info = inspect_tile(&world, 100, 100);

        assert!(info.contains("Empty") || info.contains("Nothing"));
    }

    #[test]
    fn test_inspect_entity_pop() {
        let mut world = World::new();
        let entity = world
            .spawn((
                Pop,
                GridPosition { x: 10, y: 5 },
                Needs {
                    hunger: 0.75,
                    rest: 0.50,
                    ..Default::default()
                },
            ))
            .id();

        let info = inspect_entity(&world, entity);

        assert!(info.contains("Pop"));
        assert!(info.contains("10"));
        assert!(info.contains("hunger") || info.contains("Hunger"));
    }

    #[test]
    fn test_inspect_entity_nonexistent() {
        let world = World::new();
        let fake_entity = Entity::from_raw(999_999);

        let info = inspect_entity(&world, fake_entity);

        assert!(info.contains("not found") || info.contains("None"));
    }

    #[test]
    fn test_click_to_screen_pos() {
        let viewport = Viewport { x: 10, y: 5 };
        let screen_x = 3;
        let screen_y = 2;

        let (world_x, world_y) = screen_to_world(screen_x, screen_y, &viewport);

        assert_eq!(world_x, 13); // 10 + 3
        assert_eq!(world_y, 7); // 5 + 2
    }

    #[test]
    fn test_screen_to_world_overflow() {
        // Test potential overflow safety
        let viewport = Viewport {
            x: i32::MAX,
            y: i32::MAX,
        };
        let screen_x = 10;
        let screen_y = 10;

        // Should not panic
        let (world_x, world_y) = screen_to_world(screen_x, screen_y, &viewport);

        // Should wrap around
        assert_eq!(world_x, i32::MAX.wrapping_add(10));
        assert_eq!(world_y, i32::MAX.wrapping_add(10));
    }
}
