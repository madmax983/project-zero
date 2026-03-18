# 515: Deep Crust Geomes

## Overview

**Layer:** 1

**Fantasy:** There are worlds within worlds. Digging too deep and finding something beautiful and terrible.

**Mechanic:** As the colony expands downwards across Z-levels, deep mining layers contain unique subterranean biomes (Geomes) such as "Sunless Seas", "Magma Rivers", or "Crystal Forests". These distinct areas contain rare resources but bring unique environmental hazards like flooding, extreme heat, or toxic spores.

**Emergence:** You breach a cavern wall and drain a subterranean lake into your lower mines, drowning the night shift but revealing a rich vein of precursor alloy.

**Tension:** Safe surface expansion vs. High-risk deep exploration.

## Dependencies

- `002` — Terrain Grid (for Z-levels and basic terrain handling)
- `018` — Mining Resources (for digging and revealing terrain)
- `412` — Fluid Simulation (if implemented, for water/magma dynamics, otherwise simplified hazard tiles)
- `034` — Pop Health (for environmental damage like heat or spores)

## RED Phase: Tests First

```rust
// src/layer1/geome_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::map::{TerrainGrid, TerrainType, GridPosition, ZLevel};
    use crate::layer1::geomes::{GeomeManager, GeomeType, GeomeHazard};
    use crate::layer1::mining::{mine_tile_system, MineAction};
    use crate::layer1::health::{Health, environmental_damage_system};

    #[test]
    fn test_deep_crust_geome_generation() {
        let mut world = World::new();

        // Setup TerrainGrid with a deep Z-level
        let mut grid = TerrainGrid::new(10, 10, 5); // 10x10x5 (x,y,z)
        world.insert_resource(grid);

        let mut geome_manager = GeomeManager::new();
        // Force spawn a Magma River geome at Z-level 4
        geome_manager.spawn_geome(GeomeType::MagmaRiver, ZLevel(-4), Rect::new(2, 2, 5, 5));
        world.insert_resource(geome_manager);

        // Verify terrain type at the deep level has been modified by the geome
        let grid = world.get_resource::<TerrainGrid>().unwrap();
        // Assuming the geome manager overrides the basic rock with geome-specific terrain
        assert_eq!(grid.get(GridPosition { x: 3, y: 3, z: -4 }), Some(TerrainType::MagmaRock));
    }

    #[test]
    fn test_breaching_geome_triggers_hazard() {
        let mut world = World::new();

        // Setup Map where mining a specific tile opens up a Toxic Spore geome
        let mut grid = TerrainGrid::new(10, 10, 5);
        grid.set(GridPosition { x: 5, y: 5, z: -3 }, TerrainType::SolidRock); // The wall
        grid.set(GridPosition { x: 6, y: 5, z: -3 }, TerrainType::SporeBloom); // The geome inside
        world.insert_resource(grid);

        // Spawn a miner pop
        let miner = world.spawn((
            Pop,
            Health { current: 100.0, max: 100.0 },
            GridPosition { x: 4, y: 5, z: -3 }, // Next to the wall
        )).id();

        // Simulate mining the wall
        world.spawn(MineAction { target: GridPosition { x: 5, y: 5, z: -3 } });

        // Run mining system to breach the wall
        let mut schedule = Schedule::default();
        schedule.add_systems(mine_tile_system);
        schedule.run(&mut world);

        // Run hazard diffusion system (assuming SporeBloom spreads hazard to adjacent empty tiles)
        let mut hazard_schedule = Schedule::default();
        hazard_schedule.add_systems(crate::layer1::geomes::diffuse_geome_hazards_system);
        hazard_schedule.run(&mut world);

        // Run environmental damage system
        let mut damage_schedule = Schedule::default();
        damage_schedule.add_systems(environmental_damage_system);
        damage_schedule.run(&mut world);

        // The miner should have taken damage from the newly exposed Spore hazard
        let health = world.get::<Health>(miner).unwrap();
        assert!(health.current < 100.0, "Miner should take damage from breached geome hazard");
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
// src/layer1/geomes.rs
use bevy_ecs::prelude::*;
use crate::layer1::map::{TerrainGrid, TerrainType, GridPosition, ZLevel};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeomeType {
    SunlessSea,
    MagmaRiver,
    CrystalForest,
    SporeCavern,
}

#[derive(Resource)]
pub struct GeomeManager {
    // Stores geome regions for map generation logic
    regions: Vec<(GeomeType, ZLevel, Rect)>,
}

impl GeomeManager {
    pub fn new() -> Self {
        Self { regions: Vec::new() }
    }

    pub fn spawn_geome(&mut self, geome_type: GeomeType, z: ZLevel, area: Rect) {
        self.regions.push((geome_type, z, area));
    }

    // In a real implementation, this would be called during map generation
    pub fn apply_to_grid(&self, grid: &mut TerrainGrid) {
        for (g_type, z, area) in &self.regions {
            for x in area.min.x..area.max.x {
                for y in area.min.y..area.max.y {
                    let pos = GridPosition { x, y, z: z.0 };
                    let terrain = match g_type {
                        GeomeType::MagmaRiver => TerrainType::MagmaRock,
                        GeomeType::SporeCavern => TerrainType::SporeBloom,
                        _ => TerrainType::DeepRock,
                    };
                    grid.set(pos, terrain);
                }
            }
        }
    }
}

// Simple struct for representing a 2D area
pub struct Rect {
    pub min: GridPosition,
    pub max: GridPosition,
}
impl Rect {
    pub fn new(min_x: i32, min_y: i32, max_x: i32, max_y: i32) -> Self {
        Rect {
            min: GridPosition{x: min_x, y: min_y, z: 0},
            max: GridPosition{x: max_x, y: max_y, z: 0}
        }
    }
}

// Stub for diffusing hazards (e.g. spores spreading out of their cavern)
pub fn diffuse_geome_hazards_system(mut grid: ResMut<TerrainGrid>) {
    // Logic to spread hazard tags to adjacent empty air tiles if breached
}
```

