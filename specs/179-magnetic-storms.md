# 179: Magnetic Storms

## Overview

Introduces **Magnetic Storms** as a new weather event. During a magnetic storm, all active power consumers suffer from interference, increasing their effective power demand by 50%. This forces players to manage their grid carefully during storms, either by building excess capacity or manually shutting down non-essential systems to avoid **Overload** (Spec 125).

**Why:**
- Adds strategic depth to the Energy System.
- Interacts with Weather and Grid Instability.
- Creates emergent "hunker down" moments.

## Dependencies

- `079` — Weather Events (Framework)
- `042` — Energy System (PowerConsumer)
- `125` — Grid Instability (Overload mechanic)

## RED Phase: Tests First

Write these tests in `src/layer1/weather/magnetic_storm_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::energy::{PowerConsumer, PowerSource, power_grid_system};
    use crate::layer1::weather::{WeatherState, WeatherType};
    use crate::layer1::map::GridPosition;
    use crate::layer1::building::{Building, BuildingType};

    #[test]
    fn test_magnetic_storm_variant_exists() {
        // This test ensures the variant is added to the enum
        let storm = WeatherType::MagneticStorm;
        assert_eq!(storm.name(), "Magnetic Storm");
    }

    #[test]
    fn test_magnetic_storm_increases_demand() {
        let mut world = World::new();

        // Setup Weather: Magnetic Storm
        world.insert_resource(WeatherState {
            current_weather: WeatherType::MagneticStorm,
            duration_remaining: 100,
        });

        // Generator: 20 Output
        world.spawn((
            PowerSource { output: 20.0, active: true },
            GridPosition { x: 0, y: 0 },
            Building { building_type: BuildingType::Generator },
        ));

        // Consumer: 10 Demand (Base)
        // With +50% from storm, effective demand should be 15.
        // Net = 20 - 15 = 5.
        // Supply Ratio = 20 / 15 = 1.33 (Active)
        let consumer = world.spawn((
            PowerConsumer { demand: 10.0, active: true },
            GridPosition { x: 0, y: 1 },
            crate::layer1::energy::Conduit, // Connect
            Building { building_type: BuildingType::Smelter },
        )).id();

        // Run grid system
        power_grid_system(&mut world);

        // Calculate stats manually to verify internal logic (or expose via stats resource)
        // Since we can't easily peek inside the system's local variables,
        // we can infer from the Overload behavior or check if `net` calculation was affected.
        // A better way is to check `GridOverloadEvent` if we trigger overload.

        // Let's create a scenario that WOULD overload only during a storm.
        // Gen: 10. Cons: 8.
        // Normal: 8/10 = 0.8 load. Safe.
        // Storm: 8 * 1.5 = 12. 12/10 = 1.2 load. Overload? (Threshold is 1.5).
        // Let's make it hit threshold.
        // Gen: 10. Cons: 10.
        // Normal: 1.0 load.
        // Storm: 15.0 load. 1.5 ratio. Borderline.
        // Gen: 10. Cons: 11.
        // Normal: 1.1 load.
        // Storm: 11 * 1.5 = 16.5. 1.65 ratio. Overload!
    }

    #[test]
    fn test_magnetic_storm_triggers_overload() {
        let mut world = World::new();
        world.init_resource::<Events<crate::layer1::energy::GridOverloadEvent>>();

        // Setup Weather: Magnetic Storm
        world.insert_resource(WeatherState {
            current_weather: WeatherType::MagneticStorm,
            duration_remaining: 100,
        });

        // Generator: 10 Output
        let gen = world.spawn((
            PowerSource { output: 10.0, active: true },
            GridPosition { x: 0, y: 0 },
            Building { building_type: BuildingType::Generator },
            crate::layer1::health::Health { current: 100.0, max: 100.0 },
        )).id();

        // Consumer: 11 Demand
        // Base Load: 1.1 (Safe)
        // Storm Load: 11 * 1.5 = 16.5 -> 1.65 Ratio (> 1.5 Threshold)
        world.spawn((
            PowerConsumer { demand: 11.0, active: true },
            GridPosition { x: 0, y: 1 },
            crate::layer1::energy::Conduit,
            Building { building_type: BuildingType::Smelter },
        ));

        // Run system enough times to trigger probabilistic overload
        // (Chance is ~0.75% per tick at 1.65 ratio)
        let mut triggered = false;
        for _ in 0..500 {
            power_grid_system(&mut world);
            let events = world.resource::<Events<crate::layer1::energy::GridOverloadEvent>>();
            if !events.is_empty(&world.resource::<Events<crate::layer1::energy::GridOverloadEvent>>().reader_id()) {
                // Need manual event reader in test or check health
                if world.get::<crate::layer1::health::Health>(gen).unwrap().current < 100.0 {
                    triggered = true;
                    break;
                }
            }
            // Clear events manually if needed, or rely on frame update
        }

        assert!(triggered, "Generator should take overload damage during Magnetic Storm with 110% base load");
    }

    #[test]
    fn test_normal_weather_no_overload() {
        let mut world = World::new();
        // Setup Weather: Clear
        world.insert_resource(WeatherState {
            current_weather: WeatherType::Clear,
            duration_remaining: 100,
        });

        // Same setup: Gen 10, Cons 11.
        let gen = world.spawn((
            PowerSource { output: 10.0, active: true },
            GridPosition { x: 0, y: 0 },
            Building { building_type: BuildingType::Generator },
            crate::layer1::health::Health { current: 100.0, max: 100.0 },
        )).id();

        world.spawn((
            PowerConsumer { demand: 11.0, active: true },
            GridPosition { x: 0, y: 1 },
            crate::layer1::energy::Conduit,
        ));

        for _ in 0..100 {
            power_grid_system(&mut world);
        }

        assert_eq!(world.get::<crate::layer1::health::Health>(gen).unwrap().current, 100.0, "Should NOT overload in Clear weather");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update `WeatherType` Enum

In `src/layer1/weather.rs`:

```rust
pub enum WeatherType {
    // ... existing
    MagneticStorm,
}

