# 082: Cabin Fever

## Overview

Simulates the psychological stress of confinement. Pops that spend too much time indoors or in crowded conditions accumulate a **Cabin Fever** penalty. This forces players to design settlements with outdoor spaces, parks, or wider corridors, and creates tension during long storms or sieges where pops are forced inside.

When Cabin Fever is high:
- **Morale** decreases.
- **Social Friction** increases (higher chance of negative social interactions).
- **Aggression** rises (increasing risk of violent mental breaks).

## Dependencies

- `005` — Pop Needs (Morale)
- `050` — Civil Unrest (Mental Breaks)
- `031` — Pop Morale (Calculation)
- `071` — Structural Integrity (RoofGrid for indoor detection)

## RED Phase: Tests First

Write these tests in `src/layer1/cabin_fever_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::map::GridPosition;
    use crate::layer1::needs::Needs;
    use crate::layer1::structural_integrity::RoofGrid;
    use crate::layer1::cabin_fever::{CabinFever, update_cabin_fever_system};

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(RoofGrid::new(10, 10));
        world
    }

    #[test]
    fn test_cabin_fever_increases_indoors() {
        let mut world = setup_world();

        // Roof the area (5, 5)
        world.resource_mut::<RoofGrid>().set_roof(5, 5, true);

        let pop = world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            CabinFever::default(),
        )).id();

        // Run system multiple times to simulate time passing
        for _ in 0..10 {
            world.run_system_once(update_cabin_fever_system).unwrap();
        }

        let fever = world.get::<CabinFever>(pop).unwrap();
        assert!(fever.current > 0.0, "Cabin Fever should increase when under a roof");
    }

    #[test]
    fn test_cabin_fever_decreases_outdoors() {
        let mut world = setup_world();

        // No roof at (5, 5)

        let pop = world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            CabinFever { current: 50.0, max: 100.0 },
        )).id();

        for _ in 0..10 {
            world.run_system_once(update_cabin_fever_system).unwrap();
        }

        let fever = world.get::<CabinFever>(pop).unwrap();
        assert!(fever.current < 50.0, "Cabin Fever should decrease when outdoors");
    }

    #[test]
    fn test_cabin_fever_impacts_morale() {
        // This requires integration with morale calculation logic (Spec 031).
        // Assuming we add a `get_cabin_fever_morale_penalty` function.
        let fever = CabinFever { current: 80.0, max: 100.0 };
        let penalty = fever.morale_penalty();
        assert!(penalty < 0.0, "High cabin fever should give negative morale penalty");
        assert!(penalty.abs() > 0.1, "Penalty should be significant at 80%");
    }

    #[test]
    fn test_crowding_accelerates_fever() {
        let mut world = setup_world();
        world.resource_mut::<RoofGrid>().set_roof(5, 5, true);

        // Spawn many pops at (5, 5)
        for _ in 0..5 {
            world.spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                CabinFever::default(),
            ));
        }

        let subject = world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            CabinFever::default(),
        )).id();

        world.run_system_once(update_cabin_fever_system).unwrap();

        let fever = world.get::<CabinFever>(subject).unwrap();
        // Base increase is X. With crowding it should be > X.
        // We assume base increase is known or we compare against a loner.

        // Loner comparison
        let loner = world.spawn((
            Pop,
            GridPosition { x: 0, y: 0 }, // Roofed but alone
            CabinFever::default(),
        )).id();
        world.resource_mut::<RoofGrid>().set_roof(0, 0, true);

        // Reset and run again for fair comparison
        world.entity_mut(subject).get_mut::<CabinFever>().unwrap().current = 0.0;

        world.run_system_once(update_cabin_fever_system).unwrap();

        let subject_fever = world.get::<CabinFever>(subject).unwrap().current;
        let loner_fever = world.get::<CabinFever>(loner).unwrap().current;

        assert!(subject_fever > loner_fever, "Crowded pop should gain fever faster");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Component

```rust
// src/layer1/cabin_fever.rs

use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::structural_integrity::RoofGrid;
use crate::layer1::pop::Pop;

#[derive(Component, Debug, Clone, Copy)]
pub struct CabinFever {
    pub current: f32,
    pub max: f32,
}

impl Default for CabinFever {
    fn default() -> Self {
        Self {
            current: 0.0,
            max: 100.0,
        }
    }
}

impl CabinFever {
    pub fn morale_penalty(&self) -> f32 {
        if self.current < 20.0 {
            0.0
        } else {
            // Linear scaling from 20 to 100
            // Max penalty -0.3
            -0.3 * ((self.current - 20.0) / 80.0)
        }
    }
}
```

### 2. Implement System

```rust
// src/layer1/cabin_fever.rs

pub fn update_cabin_fever_system(
    mut pops: Query<(Entity, &GridPosition, &mut CabinFever)>,
    others: Query<&GridPosition, With<Pop>>, // To check crowding
    roofs: Res<RoofGrid>,
) {
    // Optimization: Spatial map for crowding?
    // For MVP Green, O(N^2) might be slow if N is large.
    // Better: Build a transient grid count.

    let mut tile_counts = std::collections::HashMap::new();
    for pos in others.iter() {
        *tile_counts.entry((pos.x, pos.y)).or_insert(0) += 1;
    }

    for (entity, pos, mut fever) in &mut pops {
        let is_indoors = roofs.is_roofed(pos.x, pos.y);
        let count = tile_counts.get(&(pos.x, pos.y)).copied().unwrap_or(0);
        let is_crowded = count > 3; // More than 3 people on a tile

        if is_indoors {
            let mut increase = 0.1; // Base per tick
            if is_crowded {
                increase *= 2.0;
            }
            fever.current = (fever.current + increase).min(fever.max);
        } else {
            // Outdoors relief
            let decrease = 0.5;
            fever.current = (fever.current - decrease).max(0.0);
        }
    }
}
```

### 3. Integrate with Morale

Update `src/layer1/needs.rs` or wherever `calculate_effective_morale` is.

```rust
// In calculate_effective_morale signature or logic:
// Add CabinFever penalty if component exists.
```

## REFACTOR Phase: Quality & Design

- **Optimization**: `tile_counts` allocation every tick is costly for large maps. Use the existing `OccupiedTiles`? No, that tracks buildings. Use a dedicated `PopDensityMap` resource updated once per tick?
- **Scaling**: `is_roofed` assumes binary roof. Partial roofing?
- **Feedback**: Add a "Claustrophobic" status icon when Fever > 50%.
- **Traits**: "Agoraphobic" pops should *enjoy* being indoors (inverse logic). "Claustrophobic" pops gain fever 2x faster.

## Acceptance Criteria

- [ ] `CabinFever` component added to Pops.
- [ ] Fever increases when under a roof.
- [ ] Fever decreases when outdoors.
- [ ] Crowding (>3 pops/tile) accelerates fever gain.
- [ ] High Fever applies a negative Morale modifier.
- [ ] Tests pass.

## Technical Guidance

- Don't iterate `others` inside the `pops` loop (O(N^2)). Use the HashMap approach shown in Green Phase.
- Ensure `RoofGrid` is properly mocked or initialized in tests.
- Add `CabinFever` to the `spawn_initial_pops` bundle.
