# 1118: The Tether Stump

## 1. Overview

This feature introduces a massive, indestructible structure called the "Space Elevator Base" (or "The Tether Stump") during map generation in the Layer 1 colony simulation. It provides a unique geographic focal point with an infinite vertical build height but a very small footprint. Ascending its heights yields "Lost Tech", but the thin upper atmosphere requires life support, creating a tension between building density and safety (e.g., trapping colonists above a fire at the base).

## 2. Dependencies

- None (core map generation logic exists, this builds upon it).

## 3. RED Phase: Tests First

```rust
// src/layer1/systems/map_generation/tether_stump_tests.rs

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use crate::layer1::map::{TerrainGrid, TerrainType};
    use crate::layer1::atmosphere::PressureGrid;

    #[test]
    fn test_tether_stump_generation() {
        // Arrange
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        // Assuming map generation plugin exists
        app.add_systems(Startup, generate_tether_stump);

        let mut terrain = TerrainGrid::new(100, 100);
        app.insert_resource(terrain);

        // Act
        app.update();

        // Assert
        let terrain = app.world().resource::<TerrainGrid>();

        // Find the stump - should be a specific small footprint
        let mut stump_tiles = 0;
        for x in 0..100 {
            for y in 0..100 {
                if terrain.get(x, y) == TerrainType::IndestructibleStump {
                    stump_tiles += 1;
                }
            }
        }

        // Assert footprint is small (e.g., 2x2 or 3x3)
        assert!(stump_tiles > 0 && stump_tiles <= 9, "Stump should have a small footprint");
    }

    #[test]
    fn test_vertical_build_height_is_infinite_at_stump() {
        // Arrange
        let mut app = App::new();
        app.insert_resource(TerrainGrid::new(100, 100));
        app.add_systems(Startup, generate_tether_stump);
        app.update();

        // Act & Assert
        let terrain = app.world().resource::<TerrainGrid>();
        let (stump_x, stump_y) = find_stump_center(&terrain).unwrap();

        // Verify we can build at extreme heights at the stump coordinates
        assert_eq!(terrain.get_max_build_height(stump_x, stump_y), u32::MAX);
    }

    #[test]
    fn test_atmosphere_pressure_drops_with_height() {
        // Arrange
        let mut app = App::new();
        let mut pressure_grid = PressureGrid::new(100, 100, 100); // x, y, z

        // Act: Initialize atmosphere
        initialize_atmosphere_gradient(&mut pressure_grid);

        // Assert
        let ground_pressure = pressure_grid.get(50, 50, 0);
        let high_altitude_pressure = pressure_grid.get(50, 50, 50);

        assert!(high_altitude_pressure < ground_pressure, "Pressure should drop with altitude");
        assert!(high_altitude_pressure < 0.5, "High altitude should be dangerously thin"); // Assuming 1.0 is normal
    }

    #[test]
    fn test_lost_tech_spawns_at_high_altitude() {
        // Arrange
        let mut app = App::new();
        app.insert_resource(TerrainGrid::new(100, 100));
        app.add_systems(Startup, (generate_tether_stump, spawn_lost_tech_caches).chain());

        // Act
        app.update();

        // Assert
        let tech_query = app.world_mut().query::<(&Transform, &LostTech)>();
        let tech_count = tech_query.iter(app.world()).count();

        assert!(tech_count > 0, "Should spawn lost tech");

        for (transform, _) in tech_query.iter(app.world()) {
            assert!(transform.translation.z > 20.0, "Lost tech should only spawn at high altitude");
        }
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/systems/map_generation/tether_stump.rs

use bevy::prelude::*;
use crate::layer1::map::{TerrainGrid, TerrainType};
use crate::layer1::atmosphere::PressureGrid;

#[derive(Component)]
pub struct LostTech;

pub fn generate_tether_stump(mut terrain: ResMut<TerrainGrid>) {
    // Hardcoded center for minimal implementation
    let center_x = terrain.width() / 2;
    let center_y = terrain.height() / 2;

    // Create a 2x2 stump
    for x in center_x..center_x+2 {
        for y in center_y..center_y+2 {
            terrain.set(x, y, TerrainType::IndestructibleStump);
            // In the real TerrainGrid, we might need to set max_height per tile
            terrain.set_max_height(x, y, u32::MAX);
        }
    }
}

pub fn initialize_atmosphere_gradient(pressure_grid: &mut PressureGrid) {
    for x in 0..pressure_grid.width() {
        for y in 0..pressure_grid.height() {
            for z in 0..pressure_grid.depth() {
                // Simple linear drop-off for minimal implementation
                let pressure = 1.0 - (z as f32 / 100.0).clamp(0.0, 1.0);
                pressure_grid.set(x, y, z, pressure);
            }
        }
    }
}

pub fn spawn_lost_tech_caches(mut commands: Commands, terrain: Res<TerrainGrid>) {
    let (stump_x, stump_y) = find_stump_center(&terrain).unwrap_or((50, 50));

    // Spawn one piece of tech high up
    commands.spawn((
        LostTech,
        Transform::from_xyz(stump_x as f32, stump_y as f32, 50.0),
    ));
}

// Helper for minimal implementation
fn find_stump_center(terrain: &TerrainGrid) -> Option<(u32, u32)> {
    for x in 0..terrain.width() {
        for y in 0..terrain.height() {
            if terrain.get(x, y) == TerrainType::IndestructibleStump {
                return Some((x, y));
            }
        }
    }
    None
}
```

## 5. REFACTOR Phase: Quality & Design

- **Map Seed Support:** The stump's location should be driven by the map's random seed, not hardcoded to the center.
- **Atmosphere Model:** The linear pressure drop-off is naive. Consider an exponential decay to model real atmospheric pressure more accurately, creating a sharp threshold where pressure suits become mandatory.
- **Lost Tech Integration:** `LostTech` components should integrate with the game's inventory and research systems, rather than just being empty markers.
- **Pathfinding & AI:** Ensure the utility AI and pathfinding can handle extreme verticality without performance degradation. A* cost might need tuning for vertical movement.

## 6. Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Map generation successfully places an indestructible stump with a small footprint.
- [ ] Pops can build vertically at the stump without a height limit.
- [ ] Atmospheric pressure decreases with height on the grid.
- [ ] "Lost Tech" items spawn exclusively at high altitudes on or near the stump.

## 7. Technical Guidance

- **Grid Modification:** `TerrainGrid` and `PressureGrid` might need to be expanded to support 3D coordinates fully if they are currently strictly 2D. Be cautious of memory usage if moving to a full 3D voxel grid; consider sparse voxel octrees or chunking if the vertical dimension is truly "infinite".
- **Rendering:** Ensure the ratatui UI can visualize multiple vertical layers, perhaps through a "slice" view or Z-level up/down controls.
- **Fire Propagation:** For the tension (fire trapping people), ensure the fire simulation respects verticality and can spread upwards but consumes oxygen, potentially self-extinguishing in the thin upper atmosphere.

## 8. Questions
*Builder: add questions here if spec is unclear.*
The spec suggests expanding `TerrainGrid` and `PressureGrid` to 3D. However, checking the codebase reveals these are strictly 2D structures deeply integrated into numerous Layer 1 systems (pathfinding, temperature, erosion, etc.). Modifying them to support full 3D would require rewriting almost the entire layer. Can we simulate the extreme height via an auxiliary component or grid localized to the stump instead of converting the base grids to 3D?