## REFACTOR Phase: Quality & Design

- **Map Generation Integration:** The `GeomeManager` should be deeply integrated into `src/layer1/terrain.rs` generation logic, likely using Perlin noise to create organic cavern shapes instead of simple `Rect` bounds.
- **Fluid Dynamics:** If Spec 412 (Fluid Simulation) is implemented, the "Sunless Sea" and "Magma River" should use actual fluid volumes rather than static terrain tiles, creating catastrophic flooding when breached.
- **Resource Drops:** Mining Geome-specific tiles (like `MagmaRock` or `CrystalTree`) should drop rare resources distinct from surface mining.
- **UI:** The game needs a way to visually distinguish these biomes (e.g., color tinting the ASCII/tiles based on Z-level and Geome type).

## Acceptance Criteria

- [ ] A `GeomeManager` resource or system can assign distinct `TerrainType` blocks in deep Z-levels during map generation.
- [ ] At least 3 distinct Geome types are defined (e.g. Magma, Water, Spore/Crystal).
- [ ] Geomes contain hazards that can damage Pops if exposed/breached (e.g. toxic air, extreme heat tiles).
- [ ] `cargo test` returns 0 failures for the new tests.
- [ ] Test coverage ≥85% for the new module.
- [ ] `cargo clippy -- -D warnings` passes.

## Technical Guidance

- **Performance:** Avoid sweeping the entire 3D grid every tick for hazard diffusion. Use event-driven diffusion (e.g. only update hazards when a tile changes from Solid to Air).
- **Z-Levels:** Ensure your pathfinding and mining actions properly account for the 3rd dimension. Breaking into a magma geome directly above a mining shaft should drop magma/heat down.
- **Safety Defaults:** Ensure the immediate sub-surface (Z -1 to -2) is relatively free of catastrophic geomes so players aren't instantly wiped out early game.

## Questions
*Builder: add questions here if spec is unclear. Architect will address.*
