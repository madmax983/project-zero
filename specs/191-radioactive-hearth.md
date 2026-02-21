# 191: Radioactive Hearth

## Overview

Items with radioactive properties (`Waste`, `Ore`) now emit heat and radiation. In extreme cold biomes, players can exploit this by stockpiling these materials in residential walls to heat rooms without power, but at the cost of `RadiationSickness` to nearby Pops.

This adds a "Devil's Bargain" mechanic to thermal management and waste disposal.

## Dependencies

- `140` — Thermal Management
- `049` — Industrial Waste
- `004` — Health System (for Sickness)

## RED Phase: Tests First

```rust
// src/layer1/radioactive_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::temperature::{TemperatureGrid, update_temperature_system};
    use crate::layer1::map::GridPosition;
    use crate::layer1::resources::{ResourceItem, ResourceType};
    use crate::layer1::health::Health;
    use crate::layer1::pop::Pop;
    use crate::layer1::radioactive::{RadiationGrid, radiation_system, RadiationSickness};

    #[test]
    fn test_waste_emits_heat() {
        let mut world = World::new();
        // Setup TemperatureGrid
        world.insert_resource(TemperatureGrid::new(10, 10, 0.0));
        world.insert_resource(crate::layer1::seasons::SeasonState::default());

        // Spawn Waste Item
        world.spawn((
            ResourceItem { resource_type: ResourceType::Waste, amount: 1.0 },
            GridPosition { x: 5, y: 5 },
        ));

        // Run temperature update
        update_temperature_system(&mut world);

        let grid = world.resource::<TemperatureGrid>();
        assert!(grid.get(5, 5) > 0.0, "Waste should emit heat");
    }

    #[test]
    fn test_ore_emits_heat() {
        let mut world = World::new();
        world.insert_resource(TemperatureGrid::new(10, 10, 0.0));
        world.insert_resource(crate::layer1::seasons::SeasonState::default());

        world.spawn((
            ResourceItem { resource_type: ResourceType::Ore, amount: 1.0 },
            GridPosition { x: 5, y: 5 },
        ));

        update_temperature_system(&mut world);

        let grid = world.resource::<TemperatureGrid>();
        assert!(grid.get(5, 5) > 0.0, "Ore should emit heat");
    }

    #[test]
    fn test_food_does_not_emit_heat() {
        let mut world = World::new();
        world.insert_resource(TemperatureGrid::new(10, 10, 0.0));
        world.insert_resource(crate::layer1::seasons::SeasonState::default());

        world.spawn((
            ResourceItem { resource_type: ResourceType::Food, amount: 1.0 },
            GridPosition { x: 5, y: 5 },
        ));

        update_temperature_system(&mut world);

        let grid = world.resource::<TemperatureGrid>();
        assert_eq!(grid.get(5, 5), 0.0, "Food should not emit heat");
    }

    #[test]
    fn test_radiation_grid_accumulates() {
        let mut world = World::new();
        world.insert_resource(RadiationGrid::new(10, 10));

        // Spawn Waste
        world.spawn((
            ResourceItem { resource_type: ResourceType::Waste, amount: 1.0 },
            GridPosition { x: 5, y: 5 },
        ));

        // Run radiation system
        radiation_system(&mut world);

        let grid = world.resource::<RadiationGrid>();
        assert!(grid.get(5, 5) > 0.0);
        assert!(grid.get(6, 5) > 0.0); // Spread
    }

    #[test]
    fn test_radiation_sickness_application() {
        let mut world = World::new();
        let mut grid = RadiationGrid::new(10, 10);
        grid.set(5, 5, 100.0); // High radiation
        world.insert_resource(grid);

        let pop = world.spawn((
            Pop,
            Health::default(),
            GridPosition { x: 5, y: 5 },
            // Sickness component added by system? Or exists with 0 severity?
        )).id();

        radiation_system(&mut world);

        // Pop should now have RadiationSickness component
        let sickness = world.get::<RadiationSickness>(pop);
        assert!(sickness.is_some());
        assert!(sickness.unwrap().severity > 0.0);
    }

    #[test]
    fn test_sickness_damages_health() {
        let mut world = World::new();
        let pop = world.spawn((
            Pop,
            Health { current: 100.0, max: 100.0 },
            RadiationSickness { severity: 60.0 }, // Threshold is usually 50
        )).id();

        // Run health/damage system logic for sickness
        crate::layer1::radioactive::sickness_damage_system(&mut world);

        let health = world.get::<Health>(pop).unwrap();
        assert!(health.current < 100.0);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. `RadiationGrid` Resource

Create `src/layer1/radioactive.rs`.

```rust
// src/layer1/radioactive.rs

