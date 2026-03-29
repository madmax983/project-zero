# 755 - The Ecological Rubberband

## 1. Overview
**Layer:** 2 (System Map to Layer 1 Planet)
**Fantasy:** Terraformers pushing a planet's climate too fast, causing it to aggressively snap back.
**Mechanic:** When terraforming a planet to change its biome (e.g., Ice to Arid), doing it rapidly builds "Ecological Tension." If the terraforming machines lose power or are destroyed before the tension dissipates, the planet doesn't just revert to its original state—it violently swings in the opposite extreme, triggering massive planetary disasters (hyper-storms, flash-freezes) that wipe out Layer 1 colonies.

## 2. Dependencies
- `063-atmospheric-simulation.md` (Weather and atmosphere)
- `109-greenhouses.md` (Terraforming infrastructure basics)
- `079-weather-events.md` (Storms)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer2::planet::PlanetBiome;
    use crate::layer1::weather::WeatherState;
    use bevy::prelude::*;

    #[test]
    fn test_ecological_snapback() {
        let mut app = App::new();
        app.add_plugins(EcologicalTensionPlugin);

        let terraformer_id = app.world.spawn((
            Terraformer { powered: false }, // Oh no, power went out
        )).id();

        let planet_id = app.world.spawn((
            PlanetBiome::Ice,
            EcologicalTension { level: 90.0 }, // Dangerously high
        )).id();

        app.update();

        let planet = app.world.get_entity(planet_id).unwrap();
        let weather = planet.get::<WeatherState>().unwrap();

        assert!(weather.is_extreme_event());
        // Tension resets as the energy violently releases
        assert_eq!(planet.get::<EcologicalTension>().unwrap().level, 0.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;
use crate::layer2::planet::PlanetBiome;
use crate::layer1::weather::WeatherState;

pub struct EcologicalTensionPlugin;

impl Plugin for EcologicalTensionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, process_ecological_tension);
    }
}

#[derive(Component)]
pub struct Terraformer {
    pub powered: bool,
}

#[derive(Component)]
pub struct EcologicalTension {
    pub level: f32,
}

pub fn process_ecological_tension(
    mut commands: Commands,
    terraformer_query: Query<&Terraformer>,
    mut planet_query: Query<(Entity, &mut EcologicalTension, &mut WeatherState), With<PlanetBiome>>,
) {
    let mut any_powered = false;
    for terraformer in terraformer_query.iter() {
        if terraformer.powered {
            any_powered = true;
        }
    }

    // If power failed and tension is high
    if !any_powered {
        for (entity, mut tension, mut weather) in planet_query.iter_mut() {
            if tension.level >= 50.0 {
                // Snapback
                weather.set_extreme_event(true);
                tension.level = 0.0;

                // (In reality, this would spawn a flash-freeze or hyper-storm event component)
                commands.entity(entity).insert(EcologicalSnapbackEvent);
            }
        }
    }
}

#[derive(Component)]
pub struct EcologicalSnapbackEvent;
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring Opportunities**:
    - Tie into the Chronicle generator: "We melted the ice too fast. The power flickered, and the planet froze us in a single hour."
    - Scale the `EcologicalSnapbackEvent` severity by the exact `EcologicalTension` level when power failed.
    - Provide a UI dashboard warning on the Terraformer building itself ("Tension: Critical. Do not disable power.").

## 6. Acceptance Criteria (Testable!)
- [ ] `test_ecological_snapback` passes.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.

## 7. Technical Guidance
- **Gotchas**: If there are multiple terraformers, the "power failure" condition needs to evaluate the global terraforming force against the planetary tension. If *overall* terraforming power drops severely, tension snaps.

## 8. Questions
*Builder: add questions here if spec is unclear.*
