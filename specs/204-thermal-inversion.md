# 204: Thermal Inversion

## Overview

A dangerous weather event where cold air traps warm air (and pollution) near the ground.
Normally, `Smog` (Industrial Waste) dissipates into the upper atmosphere. During **Thermal Inversion**:
- **Dissipation**: Stops completely (or slows to 10%).
- **Accumulation**: Smog from factories builds up rapidly.
- **Health**: High smog levels cause `Suffocation` or `Toxicity` damage to Pops outdoors.
- **Visuals**: Map becomes hazy/grey.

## Dependencies

- `063` — Atmospheric Simulation (Smog/Gas Grid)
- `049` — Industrial Waste (Pollution source)
- `079` — Weather Events (Event system)

## RED Phase: Tests First

Write these tests in `src/layer1/weather_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::atmosphere::{AtmosphereGrid, DiffusionConfig, simulate_diffusion_system};
    use crate::layer1::weather::{WeatherState, WeatherType};
    use crate::layer1::map::GridPosition;

    #[test]
    fn test_inversion_halts_diffusion() {
        let mut world = World::new();
        let mut grid = AtmosphereGrid::default();
        // Set pollution at (10,10)
        grid.set_gas(10, 10, crate::layer1::atmosphere::GasType::Smog, 100.0);
        world.insert_resource(grid);

        // Default Config: allows diffusion
        world.insert_resource(DiffusionConfig { rate: 0.1, vertical_escape: 0.05 });

        // Set Weather to Inversion
        world.insert_resource(WeatherState {
            current: WeatherType::ThermalInversion,
            duration: 100
        });

        // Run modified diffusion system
        simulate_diffusion_system(&mut world);

        let grid = world.get_resource::<AtmosphereGrid>().unwrap();
        let smog = grid.get_gas(10, 10, crate::layer1::atmosphere::GasType::Smog);

        // Should be close to 100.0 (no vertical escape)
        // With normal weather, it would lose 5% (to 95.0)
        assert!(smog > 99.0);
    }

    #[test]
    fn test_smog_damage_during_inversion() {
        let mut world = World::new();
        // Setup Pop in smog
        let pop = world.spawn((
            crate::layer1::pop::Pop,
            crate::layer1::health::Health { current: 100.0, max: 100.0 },
            GridPosition { x: 10, y: 10 },
        )).id();

        // Setup heavy smog at pos
        let mut grid = AtmosphereGrid::default();
        grid.set_gas(10, 10, crate::layer1::atmosphere::GasType::Smog, 200.0); // Toxic level
        world.insert_resource(grid);

        // Run damage system
        apply_smog_damage_system(&mut world);

        let health = world.get::<crate::layer1::health::Health>(pop).unwrap();
        assert!(health.current < 100.0);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update Enums

Update `WeatherType` in `src/layer1/weather.rs` to include `ThermalInversion`.

### 2. Modify Diffusion System (`src/layer1/atmosphere.rs`)

```rust
use bevy_ecs::prelude::*;
use crate::layer1::weather::{WeatherState, WeatherType};

pub fn simulate_diffusion_system(
    mut grid: ResMut<AtmosphereGrid>,
    config: Res<DiffusionConfig>,
    weather: Res<WeatherState>,
) {
    let vertical_factor = if weather.current == WeatherType::ThermalInversion {
        0.0 // Trapped!
    } else {
        config.vertical_escape
    };

    // ... Standard diffusion logic ...
    // When calculating loss to "sky", use vertical_factor
}
```

### 3. Implement Damage System

```rust
pub fn apply_smog_damage_system(
    grid: Res<AtmosphereGrid>,
    mut query: Query<(&GridPosition, &mut Health), With<Pop>>,
) {
    for (pos, mut health) in query.iter_mut() {
        let smog_level = grid.get_gas(pos.x, pos.y, GasType::Smog);
        if smog_level > 150.0 { // Toxicity Threshold
            health.current -= 1.0; // Damage per tick
        }
    }
}
```

### 4. Event Generation

Add `ThermalInversion` to `generate_weather_event` table in `src/layer1/weather.rs` with low probability (e.g., 5% chance in Winter).

## REFACTOR Phase: Quality & Design

- **UI**: Add a "Haze" overlay shader or char tint to map.
- **Notification**: Alert player "Thermal Inversion Warning: Pollution Trapped".
- **Counterplay**: Building "Air Scrubbers" becomes critical.

## Acceptance Criteria

- [ ] `WeatherType::ThermalInversion` exists.
- [ ] Smog vertical dissipation stops during event.
- [ ] Pops take damage in high smog.
- [ ] Tests pass.

## Questions

*Builder: add questions here if spec is unclear.*
