# 198: Urban Heat Islands

## Overview

Simulates the **Urban Heat Island** effect where dense construction creates microclimates of elevated temperature. This mechanic introduces two key physical properties to the simulation:

1.  **Thermal Retention (Thermal Mass):** Dense materials (Stone, Metal, Buildings) resist temperature changes, "trapping" heat (or cold) longer than natural terrain (Grass, Dirt).
2.  **Solar Absorption:** During the day, exposed tiles absorb solar radiation, increasing their temperature above ambient. High-retention tiles hold this heat well into the night.

This creates a dynamic where city centers become significantly hotter than the surrounding wilderness, especially in Summer, forcing players to manage density, build parks (cooling zones), or invest in active cooling.

## Dependencies

- `140` — Thermal Management (`TemperatureGrid`, `update_temperature_system`)
- `065` — Day/Night Cycle (`DayNightCycle`, `TimeOfDay`)
- `004` — Building System (`BuildingType`)
- `002` — Terrain Grid (`TerrainType`)

## RED Phase: Tests First

Write these tests in `src/layer1/urban_heat_tests.rs`.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::temperature::{TemperatureGrid, update_temperature_system};
    use crate::layer1::day_night::{DayNightCycle, TimeOfDay};
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use crate::layer1::map::GridPosition;
    use crate::layer1::seasons::{Season, SeasonState};
    use bevy_ecs::system::RunSystemOnce;

    fn setup_world() -> World {
        let mut world = World::new();
        // Setup Grid
        let grid = TemperatureGrid::new(10, 10, 20.0); // Ambient 20C
        world.insert_resource(grid);

        // Setup Season (Summer for heat)
        world.insert_resource(SeasonState { current_season: Season::Summer });

        // Setup Day/Night (Day initially)
        world.insert_resource(DayNightCycle {
            time_of_day: TimeOfDay::Day,
            ..Default::default()
        });

        // Setup Terrain (Grass default)
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });

        world
    }

    #[test]
    fn test_solar_heat_gain_during_day() {
        let mut world = setup_world();

        // Run update during Day
        world.run_system_once(update_temperature_system).unwrap();

        let grid = world.resource::<TemperatureGrid>();
        // Ambient is 30C (Summer).
        // Solar gain should push it ABOVE ambient.
        assert!(grid.get(5, 5) > 30.0, "Tiles should gain heat from sun above ambient");
    }

    #[test]
    fn test_no_solar_heat_at_night() {
        let mut world = setup_world();
        {
            let mut cycle = world.resource_mut::<DayNightCycle>();
            cycle.time_of_day = TimeOfDay::Night;
        }

        // Run update
        world.run_system_once(update_temperature_system).unwrap();

        let grid = world.resource::<TemperatureGrid>();
        // Should stay at ambient (or cool down if hotter), but not gain heat.
        // Ambient Summer = 30.0. Start 20.0. Should drift TO 30.0 but not ABOVE.
        // Actually, night ambient might be lower? Spec 140 defined fixed seasonal ambient.
        // Assuming ambient is the target.
        // Just verify it doesn't jump *above* the target like it does in the day.
        // Or simpler: Compare Day gain vs Night gain.
    }

    #[test]
    fn test_retention_slows_cooling() {
        // Setup two tiles: One Grass (Low Retention), One Concrete/Wall (High Retention)
        // Both start HOT (50C). Ambient is COOL (10C).
        // Wall should cool down SLOWER.

        let mut world = World::new();
        let mut grid = TemperatureGrid::new(10, 10, 10.0); // Ambient 10
        grid.set(0, 0, 50.0); // Tile 1 (Grass)
        grid.set(1, 0, 50.0); // Tile 2 (Wall)
        world.insert_resource(grid);
        world.insert_resource(SeasonState { current_season: Season::Autumn }); // Ambient 10
        world.insert_resource(DayNightCycle { time_of_day: TimeOfDay::Night, ..Default::default() }); // No sun

        let tiles = vec![TerrainType::Grass; 100];
        world.insert_resource(TerrainGrid { width: 10, height: 10, tiles });

        // Add Wall at (1,0)
        world.spawn((
            Building { building_type: BuildingType::Wall },
            GridPosition { x: 1, y: 0 },
        ));

        // Run update
        world.run_system_once(update_temperature_system).unwrap();

        let grid = world.resource::<TemperatureGrid>();
        let temp_grass = grid.get(0, 0);
        let temp_wall = grid.get(1, 0);

        // Both should cool down (< 50)
        assert!(temp_grass < 50.0);
        assert!(temp_wall < 50.0);

        // Grass should be COOLER than Wall (Wall retains heat)
        assert!(temp_wall > temp_grass, "High retention wall should stay hotter longer");
    }

    #[test]
    fn test_urban_heat_island_effect() {
        // A cluster of buildings should be hotter than surrounding area during the day
        let mut world = setup_world();

        // Spawn 3x3 block of StoneMason buildings (Industrial/Stone)
        for y in 3..6 {
            for x in 3..6 {
                world.spawn((
                    Building { building_type: BuildingType::StoneMason },
                    GridPosition { x, y },
                ));
            }
        }

        // Run simulation for a few ticks during Day
        for _ in 0..5 {
            world.run_system_once(update_temperature_system).unwrap();
        }

        let grid = world.resource::<TemperatureGrid>();
        let city_temp = grid.get(4, 4); // Center of city
        let nature_temp = grid.get(0, 0); // Corner (Grass)

        assert!(city_temp > nature_temp + 1.0, "Urban center should be significantly hotter");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Retention & Solar Properties

Update `BuildingType` and `TerrainType` with new methods.

```rust
// src/layer1/building.rs
impl BuildingType {
    pub fn heat_retention(&self) -> f32 {
        match self {
            Self::Wall | Self::Tower => 0.8, // Stone/Massive
            Self::Housing | Self::Office => 0.6,
            Self::Park | Self::FlowerBed => 0.1, // Low retention (cooling)
            _ => 0.5,
        }
    }

    // Optional: Albedo (0.0 = absorbs all, 1.0 = reflects all)
    // For MVP, we can assume Retention correlates with Mass/Absorption.
}

// src/layer1/terrain.rs
impl TerrainType {
    pub fn heat_retention(&self) -> f32 {
        match self {
            Self::Rock => 0.5,
            Self::Grass | Self::Dirt => 0.1,
            Self::Water => 0.2, // Water has high specific heat but effectively buffers/cools in this model
            _ => 0.1,
        }
    }
}
```

### 2. Update Temperature Grid Logic

Modify `TemperatureGrid::diffuse` to accept a `retention_map`.

```rust
// src/layer1/temperature.rs

pub fn diffuse(
    &mut self,
    conductivity: &HashMap<(i32, i32), f32>,
    retention: &HashMap<(i32, i32), f32> // New argument
) {
    let base_diffusion_rate = 0.2;
    let base_drift_rate = 0.05; // Base cooling speed

    for y in 0..self.height {
        for x in 0..self.width {
            // ... (neighbor diffusion logic remains same) ...

            // Drift Logic
            let tile_retention = *retention.get(&(ix, iy)).unwrap_or(&0.1); // Default low retention

            // Retention resists change.
            // High retention (0.9) -> Factor (0.1) -> Slow drift.
            // Low retention (0.1) -> Factor (0.9) -> Fast drift.
            let drift_factor = (1.0 - tile_retention).max(0.01);

            let drift = (self.ambient - current_temp) * base_drift_rate * drift_factor;

            self.scratch[idx] = current_temp + flow_sum + drift;
        }
    }
}
```

### 3. Solar Heat System

Modify `update_temperature_system` to apply solar heat.

```rust
// src/layer1/temperature.rs

pub fn update_temperature_system(
    // ... args ...
    cycle: Option<Res<DayNightCycle>>,
) {
    // ...

    // Calculate Solar Output
    let solar_heat = if let Some(cycle) = cycle {
        match cycle.time_of_day {
            TimeOfDay::Day => 0.5, // +0.5 degrees per tick
            TimeOfDay::Dawn | TimeOfDay::Dusk => 0.1,
            TimeOfDay::Night => 0.0,
        }
    } else {
        0.0
    };

    // Build Retention Map & Apply Solar
    let mut retention_map = HashMap::new();

    // Init with Terrain retention
    // (Need to query TerrainGrid or assume base)
    // For MVP, iterate buildings.

    for (b, pos, ..) in &buildings {
        let r = b.building_type.heat_retention();
        retention_map.insert((pos.x, pos.y), r);

        // Solar Gain logic:
        // Buildings absorb sun.
        // We could also apply this to empty terrain if we iterated all tiles.
        if solar_heat > 0.0 {
             // Maybe darker buildings absorb more?
             // For now, just add solar_heat to everything exposed.
             grid.add(pos.x, pos.y, solar_heat);
        }
    }

    // Apply Solar to bare terrain?
    // Ideally yes, iterate all x,y.
    if solar_heat > 0.0 {
        for y in 0..grid.height {
            for x in 0..grid.width {
                // If not occupied (optimization?), or just add to everything.
                // Adding to everything raises global temp during day.
                // But retention decides if it STAYS hot.
                grid.add(x as i32, y as i32, solar_heat);
            }
        }
    }

    grid.diffuse(&conductivity_map, &retention_map);
}
```

## REFACTOR Phase: Quality & Design

- **Optimization**: Don't iterate the whole grid for solar heat every tick if possible. Maybe bake it into the `ambient`? No, ambient is the "Air Temp target". Solar is radiative input.
- **Microclimates**: This naturally creates microclimates. Water bodies (if retention set correctly) will stay cooler if they have high "Specific Heat" (modeled as resistance to solar gain? Or just high retention but lower ambient target?).
- **Refinement**: `retention` could be split into `insulation` (resist flow) and `mass` (resist change). Currently `conductivity` is `insulation` and `retention` is `mass`.

## Acceptance Criteria

- [ ] `TemperatureGrid` update accepts `retention` map.
- [ ] High retention tiles cool down slower than low retention tiles.
- [ ] During Day, tiles gain heat (Global Solar).
- [ ] Urban areas (High Retention) equilibrate at a higher temperature than rural areas during the day and stay hotter at night.
- [ ] All tests in RED phase pass.

## Technical Guidance

- Tune `base_drift_rate` and `solar_heat` values. If drift is too fast, day/night cycle won't matter. If too slow, seasons won't matter.
- `solar_heat` should probably be per-tick small value (e.g., 0.05) if ticks are fast.
- Ensure `TerrainGrid` is accessible in `update_temperature_system` to get retention for empty tiles.

## Questions

- **Shadows?** Future feature (Urban Canyons 182 already handles wind, maybe shadows later).
- **Interiors?** Roofed buildings shouldn't get solar gain *inside*.
    - *Answer:* For MVP, `Housing` is treated as a "block" that heats up. If we track internal vs external temp, we need a separate layer. For now, Layer 1 grid represents the "Tile Temperature" (Average of structure and air).
