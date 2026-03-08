# 140: Thermal Management

## Overview

Introduces a thermodynamic simulation to the colony. Temperature is now a grid-based value (`TemperatureGrid`) affected by the environment (Seasons), heat sources (Machinery, Fire), and insulation (Walls, Airlocks).

This system replaces the binary "Winter = Damage" check from Spec 040 with a granular simulation where players must actively manage heat: building Heaters in winter and cooling/venting in summer (or industrial zones).

## Dependencies

- `002` — Terrain Grid
- `040` — Clothing & Temperature (Base logic)
- `063` — Atmospheric Simulation (Grid pattern)
- `119` — Airlock & Pressure (Flow logic)
- `125` — Grid Instability (Power logic for heaters)

## RED Phase: Tests First

```rust
// src/layer1/temperature_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::temperature::{TemperatureGrid, update_temperature_system, thermal_damage_system};
    use crate::layer1::map::GridPosition;
    use crate::layer1::health::Health;
    use crate::layer1::pop::Pop;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::seasons::{Season, SeasonState};

    #[test]
    fn test_grid_initialization_to_ambient() {
        let mut world = World::new();
        world.insert_resource(SeasonState { current_season: Season::Winter }); // -5.0 C

        // Initialize grid
        let grid = TemperatureGrid::new(10, 10, -5.0);

        assert_eq!(grid.get(5, 5), -5.0);
    }

    #[test]
    fn test_heat_source_emission() {
        let mut world = World::new();
        let mut grid = TemperatureGrid::new(10, 10, 0.0);
        world.insert_resource(grid);

        // Spawn Heater
        world.spawn((
            Building { building_type: BuildingType::Heater, ..Default::default() },
            GridPosition { x: 5, y: 5 },
        ));

        // Run update
        update_temperature_system(&mut world);

        let grid = world.resource::<TemperatureGrid>();
        assert!(grid.get(5, 5) > 0.0, "Heater should raise temperature");
    }

    #[test]
    fn test_diffusion_and_insulation() {
        let mut world = World::new();
        let mut grid = TemperatureGrid::new(5, 1, 0.0);
        grid.set(0, 0, 100.0); // Hot source
        world.insert_resource(grid);

        // Wall at (2, 0)
        world.spawn((
            Building { building_type: BuildingType::Wall, ..Default::default() },
            GridPosition { x: 2, y: 0 },
        ));

        // Run update (multiple ticks for diffusion)
        for _ in 0..10 {
            update_temperature_system(&mut world);
        }

        let grid = world.resource::<TemperatureGrid>();
        // (1,0) should be warm (neighbor of source)
        assert!(grid.get(1, 0) > 10.0);
        // (3,0) should be cold (blocked by wall)
        // Walls are not perfect insulators (0.05 conductivity), but significantly colder than open air
        assert!(grid.get(3, 0) < grid.get(1, 0) * 0.5, "Wall should block most heat");
    }

    #[test]
    fn test_thermal_damage() {
        let mut world = World::new();
        let mut grid = TemperatureGrid::new(10, 10, 0.0);
        grid.set(5, 5, -20.0); // Freezing
        world.insert_resource(grid);

        let pop = world.spawn((
            Pop::default(),
            Health { current: 100.0, max: 100.0 },
            GridPosition { x: 5, y: 5 },
        )).id();

        thermal_damage_system(&mut world);

        let health = world.get::<Health>(pop).unwrap();
        assert!(health.current < 100.0, "Freezing temp should damage Pop");
    }

    #[test]
    fn test_ambient_drift() {
        // Tiles should slowly drift towards season ambient temp if not insulated
        let mut world = World::new();
        let ambient = -10.0;
        world.insert_resource(SeasonState { current_season: Season::Winter }); // Assume Winter = -10
        let mut grid = TemperatureGrid::new(10, 10, 20.0); // Start warm (20 C)
        world.insert_resource(grid);

        // Run update
        update_temperature_system(&mut world);

        let grid = world.resource::<TemperatureGrid>();
        assert!(grid.get(0, 0) < 20.0, "Should cool down towards ambient");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Temperature Grid Resource

```rust
// src/layer1/temperature.rs