// In impl WeatherType:
// name() -> "Magnetic Storm"
// speed_modifier() -> 1.0 (Doesn't affect movement, or maybe 0.9?)
```

### 2. Update `power_grid_system`

In `src/layer1/energy/mod.rs`:

```rust
use crate::layer1::weather::{WeatherState, WeatherType};

pub fn power_grid_system(world: &mut World) {
    // ... existing blackout check ...

    // Check Weather
    let demand_multiplier = world.get_resource::<WeatherState>()
        .map_or(1.0, |w| if w.current_weather == WeatherType::MagneticStorm { 1.5 } else { 1.0 });

    // ... bfs_grid loop ...

    // Inside the loop where total_demand is used:
    // Apply multiplier to effective demand for calculation purposes
    let effective_demand = total_demand * demand_multiplier;

    // Use `effective_demand` for Net calculation
    let mut net = total_production - effective_demand;

    // ... battery logic uses `net` ...

    // Use `effective_demand` for Overload calculation
    let total_available = effective_demand + net; // Recalculate based on new net?
    // Actually:
    // net = production - effective_demand
    // supply_ratio = (production + battery) / effective_demand

    let overload_ratio = if total_production > 0.0 {
        effective_demand / total_production
    } else {
        1.0
    };

    // ... rest is same ...
}
```

### 3. Update `pick_weather_for_season`

In `src/layer1/weather.rs`:
- Add `WeatherType::MagneticStorm` to probability tables (e.g., small chance in Summer/Winter).

## REFACTOR Phase: Quality & Design

- **Visuals**: Add a "Static" overlay shader during Magnetic Storms.
- **Feedback**: Add a Notification when `MagneticStorm` begins: "Magnetic interference detected. Power grid unstable."
- **Upgrades**: Introduce `ShieldedConduit` or `HardenedGenerator` later to ignore this penalty.
- **Optimization**: Ensure `demand_multiplier` is calculated once per tick, not per grid.

## Acceptance Criteria

- [ ] `WeatherType::MagneticStorm` exists.
- [ ] During Magnetic Storm, power demand is effectively 150%.
- [ ] Overload triggers correctly at lower base loads during storm.
- [ ] Normal weather behaves as before.
- [ ] `cargo test` passes.

## Technical Guidance

- Import `WeatherState` in `energy/mod.rs`.
- Be careful with float comparisons in tests.
- Ensure `Chronicle` event is fired when weather changes to `MagneticStorm`.

## Questions

- *Builder: Should Magnetic Storm affect Battery discharge rate?*
*Architect: No, it just completely disables electronics. Modifying discharge rates adds unnecessary complexity to the energy simulation.*
- *Architect: No, batteries just drain faster because the Net is lower. The 1.5x demand handles it.*
