# 106: Resource Purity

## Overview

Not all ore is created equal. Currently, every rock yields `Stone` and has a flat chance to yield `Ore`. This spec introduces **Resource Purity**, a property of the terrain that determines the quality of the yield.

High purity rocks yield `Ore` reliably. Low purity rocks yield `Ore` mixed with `Waste` (Slag/Impurity), forcing the player to manage waste disposal logistics even at the extraction phase.

This adds depth to mining: players will scout for high-purity veins to maximize efficiency and minimize waste handling.

## Dependencies

- `018` — Mining Resources (for `mine_rock`, `MiningProgress`)
- `032` — Waste and Pollution (for `ResourceType::Waste`)
- `036` — Exploration (for visibility of purity, optional)

## RED Phase: Tests First

Write these tests in `src/layer1/purity_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::resources::{mine_rock, ColonyResources, MiningProgress, ResourceItem, ResourceType};
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use crate::layer1::GridPosition;
    use crate::layer1::purity::PurityMap;

    #[test]
    fn test_purity_map_resource_exists() {
        let map = PurityMap::default();
        // Default purity should be reasonable (e.g., 1.0 or noise-based)
        // For testing, we might want a way to set it or rely on a deterministic seed.
    }

    #[test]
    fn test_mine_rock_high_purity_yields_ore() {
        let mut world = World::new();
        // Setup world with 1 Rock
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[0] = TerrainType::Rock;
        world.insert_resource(TerrainGrid { width: 10, height: 10, tiles });
        world.insert_resource(ColonyResources::default());
        world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(10, 10)); // Dependency

        // Setup PurityMap with High Purity (1.0) at (0,0)
        let mut purity_map = PurityMap::default();
        purity_map.set_override(0, 0, 1.0); // Helper for testing
        world.insert_resource(purity_map);

        // Spawn Designation
        let designation = world.spawn((
            GridPosition { x: 0, y: 0 },
            MiningProgress { current: 9.0, max: 10.0 },
        )).id();

        // Mine
        mine_rock(&mut world, designation, 1.0);

        // Assert: Should contain Ore
        let items: Vec<_> = world.query::<&ResourceItem>().iter(&world).collect();
        let has_ore = items.iter().any(|i| i.resource_type == ResourceType::Ore);
        assert!(has_ore, "High purity should yield Ore");

        // Assert: Should NOT contain Waste (at 1.0 purity)
        let has_waste = items.iter().any(|i| i.resource_type == ResourceType::Waste);
        assert!(!has_waste, "High purity should not yield Waste");
    }

    #[test]
    fn test_mine_rock_low_purity_yields_waste() {
        let mut world = World::new();
        // Setup Rock
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[0] = TerrainType::Rock;
        world.insert_resource(TerrainGrid { width: 10, height: 10, tiles });
        world.insert_resource(ColonyResources::default());
        world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(10, 10));

        // Setup PurityMap with Low Purity (0.0)
        let mut purity_map = PurityMap::default();
        purity_map.set_override(0, 0, 0.0);
        world.insert_resource(purity_map);

        let designation = world.spawn((
            GridPosition { x: 0, y: 0 },
            MiningProgress { current: 9.0, max: 10.0 },
        )).id();

        mine_rock(&mut world, designation, 1.0);

        // Assert: Should contain Waste
        let items: Vec<_> = world.query::<&ResourceItem>().iter(&world).collect();
        let has_waste = items.iter().any(|i| i.resource_type == ResourceType::Waste);
        assert!(has_waste, "Low purity should yield Waste");

        // Assert: Should NOT contain Ore (at 0.0 purity)
        let has_ore = items.iter().any(|i| i.resource_type == ResourceType::Ore);
        assert!(!has_ore, "Low purity should not yield Ore");
    }

    #[test]
    fn test_mine_rock_mixed_purity_probabilistic() {
        // This is a probabilistic test, might be flaky if not seeded.
        // Better to test the *logic function* deterministically if possible.
        // e.g. `calculate_yield(purity, rng_seed)`
    }

    #[test]
    fn test_get_purity_consistent() {
        let map = PurityMap::new(12345); // Seed
        let p1 = map.get(10, 10);
        let p2 = map.get(10, 10);
        assert_eq!(p1, p2, "Purity should be deterministic for same seed/coord");
    }
}
```

## GREEN Phase: Minimal Implementation

