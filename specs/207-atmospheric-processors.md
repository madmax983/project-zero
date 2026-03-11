# 207: Atmospheric Processors

## Overview

The planet is hostile. Toxic atmosphere and extreme temperatures make life difficult. `Atmospheric Processors` are massive, late-game buildings that slowly terraform the planet, reducing global toxicity and stabilizing temperature. This introduces a long-term goal: changing the environment itself to make survival easier (e.g., removing the need for air scrubbers or heaters).

## Dependencies

- `042` — Energy System (Massive power consumption)
- `063` — Atmospheric Simulation (Local pollution/smog)
- `140` — Thermal Management (Temperature mechanics)
- `034` — Pop Health (Global toxicity damage)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/atmospheric_processor_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::atmosphere::{AtmosphereGrid, DiffusionConfig};
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::energy::{EnergyGrid, EnergyNode};
    use crate::layer1::map::GridPosition;
    use crate::layer1::health::Health;
    use crate::layer1::pop::Pop;
    use crate::layer1::terraforming::{PlanetaryAtmosphere, update_planetary_atmosphere_system, apply_planetary_effects_system};

    #[test]
    fn test_planetary_atmosphere_initialization() {
        let atmosphere = PlanetaryAtmosphere::default();
        // Default planet is hostile
        assert!(atmosphere.toxicity > 0.5);
        assert!(atmosphere.temperature < 20.0); // Assuming cold planet start
    }

    #[test]
    fn test_processor_reduces_toxicity() {
        let mut world = World::new();
        let mut atmosphere = PlanetaryAtmosphere { toxicity: 1.0, temperature: 0.0 };
        world.insert_resource(atmosphere);

        // Spawn powered Processor in "Detoxify" mode
        world.spawn((
            Building {
                building_type: BuildingType::AtmosphericProcessor,
                ..Default::default()
            },
            GridPosition { x: 0, y: 0 },
            EnergyNode { stored: 1000.0, capacity: 1000.0, consumption: 500.0, ..Default::default() }, // Fully powered
        ));

        // Run update multiple times to simulate time passing
        for _ in 0..10 {
            update_planetary_atmosphere_system(&mut world);
        }

        let new_atmosphere = world.resource::<PlanetaryAtmosphere>();
        assert!(new_atmosphere.toxicity < 1.0, "Toxicity should decrease with active processor");
    }

    #[test]
    fn test_processor_requires_power() {
        let mut world = World::new();
        let mut atmosphere = PlanetaryAtmosphere { toxicity: 1.0, temperature: 0.0 };
        world.insert_resource(atmosphere);

        // Spawn unpowered Processor
        world.spawn((
            Building {
                building_type: BuildingType::AtmosphericProcessor,
                ..Default::default()
            },
            GridPosition { x: 0, y: 0 },
            EnergyNode { stored: 0.0, capacity: 1000.0, consumption: 500.0, ..Default::default() }, // No power
        ));

        update_planetary_atmosphere_system(&mut world);

        let new_atmosphere = world.resource::<PlanetaryAtmosphere>();
        assert_eq!(new_atmosphere.toxicity, 1.0, "Toxicity should NOT change without power");
    }

    #[test]
    fn test_global_toxicity_affects_local_diffusion() {
        let mut world = World::new();
        // High toxicity planet
        world.insert_resource(PlanetaryAtmosphere { toxicity: 0.9, temperature: 0.0 });
        world.insert_resource(AtmosphereGrid::new(10, 10));
        world.insert_resource(DiffusionConfig::default());

        apply_planetary_effects_system(&mut world);

        let grid = world.resource::<AtmosphereGrid>();
        // High global toxicity means local smog stays longer (lower decay/diffusion rate)
        // Or higher floor. Let's assume it modifies diffusion_rate.
        assert!(grid.diffusion_rate > 0.99, "High toxicity should increase retention (slower decay)");
    }

    #[test]
    fn test_low_toxicity_accelerates_decay() {
        let mut world = World::new();
        // Clean planet
        world.insert_resource(PlanetaryAtmosphere { toxicity: 0.1, temperature: 0.0 });
        world.insert_resource(AtmosphereGrid::new(10, 10));
        world.insert_resource(DiffusionConfig::default());

        apply_planetary_effects_system(&mut world);

        let grid = world.resource::<AtmosphereGrid>();
        assert!(grid.diffusion_rate < 0.99, "Low toxicity should decrease retention (faster decay)");
    }

    #[test]
    fn test_global_toxicity_damages_pops() {
        let mut world = World::new();
        world.insert_resource(PlanetaryAtmosphere { toxicity: 1.0, temperature: 0.0 }); // Max toxicity

        let pop = world.spawn((
            Pop::default(),
            Health { current: 100.0, max: 100.0 },
            GridPosition { x: 0, y: 0 },
        )).id();

        apply_planetary_effects_system(&mut world);

        let health = world.get::<Health>(pop).unwrap();
        assert!(health.current < 100.0, "Global toxicity should damage exposed pops");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Planetary Atmosphere Resource (`src/layer1/terraforming.rs`)

```rust
use bevy_ecs::prelude::*;

#[derive(Resource, Debug, Clone)]
pub struct PlanetaryAtmosphere {
    /// 0.0 = Earth-like, 1.0 = Toxic Wasteland
    pub toxicity: f32,
    /// Global average temperature offset (Celcius)
    pub temperature: f32,
}

impl Default for PlanetaryAtmosphere {
    fn default() -> Self {
        Self {
            toxicity: 0.8, // Start hostile
            temperature: -20.0, // Start cold
        }
    }
}
```

### 2. Update System

```rust
use crate::layer1::building::{Building, BuildingType};
use crate::layer1::energy::EnergyNode;

pub fn update_planetary_atmosphere_system(
    mut atmosphere: ResMut<PlanetaryAtmosphere>,
    query: Query<(&Building, &EnergyNode)>,
) {
    let mut toxicity_change = 0.0;
    let mut temp_change = 0.0;

    for (building, energy) in query.iter() {
        if building.building_type == BuildingType::AtmosphericProcessor {
            // Check if powered (simple check: stored >= consumption per tick requirement)
            // Assuming consumption is handled elsewhere and stored represents available energy
            if energy.stored >= energy.consumption {
                // Determine mode. For MVP, assume it reduces toxicity and raises temp (terraforming).
                // Future: Component `ProcessorMode` { Detoxify, Heat, Cool }

                // Very slow change per tick
                toxicity_change -= 0.0001;
                temp_change += 0.001;
            }
        }
    }

    // Apply changes clamped
    atmosphere.toxicity = (atmosphere.toxicity + toxicity_change).clamp(0.0, 1.0);
    atmosphere.temperature = (atmosphere.temperature + temp_change).clamp(-100.0, 100.0);
}
```

### 3. Effect System

```rust
use crate::layer1::atmosphere::AtmosphereGrid;
use crate::layer1::health::Health;
use crate::layer1::pop::Pop;

pub fn apply_planetary_effects_system(
    atmosphere: Res<PlanetaryAtmosphere>,
    mut grid: ResMut<AtmosphereGrid>,
    mut query: Query<&mut Health, With<Pop>>,
) {
    // 1. Modify Atmosphere Grid Decay
    // Base retention is 0.99.
    // High toxicity -> 0.999 (Smog lingers)
    // Low toxicity -> 0.90 (Smog clears fast)
    let toxicity_factor = atmosphere.toxicity;
    grid.diffusion_rate = 0.90 + (toxicity_factor * 0.099); // Maps 0.0->0.90, 1.0->0.999

    // 2. Global Health Damage (if toxicity is high)
    if atmosphere.toxicity > 0.5 {
        let damage = (atmosphere.toxicity - 0.5) * 0.05; // Small constant tick damage
        for mut health in query.iter_mut() {
            health.take_damage(damage);
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Performance**: Global updates are cheap (once per tick).
- **Architecture**: Move `PlanetaryAtmosphere` to `src/layer1/atmosphere.rs` or keep in new `terraforming.rs` if it grows.
- **Config**: Move change rates (0.0001) to constants or a `TerraformingConfig` resource.
- **Visuals**: Link `PlanetaryAtmosphere` to sky color/shader uniforms.
- **UI**: Add a global bar to the UI showing Terraform Progress.

## Acceptance Criteria

- [ ] `PlanetaryAtmosphere` resource exists and initializes.
- [ ] `AtmosphericProcessor` building can be built (add to `BuildingType`).
- [ ] Powered processors slowly reduce global `toxicity`.
- [ ] Global toxicity affects local smog decay rate.
- [ ] High global toxicity damages all pops slowly.
- [ ] `cargo test` passes.

## Technical Guidance

- Use `BuildingType::AtmosphericProcessor` (requires adding variant).
- Ensure `EnergyNode` logic works (processors are high load).
- Update `AtmosphereGrid` to allow external modification of `diffusion_rate` (it's public in current code, good).
- Add systems to `SimulationSchedule`.

## Questions

- Should terraforming be reversible? (Yes, if machines stop, planet might drift back or stay stable. For now, stable).
- Should there be "seasons" for terraforming? (No, linear progress for now).
  - *Architect:* No, terraforming progress is linear for the MVP.

*Architect:* Terraforming progress is strictly linear and stable for the MVP.
