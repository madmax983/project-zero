//! Explosive Decompression & Suction Dynamics.
//!
//! This module handles the physical consequence of sudden pressure drops. When a sealed
//! environment is breached (e.g., a wall breaks next to the vacuum of space), the violent
//! outgassing creates a severe pressure gradient.
//!
//! `suction_system` forces entities (like pops or items) from high-pressure cells toward
//! adjacent low-pressure cells if the difference exceeds the suction threshold.
//!
//! # Examples
//!
//! ```
//! use bevy_ecs::prelude::*;
//! use scale::layer1::pressure::PressureGrid;
//! use scale::layer1::suction::suction_system;
//! use scale::layer1::pop::Pop;
//! use scale::layer1::map::GridPosition;
//!
//! let mut world = World::new();
//!
//! // 1. Create a pressurized grid, but leave a vacuum gap
//! let mut grid = PressureGrid::new(10, 10);
//! grid.fill(1.0);     // 1.0 = Atmosphere
//! grid.set(6, 5, 0.0); // 0.0 = Vacuum Breach!
//! world.insert_resource(grid);
//!
//! // 2. Spawn a Pop next to the breach
//! let pop = world.spawn((
//!     Pop,
//!     GridPosition { x: 5, y: 5 }
//! )).id();
//!
//! // 3. Run the decompression logic
//! let mut schedule = Schedule::default();
//! schedule.add_systems(suction_system);
//! schedule.run(&mut world);
//!
//! // The unfortunate pop has been violently sucked into the vacuum cell!
//! let new_pos = world.get::<GridPosition>(pop).unwrap();
//! assert_eq!(new_pos.x, 6);
//! assert_eq!(new_pos.y, 5);
//! ```
//!
//! # Edge Cases
//! - **Structural Integrity:** Heavy, anchored entities (e.g., [`Building`] components like Walls
//!   or Airlocks) are immune to suction forces. They do not move, regardless of the gradient.
//! - **Equilibrium:** Slight variances in atmospheric pressure (< `SUCTION_THRESHOLD`) do not
//!   cause entities to drift; they only get sucked if the drop is massive (e.g., > 0.5).

use crate::layer1::building::Building;
use crate::layer1::items::Item;
use crate::layer1::map::GridPosition;
use crate::layer1::pop::Pop;
use crate::layer1::pressure::PressureGrid;
use bevy_ecs::prelude::*;
use bevy::utils::HashSet;

/// System to simulate explosive decompression suction.
/// Entities are moved from high pressure to low pressure if the gradient is steep enough.
#[allow(clippy::type_complexity)]
/// ⚡ Bolt Optimization: Lazy initialization of `building_positions` HashSet.
/// Defers an O(B) allocation and iteration over all buildings until a suction event actually occurs.
/// Saves 1 heap allocation and O(B) loop cycles per frame/tick when no explosive decompression is happening.
pub fn suction_system(
    mut _commands: Commands,
    pressure: Res<PressureGrid>,
    mut query: Query<(Entity, &mut GridPosition), Or<(With<Pop>, With<Item>)>>,
    buildings: Query<&GridPosition, (With<Building>, Without<Pop>, Without<Item>)>,
) {
    // Pressure difference required to move an entity
    const SUCTION_THRESHOLD: f32 = 0.5;

    // ⚡ Bolt Optimization: Lazy initialization of `building_positions` HashSet.
    // Defers an O(B) allocation and iteration over all buildings until a suction event actually occurs.
    // Saves 1 heap allocation and O(B) loop cycles per frame/tick when no explosive decompression is happening.
    let mut building_positions: Option<HashSet<(i32, i32)>> = None;

    for (_entity, mut pos) in &mut query {
        let current_p = pressure.get(pos.x, pos.y);

        // Check 4 neighbors
        let neighbors = [(0, 1), (0, -1), (1, 0), (-1, 0)];
        let mut best_target = None;
        let mut max_diff = 0.0;

        for (dx, dy) in &neighbors {
            let nx = pos.x + dx;
            let ny = pos.y + dy;

            // Bounds check
            if nx < 0 || ny < 0 {
                continue;
            }
            // Safe cast since we checked < 0
            #[allow(clippy::cast_sign_loss)]
            if (nx as usize) >= pressure.width || (ny as usize) >= pressure.height {
                continue;
            }

            let neighbor_p = pressure.get(nx, ny);
            let diff = current_p - neighbor_p;

            // Must differ by threshold AND be the strongest pull found so far
            if diff > SUCTION_THRESHOLD && diff > max_diff {
                max_diff = diff;
                best_target = Some((nx, ny));
            }
        }

        if let Some((tx, ty)) = best_target {
            // Check for collision with buildings
            let bps = building_positions
                .get_or_insert_with(|| buildings.iter().map(|p| (p.x, p.y)).collect());

            if !bps.contains(&(tx, ty)) {
                // Move entity
                pos.x = tx;
                pos.y = ty;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::health::Health;
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop;
    use crate::layer1::pressure::PressureGrid;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_entity_sucked_into_vacuum() {
        let mut world = World::new();
        let mut grid = PressureGrid::new(10, 10);
        grid.fill(1.0); // Start pressurized

        // Setup: (5,5) is High Pressure (1.0), (6,5) is Vacuum (0.0)
        grid.set(6, 5, 0.0);
        world.insert_resource(grid);

        // Spawn Pop at (5,5)
        let pop = world
            .spawn((Pop, GridPosition { x: 5, y: 5 }, Health::default()))
            .id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(super::suction_system);
        schedule.run(&mut world);

        // Assert Pop moved to (6,5)
        let pos = world.get::<GridPosition>(pop).unwrap();
        assert_eq!(
            *pos,
            GridPosition { x: 6, y: 5 },
            "Pop should be sucked into vacuum"
        );
    }

    #[test]
    fn test_anchored_building_not_moved() {
        let mut world = World::new();
        let mut grid = PressureGrid::new(10, 10);
        grid.fill(1.0);

        grid.set(6, 5, 0.0);
        world.insert_resource(grid);

        // Spawn Wall at (5,5) - Walls are anchored/immovable
        let wall = world
            .spawn((
                Building {
                    building_type: BuildingType::Wall,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(super::suction_system);
        schedule.run(&mut world);

        let pos = world.get::<GridPosition>(wall).unwrap();
        assert_eq!(
            *pos,
            GridPosition { x: 5, y: 5 },
            "Building should NOT move"
        );
    }

    #[test]
    fn test_no_suction_at_equilibrium() {
        let mut world = World::new();
        let mut grid = PressureGrid::new(10, 10);
        grid.fill(1.0);

        grid.set(6, 5, 0.9); // Small difference
        world.insert_resource(grid);

        let pop = world.spawn((Pop, GridPosition { x: 5, y: 5 })).id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(super::suction_system);
        schedule.run(&mut world);

        let pos = world.get::<GridPosition>(pop).unwrap();
        assert_eq!(
            *pos,
            GridPosition { x: 5, y: 5 },
            "Small gradient should not cause suction"
        );
    }
}
