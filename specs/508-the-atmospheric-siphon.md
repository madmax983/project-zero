# Spec 508: The Atmospheric Siphon

## 1. Overview
The colony constructs massive "Siphon Towers" to harvest atmospheric gases, condensing them into high-value trade goods. However, aggressive siphoning thins the atmosphere, drastically lowering the planet's temperature and exposing surface Pops to extreme radiation.

## 2. Dependencies
- `063` Atmospheric Simulation
- `039` Trade System
- `140` Thermal Management

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_siphon_tower_produces_condensed_gas() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(Update, siphon_atmospheric_gas_system);

        let tower_entity = app.world_mut().spawn((
            AtmosphericSiphon {
                extraction_rate: 10.0,
                accumulated_gas: 0.0,
            },
        )).id();

        app.update();

        let gas = app.world().get::<AtmosphericSiphon>(tower_entity).unwrap().accumulated_gas;
        assert!(gas > 0.0, "Siphon tower should accumulate condensed gas");
    }

    #[test]
    fn test_siphoning_reduces_global_atmospheric_density() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<GlobalAtmosphere>()
            .add_systems(Update, siphon_atmospheric_gas_system);

        let initial_density = app.world().resource::<GlobalAtmosphere>().density;

        app.world_mut().spawn((
            AtmosphericSiphon {
                extraction_rate: 10.0,
                accumulated_gas: 0.0,
            },
        ));

        app.update();

        let final_density = app.world().resource::<GlobalAtmosphere>().density;
        assert!(final_density < initial_density, "Siphoning should reduce global atmospheric density");
    }

    #[test]
    fn test_thin_atmosphere_decreases_global_temperature() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<GlobalAtmosphere>()
            .init_resource::<GlobalTemperature>()
            .add_systems(Update, update_temperature_from_atmosphere_system);

        app.world_mut().resource_mut::<GlobalAtmosphere>().density = 50.0; // Normal density
        app.update();
        let normal_temp = app.world().resource::<GlobalTemperature>().current;

        app.world_mut().resource_mut::<GlobalAtmosphere>().density = 20.0; // Thin density
        app.update();
        let thin_temp = app.world().resource::<GlobalTemperature>().current;

        assert!(thin_temp < normal_temp, "Thin atmosphere should decrease global temperature");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct AtmosphericSiphon {
    pub extraction_rate: f32,
    pub accumulated_gas: f32,
}

#[derive(Resource)]
pub struct GlobalAtmosphere {
    pub density: f32,
}

impl Default for GlobalAtmosphere {
    fn default() -> Self {
        Self { density: 100.0 }
    }
}

#[derive(Resource)]
pub struct GlobalTemperature {
    pub current: f32,
}

impl Default for GlobalTemperature {
    fn default() -> Self {
        Self { current: 20.0 }
    }
}

pub fn siphon_atmospheric_gas_system(
    mut query: Query<&mut AtmosphericSiphon>,
    mut atmosphere: ResMut<GlobalAtmosphere>,
) {
    for mut siphon in query.iter_mut() {
        siphon.accumulated_gas += siphon.extraction_rate;
        atmosphere.density -= siphon.extraction_rate * 0.1; // Siphoning decreases density slowly
    }
}

pub fn update_temperature_from_atmosphere_system(
    atmosphere: Res<GlobalAtmosphere>,
    mut temperature: ResMut<GlobalTemperature>,
) {
    // Arbitrary temperature mapping based on atmospheric density
    temperature.current = (atmosphere.density / 100.0) * 20.0 + (100.0 - atmosphere.density) * -0.5;
}
```

## 5. REFACTOR Phase: Quality & Design
- Tweak the mapping of atmospheric density to temperature to reflect realistic insulation loss.
- Expose the extracted gas as an `ItemType::CondensedAtmosphere` for the `ColonyInventory` when `accumulated_gas` crosses a threshold.
- Introduce `RadiationExposure` consequences for Pops outdoors when density drops below a critical threshold.

## 6. Acceptance Criteria
- [ ] Atmospheric siphons accumulate gas over time.
- [ ] Siphoning reduces the `GlobalAtmosphere.density`.
- [ ] Reduced atmospheric density lowers `GlobalTemperature.current`.
- [ ] Test coverage >= 85%.
- [ ] `cargo clippy -- -D warnings` passes.

## 7. Technical Guidance
- Link the condensed gas directly into the trade system. It should be highly valued by off-world traders but risky to mass-produce.
- When `GlobalAtmosphere.density` falls too low, consider triggering an event for "Critical Atmosphere Thinning" to warn the player.

## 8. Questions
*Builder: add questions here if spec is unclear.*
