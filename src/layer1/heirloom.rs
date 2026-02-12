use crate::layer1::building::{
    BuildingType, MaterialType, OccupiedTiles, can_place_building, spawn_building_with_material,
};
use crate::layer1::structure::Structure;
use crate::layer1::terrain::TerrainGrid;
use bevy_ecs::prelude::*;

/// Component marking an entity as an Heirloom.
///
/// Heirlooms are ancient, unrepairable structures from the Old World.
/// They decay over time and cannot be built by the player.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Heirloom;

/// Decays the HP of Heirloom structures over time.
pub fn heirloom_decay_system(mut query: Query<&mut Structure, With<Heirloom>>) {
    const DECAY_RATE: f32 = 0.05;
    for mut structure in &mut query {
        structure.current_hp -= DECAY_RATE;
        if structure.current_hp < 0.0 {
            structure.current_hp = 0.0;
        }
    }
}

/// Spawns the initial Heirloom structures on the map.
pub fn spawn_heirlooms(world: &mut World) {
    let (width, height) = {
        let grid = world.resource::<TerrainGrid>();
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        (grid.width as i32, grid.height as i32)
    };

    let center_x = width / 2;
    let center_y = height / 2;

    // Place Ancient Reactor
    if let Some((x, y)) = find_valid_spot(world, center_x, center_y, 15) {
        spawn_building_with_material(
            world,
            x,
            y,
            BuildingType::AncientReactor,
            MaterialType::default(),
        );
        world.resource_mut::<OccupiedTiles>().0.insert((x, y));

        if let Some(mut log) = world.get_resource_mut::<crate::shared::log::MessageLog>() {
            log.add(format!("Ancient Reactor located at ({x}, {y})"));
        }
    }

    // Place Ancient Fabricator (offset slightly)
    if let Some((x, y)) = find_valid_spot(world, center_x + 3, center_y + 3, 15) {
        spawn_building_with_material(
            world,
            x,
            y,
            BuildingType::AncientFabricator,
            MaterialType::default(),
        );
        world.resource_mut::<OccupiedTiles>().0.insert((x, y));

        if let Some(mut log) = world.get_resource_mut::<crate::shared::log::MessageLog>() {
            log.add(format!("Ancient Fabricator located at ({x}, {y})"));
        }
    }
}

fn find_valid_spot(world: &World, cx: i32, cy: i32, radius: i32) -> Option<(i32, i32)> {
    // Spiral search outward from center
    for r in 0..radius {
        for dy in -r..=r {
            for dx in -r..=r {
                // Only check the perimeter of the current radius to avoid re-checking inner tiles
                if dx.abs() != r && dy.abs() != r {
                    continue;
                }

                let x = cx + dx;
                let y = cy + dy;
                if can_place_building(world, x, y) {
                    return Some((x, y));
                }
            }
        }
    }
    None
}