use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::health::Health;
use crate::layer1::pop::Pop;
use crate::layer1::resources::{ResourceItem, ResourceType};

#[derive(Resource)]
pub struct RadiationGrid {
    pub width: usize,
    pub height: usize,
    pub values: Vec<f32>,
}

impl RadiationGrid {
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

    pub fn set(&mut self, x: usize, y: usize, val: f32) {
        if x < self.width && y < self.height {
            self.values[y * self.width + x] = val;
        }
    }

    pub fn clear(&mut self) {
        self.values.fill(0.0);
    }

    pub fn add_source(&mut self, x: i32, y: i32, intensity: f32, radius: f32) {
        let r_int = radius.ceil() as i32;
        for dy in -r_int..=r_int {
            for dx in -r_int..=r_int {
                let dist = ((dx*dx + dy*dy) as f32).sqrt();
                if dist <= radius {
                    let falloff = 1.0 - (dist / (radius + 0.1));
                    if falloff > 0.0 {
                        let nx = x + dx;
                        let ny = y + dy;
                        if nx >= 0 && ny >= 0 && nx < self.width as i32 && ny < self.height as i32 {
                             let idx = (ny as usize) * self.width + (nx as usize);
                             self.values[idx] += intensity * falloff;
                        }
                    }
                }
            }
        }
    }
}

#[derive(Component, Default, Debug, Clone, Copy)]
pub struct RadiationSickness {
    pub severity: f32,
}

pub fn radiation_system(
    mut grid: Option<ResMut<RadiationGrid>>,
    items: Query<(&ResourceItem, &GridPosition)>,
    mut pops: Query<(Entity, &GridPosition, Option<&mut RadiationSickness>), With<Pop>>,
    mut commands: Commands,
) {
    let Some(mut grid) = grid else { return };

    // 1. Reset Grid
    grid.clear();

    // 2. Add Sources
    for (item, pos) in &items {
        let (rads, radius) = match item.resource_type {
            ResourceType::Waste => (5.0, 3.0),
            ResourceType::Ore => (1.0, 1.0),
            _ => (0.0, 0.0),
        };
        if rads > 0.0 {
            grid.add_source(pos.x, pos.y, rads, radius);
        }
    }

    // 3. Apply to Pops
    for (entity, pos, sickness_opt) in &mut pops {
        let exposure = grid.get(pos.x as usize, pos.y as usize);
        if exposure > 0.0 {
            if let Some(mut sick) = sickness_opt {
                sick.severity += exposure * 0.1;
                sick.severity = sick.severity.min(100.0);
            } else {
                commands.entity(entity).insert(RadiationSickness { severity: exposure * 0.1 });
            }
        } else if let Some(mut sick) = sickness_opt {
            // Recovery
            sick.severity = (sick.severity - 0.1).max(0.0);
            if sick.severity <= 0.0 {
                commands.entity(entity).remove::<RadiationSickness>();
            }
        }
    }
}

pub fn sickness_damage_system(mut query: Query<(&mut Health, &RadiationSickness)>) {
    for (mut health, sick) in &mut query {
        if sick.severity > 50.0 {
            health.take_damage(0.1); // Slow death
        }
        if sick.severity > 90.0 {
            health.take_damage(0.5); // Fast death
        }
    }
}
```

### 2. Update `TemperatureGrid` Logic

Modify `src/layer1/temperature.rs` to include `ResourceItem` query.

```rust
// In update_temperature_system signature:
items: Query<(&ResourceItem, &GridPosition)>,

// Inside function:
for (item, pos) in &items {
    let heat = match item.resource_type {
        ResourceType::Waste => 2.0,
        ResourceType::Ore => 0.5,
        _ => 0.0,
    };
    if heat > 0.0 {
        grid.add(pos.x, pos.y, heat);
    }
}
```

## REFACTOR Phase: Quality & Design

- **Performance**: Optimizing the grid clear/rebuild. If item count is high, spatial partitioning helps. But typically `ResourceItem` count is moderate (hundreds, not millions).
- **Integration**: Ensure `RadiationGrid` is initialized in `setup.rs`.
- **UI**: Display radiation levels in Inspector or via an Overlay.

## Acceptance Criteria

- [ ] `Waste` and `Ore` items emit heat in `TemperatureGrid`.
- [ ] `Waste` and `Ore` items create radiation in `RadiationGrid`.
- [ ] Pops near radiation gain `RadiationSickness`.
- [ ] `RadiationSickness` damages health at high severity.
- [ ] `RadiationSickness` decays when away from source.
- [ ] `cargo test` passes.
