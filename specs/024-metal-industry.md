# 024: Metal Industry

## Overview

Expands the resource economy by introducing **Ore** and **Metal**.
- **Ore**: A raw material found rarely when mining rock.
- **Metal**: A refined material produced by smelting Ore.
- **Smelter**: A new building that converts Ore + Wood into Metal.

This lays the foundation for the "Tool Economy" and advanced construction.

## Dependencies

- `018` — Mining (for `mine_rock`)
- `023` — Refining Industry (for `RefiningProgress` and `process_refining_system`)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/metal_industry_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::resources::{ColonyResources, mine_rock, MiningProgress};
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::refining::{RefiningProgress, process_refining_system};
    use crate::layer1::pop::Pop;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use crate::layer1::{GridPosition, Designation, DesignationType};

    #[test]
    fn test_resources_metal_fields() {
        let res = ColonyResources::default();
        assert_eq!(res.ore, 0.0);
        assert_eq!(res.metal, 0.0);
        assert_eq!(res.max_ore, 20.0);
        assert_eq!(res.max_metal, 20.0);
    }

    #[test]
    fn test_building_type_smelter() {
        let b = BuildingType::Smelter;
        assert_eq!(b.label(), "Smelter");
        assert_eq!(b.char(), 'S');
    }

    #[test]
    fn test_mine_rock_yields_ore_probabilistically() {
        let mut world = World::new();
        // Setup massive grid of rocks
        let tiles = vec![TerrainType::Rock; 1000];
        world.insert_resource(TerrainGrid { width: 100, height: 10, tiles });
        world.insert_resource(ColonyResources::default());

        let mut ore_count = 0.0;

        // Mine 100 rocks
        for i in 0..100 {
            let entity = world.spawn((
                Designation { designation_type: DesignationType::Mine },
                MiningProgress { current: 9.9, max: 10.0 }, // Almost done
                GridPosition { x: i % 100, y: 0 },
            )).id();

            mine_rock(&mut world, entity, 0.2); // Complete it

            let res = world.resource::<ColonyResources>();
            if res.ore > ore_count {
                ore_count = res.ore;
            }
        }

        // Should have found SOME ore (20% chance * 100 trials = ~20)
        assert!(ore_count > 0.0, "Mining 100 rocks should yield some ore");
        assert!(ore_count < 100.0, "Every rock shouldn't yield ore");
    }

    #[test]
    fn test_smelter_refines_ore_to_metal() {
        let mut world = World::new();
        let mut res = ColonyResources::default();
        res.ore = 10.0;
        res.wood = 10.0;
        res.metal = 0.0;
        world.insert_resource(res);

        // Spawn Smelter
        world.spawn((
            Building { building_type: BuildingType::Smelter },
            GridPosition { x: 5, y: 5 },
            RefiningProgress { current: 9.9, max: 10.0 },
        ));

        // Spawn Worker
        world.spawn((Pop, GridPosition { x: 5, y: 6 }));

        // Run system
        process_refining_system(&mut world);

        let res = world.resource::<ColonyResources>();
        // Cost: 1 Ore + 1 Wood
        assert!((res.ore - 9.0).abs() < f32::EPSILON);
        assert!((res.wood - 9.0).abs() < f32::EPSILON);
        // Gain: 1 Metal
        assert!((res.metal - 1.0).abs() < f32::EPSILON);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update ColonyResources

```rust
// src/layer1/resources.rs

pub struct ColonyResources {
    // ... existing
    pub ore: f32,
    pub metal: f32,
    pub max_ore: f32,
    pub max_metal: f32,
}

impl Default for ColonyResources {
    fn default() -> Self {
        Self {
            // ... existing
            ore: 0.0,
            metal: 0.0,
            max_ore: 20.0,
            max_metal: 20.0,
            // ...
        }
    }
}
```

### 2. Update Mining Logic

```rust
// src/layer1/resources.rs
use rand::Rng; // Add rand to imports

pub fn mine_rock(world: &mut World, designation_entity: Entity, work_amount: f32) {
    // ... checks ...
    if completed {
        // ... terrain update ...

        // Add resources
        let mut resources = world.resource_mut::<ColonyResources>();
        resources.add_stone(1.0);

        // Chance for Ore (20%)
        let mut rng = rand::thread_rng();
        if rng.gen_bool(0.2) {
            resources.add_ore(1.0); // Implement add_ore()
        }

        // ... removal ...
    }
}
```

### 3. Update BuildingType

```rust
// src/layer1/building.rs
pub enum BuildingType {
    // ...
    Smelter,
}

impl BuildingType {
    pub fn char(&self) -> char {
        match self {
            Self::Smelter => 'S',
            // ...
        }
    }
    // Update cost: Smelter = 50 Stone, 20 Wood
}
```

### 4. Update Refining System (from Spec 023)

```rust
// src/layer1/refining.rs

pub fn process_refining_system(world: &mut World) {
    // ... inside building loop ...

    let (can_refine, input_cost, output_gain) = match building.building_type {
        BuildingType::LumberMill => { /* ... */ },
        BuildingType::StoneMason => { /* ... */ },
        BuildingType::Smelter => {
            (
                resources.ore >= 1.0 && resources.wood >= 1.0 && resources.metal < resources.max_metal,
                ColonyResources { ore: 1.0, wood: 1.0, ..Default::default() },
                ColonyResources { metal: 1.0, ..Default::default() }
            )
        },
        _ => (false, ColonyResources::default(), ColonyResources::default()),
    };

    // ... apply logic ...
}
```

## REFACTOR Phase: Quality & Design

- **RNG Injection**: For better testing, pass an RNG trait or seed to `mine_rock` instead of using `thread_rng` directly.
- **Recipe Data**: The `match` block in `process_refining_system` is growing. Move recipes to a `RefiningRecipe` struct or look-up table.
- **Fuel Mechanics**: Burning wood for smelting is simple now. Future: `Coal` resource or `Fuel` trait?

## Acceptance Criteria

- [ ] `ColonyResources` has `ore` and `metal`.
- [ ] Mining rock yields ore ~20% of the time.
- [ ] `Smelter` building exists and can be placed.
- [ ] `Smelter` consumes 1 Ore + 1 Wood to produce 1 Metal.
- [ ] Tests pass.

## Technical Guidance

- Ensure `add_ore` and `add_metal` clamp to `max_ore` / `max_metal`.
- The `RefiningProgress` component from Spec 023 is reused; ensure it is imported correctly.
- Add `rand` crate to `Cargo.toml` if not present (it should be, used by map gen).
