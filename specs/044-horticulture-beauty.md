# 044: Horticulture and Beauty

## Overview

Introduces an environmental **Beauty** system. Tiles have a beauty value that affects Pop morale.
- **Beauty Grid**: A map layer tracking beauty values (e.g., -10 to +10).
- **Decorations**: Buildings like `FlowerBed` and `Statue` that emit positive beauty.
- **Filth**: Debris/Rot emits negative beauty (future spec, but supported by grid).
- **Morale Impact**: Pops gain `Leisure` need satisfaction or direct Morale bonus when in high beauty areas.

## Dependencies

- `031` — Pop Morale (for `Needs` and `Morale` calculation)
- `006` — Building Placement (for `Decoration` buildings)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/beauty_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::map::GridPosition;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::Pop;

    // 1. Beauty Grid
    #[test]
    fn test_beauty_grid_resource_defaults() {
        // Need a new resource for BeautyGrid
        let grid = crate::layer1::beauty::BeautyGrid::new(10, 10);
        assert_eq!(grid.get(0, 0), 0.0);
    }

    #[test]
    fn test_beauty_emission_from_buildings() {
        let mut world = World::new();
        // Setup grid
        world.insert_resource(crate::layer1::beauty::BeautyGrid::new(10, 10));

        // Spawn FlowerBed (Beauty +5)
        world.spawn((
            Building { building_type: BuildingType::FlowerBed },
            GridPosition { x: 5, y: 5 },
        ));

        // Run system to update grid
        crate::layer1::beauty::update_beauty_grid_system(&mut world);

        let grid = world.resource::<crate::layer1::beauty::BeautyGrid>();
        assert_eq!(grid.get(5, 5), 5.0);

        // Check falloff? (Optional for MVP, maybe just local tile)
        // Let's assume simple 1-tile radius for MVP
        assert_eq!(grid.get(4, 5), 0.0);
    }

    #[test]
    fn test_negative_beauty_from_trash() {
        let mut world = World::new();
        world.insert_resource(crate::layer1::beauty::BeautyGrid::new(10, 10));

        // Spawn Trash (Beauty -5) - Future proofing
        // For now, let's say a specific "Debris" building or similar
        // Or just test that grid accepts negative values
        let mut grid = world.resource_mut::<crate::layer1::beauty::BeautyGrid>();
        grid.set(0, 0, -5.0);

        assert_eq!(grid.get(0, 0), -5.0);
    }

    // 2. Morale Impact
    #[test]
    fn test_beauty_satisfies_leisure() {
        let mut world = World::new();
        let mut grid = crate::layer1::beauty::BeautyGrid::new(10, 10);
        grid.set(0, 0, 10.0); // High beauty
        world.insert_resource(grid);

        let pop = world.spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            Needs { leisure: 0.1, ..Default::default() }, // Low leisure
        )).id();

        // Run system
        crate::layer1::beauty::apply_beauty_effects_system(&mut world);

        let needs = world.get::<Needs>(pop).unwrap();
        assert!(needs.leisure > 0.1, "High beauty should increase leisure");
    }

    #[test]
    fn test_negative_beauty_decreases_leisure() {
        let mut world = World::new();
        let mut grid = crate::layer1::beauty::BeautyGrid::new(10, 10);
        grid.set(0, 0, -10.0); // Disgusting
        world.insert_resource(grid);

        let pop = world.spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            Needs { leisure: 0.5, ..Default::default() },
        )).id();

        crate::layer1::beauty::apply_beauty_effects_system(&mut world);

        let needs = world.get::<Needs>(pop).unwrap();
        assert!(needs.leisure < 0.5, "Negative beauty should decrease leisure");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. `BeautyGrid` Resource (`src/layer1/beauty.rs`)

```rust
use bevy_ecs::prelude::*;

#[derive(Resource, Default)]
pub struct BeautyGrid {
    pub width: usize,
    pub height: usize,
    pub values: Vec<f32>,
}

impl BeautyGrid {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            values: vec![0.0; width * height],
        }
    }

    pub fn get(&self, x: usize, y: usize) -> f32 {
        if x >= self.width || y >= self.height { return 0.0; }
        self.values[y * self.width + x]
    }

    pub fn set(&mut self, x: usize, y: usize, value: f32) {
        if x >= self.width || y >= self.height { return; }
        self.values[y * self.width + x] = value;
    }

    pub fn clear(&mut self) {
        self.values.fill(0.0);
    }
}
```

### 2. Update `BuildingType`

Add `FlowerBed` and `Statue`.

```rust
// src/layer1/building.rs
pub enum BuildingType {
    // ...
    FlowerBed, // Beauty +5
    Statue,    // Beauty +10
}

impl BuildingType {
    pub fn beauty_value(&self) -> f32 {
        match self {
            Self::FlowerBed => 5.0,
            Self::Statue => 10.0,
            _ => 0.0,
        }
    }
}
```

### 3. Update System

```rust
// src/layer1/beauty.rs

use crate::layer1::building::Building;
use crate::layer1::map::GridPosition;
use crate::layer1::pop::Pop;
use crate::layer1::needs::Needs;

pub fn update_beauty_grid_system(
    mut grid: ResMut<BeautyGrid>,
    buildings: Query<(&GridPosition, &Building)>,
) {
    grid.clear();

    for (pos, building) in &buildings {
        let value = building.building_type.beauty_value();
        if value != 0.0 {
            // Apply to grid (simple 1-tile for now)
            if let (Ok(x), Ok(y)) = (usize::try_from(pos.x), usize::try_from(pos.y)) {
                let current = grid.get(x, y);
                grid.set(x, y, current + value);
            }
        }
    }
}

pub fn apply_beauty_effects_system(
    grid: Res<BeautyGrid>,
    mut pops: Query<(&GridPosition, &mut Needs), With<Pop>>,
) {
    for (pos, mut needs) in &mut pops {
        if let (Ok(x), Ok(y)) = (usize::try_from(pos.x), usize::try_from(pos.y)) {
            let beauty = grid.get(x, y);
            if beauty > 0.0 {
                // Boost leisure
                needs.leisure = (needs.leisure + beauty * 0.001).min(1.0);
            } else if beauty < 0.0 {
                // Decay leisure
                needs.leisure = (needs.leisure + beauty * 0.001).max(0.0);
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Performance**: Only update beauty grid when buildings change (use `Changed<Building>` query or `Event`).
- **Diffusion**: Beauty should spread to adjacent tiles (blur pass).
- **Visualization**: Render beauty overlay (debug mode or specialized view).
- **Traits**: Pops with "Ascetic" trait might ignore beauty.

## Acceptance Criteria

- [ ] `BeautyGrid` resource exists.
- [ ] `FlowerBed` and `Statue` buildings exist.
- [ ] `update_beauty_grid_system` populates the grid.
- [ ] `apply_beauty_effects_system` modifies Pop `Leisure`.
- [ ] Tests pass.

## Questions

- Should nature (Trees) have beauty? (Yes, `019 Forestry` trees should emit beauty in `update_beauty_grid_system`).
- Should `Stockpile` (Rotting food) emit negative beauty? (Yes, in `032 Spoilage` integration).
