# 205: Auroral Harvesting

## Overview

Introduces the **Auroral Collector**, a specialized power generation building that harnesses the electromagnetic energy of `Magnetic Storms` (Spec 179). While normal generators struggle during storms due to interference, the Auroral Collector thrives, generating massive amounts of power (50.0) only during these events. During normal weather (`Clear`, `Rain`, etc.), it generates 0.0 power.

**Why:**
- Creates a "High Risk / High Reward" power strategy.
- Incentivizes resilience: Players might rely on Collectors and pray for storms, or use them as emergency backups.
- Adds strategic depth to weather events beyond just "hunker down".

## Dependencies

- `179` — Magnetic Storms (WeatherType::MagneticStorm)
- `042` — Energy System (PowerSource)
- `004` — Building System (BuildingType)

## RED Phase: Tests First

Write these tests in `src/layer1/energy/auroral_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::energy::{PowerSource, update_auroral_output_system};
    use crate::layer1::weather::{WeatherState, WeatherType};
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::lighting::LightSource;

    #[test]
    fn test_auroral_collector_variant_exists() {
        // Ensure the building type exists
        let collector = BuildingType::AuroralCollector;
        assert_eq!(collector.label(), "Auroral Collector");
    }

    #[test]
    fn test_auroral_output_zero_in_clear_weather() {
        let mut world = World::new();

        // Setup Weather: Clear
        world.insert_resource(WeatherState {
            current_weather: WeatherType::Clear,
            duration_remaining: 100,
        });

        // Spawn Collector
        let collector = world.spawn((
            PowerSource { output: 10.0, active: true }, // Initial dummy value
            Building { building_type: BuildingType::AuroralCollector },
            LightSource::default(), // For visual feedback check
        )).id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(update_auroral_output_system);
        schedule.run(&mut world);

        // Verify Output is 0.0
        let source = world.get::<PowerSource>(collector).unwrap();
        assert_eq!(source.output, 0.0, "Output should be 0.0 in Clear weather");

        // Verify Light is off
        let light = world.get::<LightSource>(collector).unwrap();
        assert_eq!(light.intensity, 0.0, "Light intensity should be 0.0 in Clear weather");
    }

    #[test]
    fn test_auroral_output_high_in_magnetic_storm() {
        let mut world = World::new();

        // Setup Weather: Magnetic Storm
        world.insert_resource(WeatherState {
            current_weather: WeatherType::MagneticStorm,
            duration_remaining: 100,
        });

        // Spawn Collector
        let collector = world.spawn((
            PowerSource { output: 0.0, active: true },
            Building { building_type: BuildingType::AuroralCollector },
            LightSource::default(),
        )).id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(update_auroral_output_system);
        schedule.run(&mut world);

        // Verify Output is High (e.g., 50.0)
        let source = world.get::<PowerSource>(collector).unwrap();
        assert_eq!(source.output, 50.0, "Output should be 50.0 during Magnetic Storm");

        // Verify Light is on
        let light = world.get::<LightSource>(collector).unwrap();
        assert!(light.intensity > 0.0, "Light intensity should be active during storm");
    }

    #[test]
    fn test_other_buildings_ignored() {
        let mut world = World::new();
        world.insert_resource(WeatherState {
            current_weather: WeatherType::MagneticStorm,
            duration_remaining: 100,
        });

        // Normal Generator
        let generator = world.spawn((
            PowerSource { output: 10.0, active: true },
            Building { building_type: BuildingType::Generator },
        )).id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(update_auroral_output_system);
        schedule.run(&mut world);

        // Output should remain unchanged by *this* system
        let source = world.get::<PowerSource>(generator).unwrap();
        assert_eq!(source.output, 10.0, "Normal Generator should not be modified by Auroral system");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update `BuildingType` Enum

In `src/layer1/building.rs`:

```rust
pub enum BuildingType {
    // ... existing
    AuroralCollector,
}

// In impl BuildingType:
// label() -> "Auroral Collector"
// char() -> 'Ψ'
// cost() -> Metal: 50.0, Stone: 20.0 (Expensive but powerful)
// required_tech() -> Some(Tech::Electromagnetism) // If exists, or AdvancedPower
// tier_info() -> (Category::Power, Tier::Advanced)
```

In `configure_power`:

```rust
        BuildingType::AuroralCollector => {
            entity.insert((
                PowerSource {
                    output: 0.0, // Starts inactive
                    active: true,
                },
                crate::layer1::lighting::LightSource {
                    radius: 6.0,
                    intensity: 0.0, // Starts dark
                    color: (0, 255, 255), // Cyan Aurora color
                },
                // Maybe SeismicSource?
            ));
        }
```

### 2. Implement `update_auroral_output_system`

In `src/layer1/energy/mod.rs`:

```rust
use crate::layer1::weather::{WeatherState, WeatherType};
use crate::layer1::building::{Building, BuildingType};
use crate::layer1::lighting::LightSource;

pub fn update_auroral_output_system(
    weather: Res<WeatherState>,
    mut query: Query<(&mut PowerSource, &mut LightSource, &Building)>,
) {
    let is_storm = weather.current_weather == WeatherType::MagneticStorm;
    let target_output = if is_storm { 50.0 } else { 0.0 };
    let target_intensity = if is_storm { 1.0 } else { 0.0 };

    for (mut source, mut light, building) in query.iter_mut() {
        if building.building_type == BuildingType::AuroralCollector {
            // Update Power
            if (source.output - target_output).abs() > f32::EPSILON {
                source.output = target_output;
            }

            // Update Visuals
            if (light.intensity - target_intensity).abs() > f32::EPSILON {
                light.intensity = target_intensity;
            }
        }
    }
}
```

### 3. Register System

In `src/layer1/systems.rs`:
Add `energy::update_auroral_output_system` to `SimulationUpdate` schedule, preferably *before* `energy::power_grid_system`.

## REFACTOR Phase: Quality & Design

- **Optimization:** The system iterates all buildings with PowerSource. Filter query by `With<AuroralCollector>` component if added, or rely on `BuildingType` check (acceptable for prototype).
- **Interpolation:** Smoothly ramp up power/light intensity instead of instant toggle for better visual feel (requires `AuroralState` component).
- **Feedback:** Add a "Hum" sound effect when active (via `AudioSource`).

## Acceptance Criteria

- [ ] `AuroralCollector` is constructible.
- [ ] Output is 0.0 in normal weather.
- [ ] Output is 50.0 during Magnetic Storm.
- [ ] Light turns on during Magnetic Storm.
- [ ] `cargo test` passes.

## Technical Guidance

- Ensure `WeatherType::MagneticStorm` is available (Spec 179).
- The system should run every tick or only on weather change (event-driven optimization optional but recommended later).

## Questions

- *Builder: Should the collector take damage during the storm?*
*Architect: No, the Auroral Collector is purpose-built to withstand magnetic storms. However, surrounding normal electronics will still suffer damage.*
- *Architect: No, it is designed for it. Other buildings take damage (Spec 179).*
