//! Ancient Structures and Heirloom Items.
//!
//! This module manages two distinct but related concepts from the Old World:
//!
//! 1. **Ancient Structures:** Unrepairable buildings (`AncientStructure`) that decay over time.
//!    These structures, such as the `AncientReactor`, can be sacrificed for knowledge via the
//!    `RetrogradeEngineeringEvent`.
//!
//! 2. **Heirlooms:** Exceptional tools (`Heirloom`) that have been used extensively.
//!    Tools track their usage history via `ToolHistory`, and when a threshold is met,
//!    they are promoted to Heirloom status, gaining a unique name and efficiency bonuses.

use crate::layer1::building::{
    can_place_building, spawn_building_with_material, BuildingType, MaterialType, OccupiedTiles,
};
use crate::layer1::structure::Structure;
use crate::layer1::terrain::TerrainGrid;
use bevy_ecs::prelude::*;

/// Component marking an entity as an Ancient Structure (formerly Heirloom Tech).
///
/// Ancient Structures are unrepairable structures from the Old World.
/// They decay over time and cannot be built by the player.
///
/// # Examples
///
/// ```
/// use bevy_ecs::prelude::*;
/// use scale::layer1::heirloom::AncientStructure;
/// use scale::layer1::structure::Structure;
///
/// let mut world = World::new();
/// let entity = world.spawn((
///     AncientStructure,
///     Structure { current_hp: 100.0, max_hp: 100.0 }
/// )).id();
///
/// assert!(world.get::<AncientStructure>(entity).is_some());
/// ```
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct AncientStructure;

/// Component tracking the history of a tool.
#[derive(Component, Debug, Default, Clone)]
pub struct ToolHistory {
    /// Number of ticks the tool has been used for work.
    pub ticks_used: u32,
    /// Number of items harvested/produced with this tool (optional metric).
    pub items_harvested: u32,
}

/// Component marking a tool as an Heirloom Item.
///
/// Heirloom items have a unique name and provide efficiency bonuses.
#[derive(Component, Debug, Clone)]
pub struct Heirloom {
    /// The unique name of the heirloom (e.g., "The Stone-Eater").
    pub name: String,
    /// The efficiency multiplier (e.g., 0.25 for +25%).
    pub efficiency_bonus: f32,
}

/// Event triggered when an Ancient Structure is sacrificed for knowledge.
#[derive(Event, Debug, Clone)]
pub struct RetrogradeEngineeringEvent {
    /// The label of the building destroyed (e.g. "Ancient Reactor").
    pub building_label: String,
    /// Amount of knowledge gained.
    pub knowledge_gained: f32,
}

/// Decays the HP of Ancient Structures over time.
pub fn ancient_structure_decay_system(mut query: Query<&mut Structure, With<AncientStructure>>) {
    const DECAY_RATE: f32 = 0.05;
    for mut structure in &mut query {
        structure.current_hp -= DECAY_RATE;
        if structure.current_hp < 0.0 {
            structure.current_hp = 0.0;
        }
    }
}

/// Checks if tools should be promoted to Heirloom status.
pub fn check_heirloom_status_system(
    mut commands: Commands,
    query: Query<(Entity, &ToolHistory), Without<Heirloom>>,
) {
    const HEIRLOOM_THRESHOLD: u32 = 1000;

    for (entity, history) in &query {
        if history.ticks_used >= HEIRLOOM_THRESHOLD {
            // Generate a cool name (placeholder for now)
            let name = format!("Legendary Tool #{}", entity.index());

            commands.entity(entity).insert(Heirloom {
                name,
                efficiency_bonus: 0.25, // +25% efficiency
            });

            // Optional: Log or notify about the new Heirloom
        }
    }
}

/// Spawns the initial Ancient Structures on the map.
pub fn spawn_ancient_structures(world: &mut World) {
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
