# 190: Atmospheric Tides

## Overview

Introduces dynamic global air pressure cycles ("Atmospheric Tides") that affect wind speed and movement difficulty. The planet "breathes," cycling between High Pressure (thick air, strong winds, slow movement) and Low Pressure (thin air, weak winds, fast movement). This creates a rhythm to the game loop where players must time their logistics and power generation.

## Dependencies

- `182` — Urban Canyons (WindGrid, GlobalWind)
- `001` — Simulation Time

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/atmosphere_tides_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::atmosphere::{AtmosphericTide, update_atmospheric_tide_system, BaseGlobalWind, sync_global_wind_system};
    use crate::layer1::wind::GlobalWind;
    use crate::shared::time::SimulationTime;
    use glam::Vec2;

    #[test]
    fn test_tide_oscillation() {
        let mut world = World::new();
        world.insert_resource(SimulationTime::default());
        world.insert_resource(AtmosphericTide::default());

        // Default pressure should be 1.0
        assert_eq!(world.resource::<AtmosphericTide>().pressure, 1.0);

        // Advance time to 1/4 cycle (Peak High Pressure)
        // Assuming cycle is 1000 ticks
        let mut time = SimulationTime::default();
        time.tick = 250;
        world.insert_resource(time);

        // Update tide
        let mut schedule = Schedule::new();
        schedule.add_systems(update_atmospheric_tide_system);
        schedule.run(&mut world);

        let pressure = world.resource::<AtmosphericTide>().pressure;
        assert!(pressure > 1.2, "Pressure should be high (got {})", pressure);

        // Advance to 3/4 cycle (Peak Low Pressure)
        let mut time = SimulationTime::default();
        time.tick = 750;
        world.insert_resource(time);
        schedule.run(&mut world);

        let pressure = world.resource::<AtmosphericTide>().pressure;
        assert!(pressure < 0.8, "Pressure should be low (got {})", pressure);
    }

    #[test]
    fn test_wind_sync() {
        let mut world = World::new();
        world.insert_resource(BaseGlobalWind { speed: 10.0, direction: Vec2::X });
        world.insert_resource(GlobalWind::default());

        // High Pressure
        world.insert_resource(AtmosphericTide { pressure: 1.5 });

        let mut schedule = Schedule::new();
        schedule.add_systems(sync_global_wind_system);
        schedule.run(&mut world);

        let effective = world.resource::<GlobalWind>();
        // Speed should be Base * Pressure
        assert!((effective.speed - 15.0).abs() < 0.01, "Wind speed should scale with pressure");
        assert_eq!(effective.direction, Vec2::X, "Direction should persist");
    }

    #[test]
    fn test_movement_cost_integration() {
        // This test requires mocking or integrating with movement_system.
        // For unit testing, we can expose a cost calculation function.
        use crate::layer1::atmosphere::calculate_atmospheric_movement_cost;

        // High Pressure (Thick Air) -> Harder to move
        let cost = calculate_atmospheric_movement_cost(1.5);
        assert!(cost > 1.0);

        // Low Pressure (Thin Air) -> Easier to move (less resistance)
        let cost = calculate_atmospheric_movement_cost(0.5);
        assert!(cost < 1.0);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Resources (`src/layer1/atmosphere.rs`)

```rust
use bevy_ecs::prelude::*;
use glam::Vec2;
use crate::layer1::wind::GlobalWind;
use crate::shared::time::SimulationTime;
use std::f32::consts::PI;

/// Represents the static "base" wind of the map, unaffected by tides.
#[derive(Resource)]
pub struct BaseGlobalWind {
    pub direction: Vec2,
    pub speed: f32,
}

impl Default for BaseGlobalWind {
    fn default() -> Self {
        Self { direction: Vec2::X, speed: 1.0 }
    }
}

/// Tracks the global atmospheric pressure.
#[derive(Resource)]
pub struct AtmosphericTide {
    /// Multiplier centered at 1.0. Range usually 0.5 to 1.5.
    pub pressure: f32,
}

impl Default for AtmosphericTide {
    fn default() -> Self {
        Self { pressure: 1.0 }
    }
}

// Cycle length in ticks
const TIDE_CYCLE_TICKS: u64 = 1000;

pub fn update_atmospheric_tide_system(
    mut tide: ResMut<AtmosphericTide>,
    time: Res<SimulationTime>,
) {
    // Simple Sine Wave: 1.0 + 0.5 * sin(t)
    let phase = (time.tick % TIDE_CYCLE_TICKS) as f32 / TIDE_CYCLE_TICKS as f32;
    let angle = phase * 2.0 * PI;
    tide.pressure = 1.0 + 0.5 * angle.sin();
}

pub fn sync_global_wind_system(
    base: Res<BaseGlobalWind>,
    tide: Res<AtmosphericTide>,
    mut effective: ResMut<GlobalWind>,
) {
    effective.direction = base.direction;
    effective.speed = base.speed * tide.pressure;
}

pub fn calculate_atmospheric_movement_cost(pressure: f32) -> f32 {
    // High pressure = High Drag = High Cost
    // Low pressure = Low Drag = Low Cost
    // Mapping: 0.5 -> 0.8 cost, 1.5 -> 1.2 cost
    // Formula: 0.6 + 0.4 * pressure
    0.6 + 0.4 * pressure
}
```

### 2. Integration with Execution (`src/layer1/execution.rs`)

Modify `movement_system` to include wind and pressure penalties.

```rust
// Inside movement_system loop...

// 1. Get Terrain Cost (Existing)
let terrain_cost = terrain.get(x, y).map_or(1.0, |t| t.movement_cost());

// 2. Get Wind Cost (New)
let wind_vec = wind_grid.get_wind(current_pos.x, current_pos.y);
let move_dir = (new_pos - current_pos).as_vec2();
let wind_mod = crate::layer1::wind::calculate_wind_movement_penalty(wind_vec, move_dir);

// 3. Get Pressure Cost (New)
let pressure_mod = crate::layer1::atmosphere::calculate_atmospheric_movement_cost(tide.pressure);

let total_cost = terrain_cost * wind_mod * pressure_mod;

// Apply logic...
```

## REFACTOR Phase: Quality & Design

- **Migration**: Existing setup code initializes `GlobalWind`. We must change `setup.rs` to initialize `BaseGlobalWind` instead, and let `sync_global_wind_system` handle `GlobalWind`.
- **UI**: Add a pressure indicator to the HUD/Inspector so players know why they are moving slow.
- **Visuals**: Wind particles (if implemented) should speed up/slow down.
- **Wind Turbines**: Ensure turbine power generation logic reads `GlobalWind` (effective) so it naturally scales with tide.

## Acceptance Criteria

- [ ] `AtmosphericTide` resource oscillates over time.
- [ ] `GlobalWind` speed is updated based on `BaseGlobalWind` * `pressure`.
- [ ] Movement speed tests show variance based on pressure (High pressure = slow).
- [ ] Movement speed tests show variance based on wind direction (Headwind = slow).
- [ ] `BaseGlobalWind` is initialized in setup.

## Technical Guidance

- Ensure `sync_global_wind_system` runs *before* `update_wind_system` (Layer 1 Wind) so the grid calculation uses the fresh effective speed.
- Ensure `update_atmospheric_tide_system` runs before `sync_global_wind_system`.
- In `execution.rs`, you will need to add `Res<WindGrid>` and `Res<AtmosphericTide>` to the `movement_system` signature. Handle the `Option` case gracefully for tests where they might be missing.