#[derive(Resource)]
pub struct TemperatureGrid {
    pub width: usize,
    pub height: usize,
    pub values: Vec<f32>,
    pub scratch: Vec<f32>,
    pub ambient: f32, // Target temp for edges/drift
}

impl TemperatureGrid {
    pub fn new(width: usize, height: usize, ambient: f32) -> Self {
        // ... standard grid impl ...
        Self {
            width, height,
            values: vec![ambient; width * height],
            scratch: vec![ambient; width * height],
            ambient,
        }
    }

    // Standard get/set/add...

    pub fn diffuse(&mut self, conductivity: &HashMap<(i32, i32), f32>) {
        // Box blur with conductivity
        // Edges are fixed to self.ambient
        // Inner tiles mix with neighbors based on conductivity
        // Also apply slight drift to ambient? Or let edges handle it?
        // Let edges handle it for closed systems.
    }
}
```

### 2. Building Updates

Update `BuildingType` to include `Heater`.
Update `BuildingType::thermal_conductivity()`:
- Wall: 0.05
- Airlock (Closed): 0.1
- Airlock (Open): 1.0
- Vent: 1.0
- Empty: 1.0

### 3. Update System

```rust
pub fn update_temperature_system(
    mut grid: ResMut<TemperatureGrid>,
    season: Res<SeasonState>,
    buildings: Query<(&Building, &GridPosition)>,
) {
    // 1. Update Ambient from Season
    grid.ambient = season.current_season.base_temperature();

    // 2. Apply Heat Sources
    for (b, pos) in &buildings {
        let heat = match b.building_type {
            BuildingType::Heater => 5.0, // Degrees per tick added
            BuildingType::Smelter => 2.0,
            BuildingType::Reactor => 10.0,
            _ => 0.0,
        };
        grid.add(pos.x, pos.y, heat);
    }

    // 3. Build Conductivity Map
    let map = HashMap::new();
    // ... populate from buildings ...

    // 4. Diffuse
    grid.diffuse(&map);
}
```

### 4. Damage System

Replace old `hypothermia_system` in `clothing.rs` (or update it) to read from `TemperatureGrid`.

```rust
pub fn thermal_damage_system(
    grid: Res<TemperatureGrid>,
    mut pops: Query<(&mut Health, &GridPosition, Option<&Clothing>)>,
) {
    for (mut health, pos, clothing) in &mut pops {
        let temp = grid.get(pos.x, pos.y);

        // Safe range: 10C to 30C (approx)
        // Clothing extends safe range to -10C

        let min_safe = if clothing.is_some() { -10.0 } else { 10.0 };
        let max_safe = 35.0; // Hyperthermia threshold

        if temp < min_safe {
            health.take_damage(0.5); // Hypothermia
        } else if temp > max_safe {
            health.take_damage(0.5); // Heatstroke
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Optimization**: Run diffusion every N ticks, or use a "Chunk" system.
- **Visuals**: Add a `TemperatureOverlay` to the map rendering.
- **Fire**: Ensure Fire entities add massive heat (e.g., +50C) to the grid.
- **Vacuum**: Vacuum (Space) temp should be absolute zero (-270C)? Or just "Very Cold" (-50C) for game balance? Seasons handle "Planet Surface" temp.

## Acceptance Criteria

- [ ] `TemperatureGrid` initializes and diffuses.
- [ ] Heaters raise temperature.
- [ ] Walls insulate (low conductivity).
- [ ] Pops take damage in extreme cold/heat.
- [ ] Clothing mitigates cold damage.
- [ ] `cargo test` passes.

## Technical Guidance

- Use `f32` for temperature (Celsius).
- Be careful with `diffuse` logic—conservation of energy is tricky with fixed boundary conditions (ambient). A simple mix + boundary drift is sufficient for gameplay.
- Reuse `PressureGrid` double-buffering pattern.

## Questions

- Should "Vacuum" tiles (outside hull) always be at ambient temp? Yes, effectively infinite heat sink.
- *Architect:* Confirmed, Vacuum tiles act as an infinite heat sink locked to ambient temperature.
