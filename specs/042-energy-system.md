# 042: Energy System

## Overview

Introduces an **Energy Grid** where buildings can produce and consume power.
- **Energy**: An ephemeral resource (per tick). Not stored (unless batteries added later).
- **Grid**: A network of connected buildings sharing power.
- **PowerSource**: Component for generators (Produces +X Power).
- **PowerConsumer**: Component for machines (Consumes -Y Power).
- **Conduit**: Component for power lines (Connects grid).

If `Total Production < Total Demand`, the grid "browns out" or shuts down consumers based on priority.

## Dependencies

- `006` — Building Placement (for `Building` entities)
- `024` — Metal Industry (for `Metal` resource usage in construction)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/energy_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::energy::{PowerGrid, PowerSource, PowerConsumer, Conduit, calculate_grid_stats};
    use crate::layer1::map::GridPosition;
    use crate::layer1::building::{Building, BuildingType};

    #[test]
    fn test_power_components() {
        let source = PowerSource { output: 10.0 };
        let consumer = PowerConsumer { demand: 5.0, active: true };
        let conduit = Conduit; // Marker

        assert_eq!(source.output, 10.0);
        assert_eq!(consumer.demand, 5.0);
    }

    #[test]
    fn test_grid_connectivity_isolated() {
        // Source and Consumer far apart, no conduit
        let mut world = World::new();

        // Generator at 0,0
        let gen = world.spawn((
            PowerSource { output: 10.0 },
            GridPosition { x: 0, y: 0 },
            Building { building_type: BuildingType::Generator }, // Assume new type
        )).id();

        // Consumer at 10,10
        let cons = world.spawn((
            PowerConsumer { demand: 5.0, active: false },
            GridPosition { x: 10, y: 10 },
            Building { building_type: BuildingType::Smelter },
        )).id();

        // Run calculation
        let (production, demand) = calculate_grid_stats(&world, gen); // Pass root entity to flood fill

        // Gen is its own grid
        assert_eq!(production, 10.0);

        // Consumer is isolated
        let (c_prod, c_demand) = calculate_grid_stats(&world, cons);
        assert_eq!(c_prod, 0.0);
        assert_eq!(c_demand, 5.0);
    }

    #[test]
    fn test_grid_connectivity_connected() {
        let mut world = World::new();

        // Generator at 0,0
        let gen = world.spawn((
            PowerSource { output: 10.0 },
            GridPosition { x: 0, y: 0 },
            Building { building_type: BuildingType::Generator },
        )).id();

        // Conduit at 0,1
        world.spawn((
            Conduit,
            GridPosition { x: 0, y: 1 },
            Building { building_type: BuildingType::PowerPole }, // New type
        ));

        // Consumer at 0,2
        let cons = world.spawn((
            PowerConsumer { demand: 5.0, active: false },
            GridPosition { x: 0, y: 2 },
            Building { building_type: BuildingType::Smelter },
        )).id();

        // Run calculation (start from generator)
        let (production, demand) = calculate_grid_stats(&world, gen);

        assert_eq!(production, 10.0);
        assert_eq!(demand, 5.0);
    }

    #[test]
    fn test_overload_shutdown() {
        // 10 Prod, 15 Demand
        let mut world = World::new();

        // Gen 10
        let gen = world.spawn((
            PowerSource { output: 10.0 },
            GridPosition { x: 0, y: 0 },
        )).id();

        // Cons 1 (10)
        world.spawn((
            PowerConsumer { demand: 10.0, active: true },
            GridPosition { x: 0, y: 1 },
        ));

        // Cons 2 (5)
        world.spawn((
            PowerConsumer { demand: 5.0, active: true },
            GridPosition { x: 0, y: 2 },
        ));

        // Assume implicit connection for this test simplicity (or setup conduits)
        // Implementation might calculate per-network.

        // Run system
        // crate::layer1::energy::power_grid_system(&mut world);

        // Verify active state toggles?
        // Or "efficiency" drops?
        // For MVP: If Demand > Production, ALL consumers on that grid shut down (active = false).
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Components (`src/layer1/energy.rs`)

```rust
use bevy_ecs::prelude::*;

#[derive(Component, Debug, Clone)]
pub struct PowerSource {
    pub output: f32,
}

#[derive(Component, Debug, Clone)]
pub struct PowerConsumer {
    pub demand: f32,
    pub active: bool, // Set by system if powered
}

#[derive(Component, Debug, Clone)]
pub struct Conduit; // Marker for power line

// Building types need update in building.rs:
// Generator, PowerPole
```

### 2. Grid Logic (`src/layer1/energy.rs`)

```rust
use crate::layer1::GridPosition;
use crate::layer1::map::GridPosition;
use std::collections::{HashSet, VecDeque};

pub fn calculate_grid_stats(world: &World, start_entity: Entity) -> (f32, f32) {
    // BFS implementation
    // Find all connected entities (Source, Consumer, Conduit)
    // Sum Output and Demand
    // Return (Prod, Demand)
    (0.0, 0.0) // Placeholder
}

pub fn power_grid_system(world: &mut World) {
    // 1. Identify all grids (groups of connected power entities)
    // 2. For each grid:
    //    Sum Production, Sum Demand.
    //    If Prod >= Demand: Set all Consumers active = true.
    //    Else: Set all active = false (Brownout).
    // Note: optimization needed for large maps (Disjoint Set Union or dirty flags).
}
```

### 3. Update `BuildingType`

```rust
// src/layer1/building.rs
pub enum BuildingType {
    // ...
    Generator, // Costs Metal + Stone
    PowerPole, // Costs Metal
}
```

## REFACTOR Phase: Quality & Design

- **Optimization**: Re-scanning the whole grid every tick is slow. Use a `GridGraph` resource that updates only when buildings are placed/removed.
- **Partial Power**: Instead of full shutdown, maybe reduce efficiency? Or random shutdown?
- **Fuel**: `Generator` currently produces free power. It should consume `Wood`/`Coal`. Add `FuelConsumer` component?
- **Visuals**: Draw lines between PowerPoles?

## Acceptance Criteria

- [ ] `PowerSource`, `PowerConsumer`, `Conduit` components exist.
- [ ] `Generator` and `PowerPole` buildings are placeable.
- [ ] Power flows through adjacent Conduits/Buildings.
- [ ] Consumers activate only if connected to sufficient power.
- [ ] Overload causes shutdown (brownout).
- [ ] Tests pass.

## Questions

- Should walls conduct power? (No, requires explicit conduits for now).
- Is power global or local? (Local grids).
  - *Architect:* See related specifications for design details. MVP implementation should follow standard conventions.
