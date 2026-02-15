# 119: Airlock & Pressure System

## Overview

Introduces atmospheric pressure simulation to Layer 1. The default environment is a vacuum (0.0 Pressure). Pops require pressure (approx 1.0) to survive; otherwise, they take suffocation damage.

This spec adds:
1.  **PressureGrid**: A resource tracking air pressure per tile (distinct from `AtmosphereGrid` which tracks pollution).
2.  **Life Support**: A building that generates pressure.
3.  **Airlock**: A building that allows passage without venting pressure (unlike standard Doors/Gates).
4.  **Suffocation**: A health mechanic for pops in low-pressure environments.

## Dependencies

- `002` — Terrain Grid
- `007` — Housing (Walls/Gates)
- `034` — Pop Health
- `063` — Atmospheric Simulation (Architecture pattern)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/pressure_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pressure::{PressureGrid, update_pressure_system, pressure_damage_system};
    use crate::layer1::map::GridPosition;
    use crate::layer1::health::Health;
    use crate::layer1::pop::Pop;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::structure::Structure;

    #[test]
    fn test_pressure_grid_defaults_to_vacuum() {
        let grid = PressureGrid::new(10, 10);
        assert_eq!(grid.get(5, 5), 0.0);
    }

    #[test]
    fn test_life_support_generates_pressure() {
        let mut world = World::new();
        let mut grid = PressureGrid::new(10, 10);
        world.insert_resource(grid);

        // Spawn Life Support
        world.spawn((
            Building { building_type: BuildingType::LifeSupport },
            GridPosition { x: 5, y: 5 },
        ));

        // Run update
        update_pressure_system(&mut world);

        let grid = world.resource::<PressureGrid>();
        assert!(grid.get(5, 5) > 0.0, "Life Support should generate pressure");
    }

    #[test]
    fn test_walls_block_diffusion() {
        let mut world = World::new();
        let mut grid = PressureGrid::new(3, 3);
        grid.set(1, 1, 1.0); // Pressure in center
        world.insert_resource(grid);

        // Spawn Wall at (1, 0) - North
        world.spawn((
            Building { building_type: BuildingType::Wall },
            GridPosition { x: 1, y: 0 },
            Structure::default(), // Valid blocking structure
        ));

        // Run update (diffusion)
        update_pressure_system(&mut world);

        let grid = world.resource::<PressureGrid>();
        // Center should retain more pressure because North is blocked
        // (compared to open diffusion)
        // More importantly, (1, 0) should remain near 0 if wall is perfect seal,
        // but walls occupy tiles. So check (1, -1) if grid was larger?
        // Let's check that diffusion didn't equalize perfectly to the wall tile
        // if the wall tile itself is considered "solid".
        // Better test: Wall at (2, 1). Pressure at (1, 1). (3, 1) should remain 0.

        // Re-setup
        let mut grid = PressureGrid::new(5, 1);
        grid.set(1, 0, 1.0); // Source
        world.insert_resource(grid);

        // Wall at (2, 0)
        world.spawn((
            Building { building_type: BuildingType::Wall },
            GridPosition { x: 2, y: 0 },
            Structure::default(),
        ));

        update_pressure_system(&mut world);

        let grid = world.resource::<PressureGrid>();
        assert!(grid.get(3, 0) < 0.01, "Pressure should not pass through wall");
    }

    #[test]
    fn test_gate_leaks_pressure() {
        // Gates are not airtight.
        let mut world = World::new();
        let mut grid = PressureGrid::new(5, 1);
        grid.set(1, 0, 1.0);
        world.insert_resource(grid);

        // Gate at (2, 0)
        world.spawn((
            Building { building_type: BuildingType::Gate },
            GridPosition { x: 2, y: 0 },
            Structure::default(),
        ));

        update_pressure_system(&mut world);

        let grid = world.resource::<PressureGrid>();
        assert!(grid.get(3, 0) > 0.05, "Pressure SHOULD leak through Gate");
    }

    #[test]
    fn test_airlock_blocks_pressure() {
        // Airlocks are airtight.
        let mut world = World::new();
        let mut grid = PressureGrid::new(5, 1);
        grid.set(1, 0, 1.0);
        world.insert_resource(grid);

        // Airlock at (2, 0)
        world.spawn((
            Building { building_type: BuildingType::Airlock },
            GridPosition { x: 2, y: 0 },
            Structure::default(),
        ));

        update_pressure_system(&mut world);

        let grid = world.resource::<PressureGrid>();
        assert!(grid.get(3, 0) < 0.01, "Pressure should NOT leak through Airlock");
    }

    #[test]
    fn test_suffocation_damage() {
        let mut world = World::new();
        let grid = PressureGrid::new(10, 10); // All 0.0
        world.insert_resource(grid);

        let pop = world.spawn((
            Pop::default(),
            Health { current: 100.0, max: 100.0 },
            GridPosition { x: 5, y: 5 },
        )).id();

        pressure_damage_system(&mut world);

        let health = world.get::<Health>(pop).unwrap();
        assert!(health.current < 100.0, "Pop in vacuum should take damage");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. PressureGrid (`src/layer1/pressure.rs`)

Clone the pattern from `AtmosphereGrid` (Spec 063) but add blocking logic.

```rust
#[derive(Resource)]
pub struct PressureGrid {
    pub width: usize,
    pub height: usize,
    pub values: Vec<f32>,
}

impl PressureGrid {
    // ... new, get, set, add ...

    // Custom diffuse that respects blockers
    pub fn diffuse(&mut self, blockers: &HashMap<(i32, i32), f32>) {
        // Transmissivity: 1.0 = Open, 0.0 = Wall, 0.5 = Leaky Door
        // For each tile, average with neighbors weighted by transmissivity
    }
}
```

### 2. Building Types (`src/layer1/building.rs`)

Add `LifeSupport` and `Airlock` to `BuildingType`.

```rust
pub enum BuildingType {
    // ...
    LifeSupport,
    Airlock,
}

// Update impls:
// LifeSupport cost: Metal + Energy usage?
// Airlock cost: Metal
// Char: 'L', 'A' (or 'D'?)
```

### 3. Update System

```rust
pub fn update_pressure_system(world: &mut World) {
    // 1. Identification of blockers
    let mut blockers = HashMap::new();
    for (b, pos) in world.query::<(&Building, &GridPosition)>().iter(world) {
        let transmissivity = match b.building_type {
            BuildingType::Wall => 0.0,
            BuildingType::Airlock => 0.0, // Sealed
            BuildingType::Gate => 0.5,    // Leaky
            _ => 1.0, // Most buildings don't block air (they are inside)
        };
        if transmissivity < 1.0 {
            blockers.insert((pos.x, pos.y), transmissivity);
        }
    }

    // 2. Generation
    let mut grid = world.resource_mut::<PressureGrid>();
    for (b, pos) in world.query::<(&Building, &GridPosition)>().iter(world) {
        if b.building_type == BuildingType::LifeSupport {
            // Life support tries to maintain 1.0 pressure
            // It doesn't just "add", it "fills" up to 1.0
            let current = grid.get(pos.x, pos.y);
            if current < 1.0 {
                grid.set(pos.x, pos.y, (current + 0.1).min(1.0));
            }
        }
    }

    // 3. Diffusion
    grid.diffuse(&blockers);

    // 4. Vacuum Sink
    // Edges of map are always 0.0? Or just naturally diffuse to 0 if not enclosed?
    // If map is initialized to 0.0, diffusion to 0-pressure neighbors acts as a sink.
}
```

### 4. Damage System

```rust
pub fn pressure_damage_system(world: &mut World) {
    let grid = world.resource::<PressureGrid>();
    // Query pops, check pressure, damage if < 0.2
    // Damage amount: 1.0 HP per tick (suffocation is fast)
}
```

## REFACTOR Phase: Quality & Design

- **Optimization**: Diffusion is expensive. Run every 5-10 ticks, or use a "Room" abstraction (Spec 064 Room Quality might help).
    - *Decision*: Stick to grid diffusion for MVP to handle "breaches" organically. Optimization: Only simulate active areas or use a "stable" flag.
- **Visuals**: Add a debug view mode to see pressure.
- **Spacesuits**: Future feature `040` (Clothing) could add `Spacesuit` item that negates vacuum damage.

## Acceptance Criteria

- [ ] `PressureGrid` defaults to 0.0.
- [ ] `LifeSupport` building raises local pressure.
- [ ] `Wall` and `Airlock` block pressure spread.
- [ ] `Gate` allows partial pressure leak.
- [ ] Pops in low pressure take damage.
- [ ] `cargo test` passes.

## Technical Guidance

- Use `AtmosphereGrid` as a template but remember: Pollution *adds* to infinity (technically), Pressure *equalizes* to 1.0.
- Ensure `Airlock` is walkable (non-obstacle) in `BuildingType::is_obstacle`, or has special handling in pathfinding (cost penalty).
- Integration: Add `update_pressure_system` to `SimulationSchedule`.