Write the SIMPLEST code to make tests pass.

### 1. Define `PurityMap` Resource

```rust
// src/layer1/purity.rs

use bevy_ecs::prelude::*;
use std::collections::HashMap;

#[derive(Resource)]
pub struct PurityMap {
    seed: u32,
    overrides: HashMap<(i32, i32), f32>, // For testing
}

impl Default for PurityMap {
    fn default() -> Self {
        Self { seed: 0, overrides: HashMap::new() }
    }
}

impl PurityMap {
    pub fn new(seed: u32) -> Self {
        Self { seed, overrides: HashMap::new() }
    }

    pub fn set_override(&mut self, x: i32, y: i32, value: f32) {
        self.overrides.insert((x, y), value);
    }

    pub fn get(&self, x: i32, y: i32) -> f32 {
        if let Some(&val) = self.overrides.get(&(x, y)) {
            return val;
        }

        // Simple pseudo-random noise for MVP
        // In real impl, use Perlin/Simplex noise
        let mut h = self.seed as u64;
        h = h.wrapping_add((x as u64) << 32);
        h = h.wrapping_add(y as u64);
        // ... hash function ...
        // Return 0.0 to 1.0
        0.5 // Placeholder
    }
}
```

### 2. Update `mine_rock` in `src/layer1/resources.rs`

```rust
// In mine_rock function:

// ... check is_rock ...
// ... update progress ...

if completed {
    // ... change terrain to Dirt ...
    // ... spawn Stone ...

    // NEW LOGIC: Check Purity
    let purity = if let Some(map) = world.get_resource::<PurityMap>() {
        map.get(pos.x, pos.y)
    } else {
        0.2 // Fallback to old hardcoded chance if resource missing
    };

    let mut rng = rand::thread_rng();

    // Ore Check: Probability = Purity
    if rng.gen_bool(purity as f64) {
        world.spawn((
            ResourceItem { resource_type: ResourceType::Ore, amount: 1.0 },
            pos,
        ));
        // Log "Mined High-Grade Ore"
    }

    // Waste Check: Probability = 1.0 - Purity
    // Or maybe: if we didn't get Ore, we get Waste?
    // Or independent?
    // Let's say:
    // - Purity 1.0: 100% Ore, 0% Waste
    // - Purity 0.0: 0% Ore, 100% Waste
    // - Purity 0.5: 50% Ore, 50% Waste (could get both, or neither?)

    // Deterministic mutual exclusion logic:
    // val = rng.gen()
    // if val < purity { spawn Ore }
    // else { spawn Waste }

    // BUT: We always want *something*?
    // Spec says "Low purity rocks yield Ore mixed with Waste".
    // Maybe independent checks are better for "mixed".

    if rng.gen_bool((1.0 - purity) as f64) {
        world.spawn((
            ResourceItem { resource_type: ResourceType::Waste, amount: 1.0 },
            pos,
        ));
        // Log "Mined Slag"
    }
}
```

## REFACTOR Phase: Quality & Design

### Noise Function
Replace the placeholder hash with a proper noise function (e.g., `noise` crate or a simple LCID) to create "veins" where high purity clusters together.

### UI Integration
Update the **Inspector** or **Selection Details** UI to show the Purity % when hovering over a Rock tile.
`Target: Rock (Purity: 85%)`

### Waste Logistics
Ensure `Waste` resource items can be hauled to a `Landfill` or `Stockpile`. (Already covered by Spec 032).

## Acceptance Criteria

- [ ] All RED tests pass.
- [ ] `PurityMap` resource exists.
- [ ] `mine_rock` uses `PurityMap` to determine yield.
- [ ] High purity locations reliably yield Ore.
- [ ] Low purity locations reliably yield Waste.
- [ ] Existing "20% fixed chance" logic is replaced by this system.

## Technical Guidance

- Use `PurityMap::set_override` strictly for tests.
- Ensure `PurityMap` is inserted in `src/setup.rs`.
- Ensure `ResourceType::Waste` is handled by hauling logic (it should be if `ResourceItem` is generic).

## Questions

*Builder: Should Purity also affect the amount of Stone?*
*Architect:* Yes, lower Purity veins should yield proportionally more Stone and less target ore per mining action to accurately reflect the extraction inefficiency.
*Architect: No, Resource Purity affects the Ore/Waste ratio, not overall Stone amount.*
