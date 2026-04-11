# 187 Adaptive Biology

## Overview

"The planet changes you as much as you change it."

Adaptive Biology introduces a mechanism where Pops slowly mutate based on prolonged exposure to environmental conditions. Instead of just taking damage from extreme heat, cold, or toxicity, Pops accumulate "Exposure". When this exposure crosses a threshold, they gain a permanent **Trait** that adapts them to that environment, often with a tradeoff.

This adds a "Losing is Interesting" layer: your miners in the deep, hot levels become "Heat Adapted" but frail in the cold. Your surface explorers become "Void Touched".

## Dependencies

*   `005` Pop Needs (Health)
*   `084` Pop Traits
*   `140` Thermal Management
*   `063` Atmospheric Simulation

## RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::traits::{Trait, Traits};
    use crate::layer1::map::GridPosition;
    use crate::layer1::temperature::TemperatureGrid;
    use bevy_ecs::prelude::*;

    #[test]
    fn test_exposure_accumulation() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, adaptive_biology_system);

        // Setup Grid with extreme heat
        let mut temp_grid = TemperatureGrid::new(10, 10, 20.0);
        temp_grid.set(5, 5, 50.0); // Very Hot
        app.insert_resource(temp_grid);

        // Setup Pop
        let pop_id = app.world.spawn((
            Pop::default(),
            GridPosition { x: 5, y: 5 },
            EnvironmentalExposure::default(),
            Traits::default(),
        )).id();

        // Act
        app.update();

        // Assert
        let exposure = app.world.get::<EnvironmentalExposure>(pop_id).unwrap();
        assert!(exposure.heat > 0.0, "Pop should accumulate heat exposure");
        assert_eq!(exposure.cold, 0.0, "Pop should not accumulate cold exposure");
    }

    #[test]
    fn test_mutation_trigger() {
        // Arrange
        let mut app = App::new();
        app.add_systems(Update, adaptive_biology_system);
        app.insert_resource(TemperatureGrid::new(10, 10, 20.0));

        // Setup Pop near threshold
        let threshold = ADAPTATION_THRESHOLD;
        let pop_id = app.world.spawn((
            Pop::default(),
            GridPosition { x: 0, y: 0 },
            EnvironmentalExposure {
                heat: threshold + 1.0, // Over threshold
                ..Default::default()
            },
            Traits::default(),
        )).id();

        // Act
        app.update();

        // Assert
        let traits = app.world.get::<Traits>(pop_id).unwrap();
        assert!(traits.0.contains(&Trait::HeatAdapted), "Pop should gain HeatAdapted trait");

        let exposure = app.world.get::<EnvironmentalExposure>(pop_id).unwrap();
        assert!(exposure.heat < threshold, "Exposure should reset or clamp after mutation");
    }

    #[test]
    fn test_opposing_adaptations_cancel() {
         // A pop cannot be both HeatAdapted and ColdAdapted.
         // If they have HeatAdapted and gain Cold exposure, maybe they lose HeatAdapted?
         // Or they just can't gain ColdAdapted?
         // DESIGN CHOICE: They are mutually exclusive. Gaining one might be blocked or require losing the other.
         // For MVP: Mutually exclusive, cannot gain opposing trait.

        let mut app = App::new();
        app.add_systems(Update, adaptive_biology_system);
        app.insert_resource(TemperatureGrid::new(10, 10, -50.0)); // Cold

        let pop_id = app.world.spawn((
            Pop::default(),
            GridPosition { x: 0, y: 0 },
            EnvironmentalExposure {
                cold: ADAPTATION_THRESHOLD + 10.0,
                ..Default::default()
            },
            Traits(HashSet::from([Trait::HeatAdapted])), // Already adapted to heat
        )).id();

        app.update();

        let traits = app.world.get::<Traits>(pop_id).unwrap();
        assert!(traits.0.contains(&Trait::HeatAdapted), "Should retain HeatAdapted");
        assert!(!traits.0.contains(&Trait::ColdAdapted), "Should not gain ColdAdapted while HeatAdapted");
    }

    #[test]
    fn test_trait_effects() {
        // Verify the traits actually do something (e.g. modify stats)
        // This likely goes in `traits.rs` tests, but good to spec here.
        let heat_trait = Traits(HashSet::from([Trait::HeatAdapted]));
        // Assuming we add a function `get_thermal_tolerance(&Traits)`
        // assert!(get_thermal_tolerance(&heat_trait).max_temp > 35.0);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. `EnvironmentalExposure` Component

```rust
#[derive(Component, Default, Debug, Clone)]
pub struct EnvironmentalExposure {
    pub heat: f32,
    pub cold: f32,
    pub toxic: f32,
    // potential for gravity, radiation later
}
```

### 2. New Traits

Update `src/layer1/traits.rs`:

```rust
pub enum Trait {
    // ... existing
    HeatAdapted, // +MaxTemp, -MinTemp (frail in cold)
    ColdAdapted, // -MinTemp, +MaxTemp (overheats easily)
    VoidTouched, // Adapted to low pressure/vacuum (requires Oxygen/Pressure system integration, or just general "toughness")
}
```

### 3. `adaptive_biology_system`

```rust
pub const ADAPTATION_THRESHOLD: f32 = 1000.0; // Ticks of exposure? Or magnitude * ticks?
pub const EXPOSURE_RATE: f32 = 1.0;

pub fn adaptive_biology_system(
    mut pops: Query<(Entity, &GridPosition, &mut EnvironmentalExposure, &mut Traits)>,
    temp_grid: Res<TemperatureGrid>,
    // atmosphere_grid: Res<AtmosphereGrid>,
) {
    for (entity, pos, mut exposure, mut traits) in &mut pops {
        let temp = temp_grid.get(pos.x as usize, pos.y as usize);

        // Heat Exposure
        if temp > 35.0 {
            exposure.heat += EXPOSURE_RATE * (temp - 35.0).min(5.0); // Cap rate
        } else {
            exposure.heat = (exposure.heat - 0.1).max(0.0); // Decay
        }

        // Cold Exposure
        if temp < 10.0 {
            exposure.cold += EXPOSURE_RATE * (10.0 - temp).min(5.0);
        } else {
            exposure.cold = (exposure.cold - 0.1).max(0.0);
        }

        // Trigger Mutation
        if exposure.heat > ADAPTATION_THRESHOLD {
            if !traits.0.contains(&Trait::ColdAdapted) && !traits.0.contains(&Trait::HeatAdapted) {
                 traits.0.insert(Trait::HeatAdapted);
                 exposure.heat = 0.0;
                 // Send Notification Event
            }
        }

        if exposure.cold > ADAPTATION_THRESHOLD {
             if !traits.0.contains(&Trait::HeatAdapted) && !traits.0.contains(&Trait::ColdAdapted) {
                 traits.0.insert(Trait::ColdAdapted);
                 exposure.cold = 0.0;
            }
        }
    }
}
```

### 4. Updates to `thermal_damage_system`

The damage system in `temperature.rs` needs to respect these new traits.

```rust
// in temperature.rs or traits.rs
pub fn get_temp_tolerance_modifiers(traits: &Traits) -> (f32, f32) {
    let mut min_mod = 0.0;
    let mut max_mod = 0.0;

    if traits.0.contains(&Trait::HeatAdapted) {
        max_mod += 20.0; // Can stand 55C
        min_mod += 10.0; // Freezes at 20C (frail)
    }
    if traits.0.contains(&Trait::ColdAdapted) {
        min_mod -= 20.0; // Can stand -10C naked
        max_mod -= 10.0; // Overheats at 25C
    }
    (min_mod, max_mod)
}
```

## REFACTOR Phase

*   **Config Resource:** Move `ADAPTATION_THRESHOLD` and rates to a `BiologyConfig` resource for tweaking.
*   **Event Bus:** Emit a `MutationEvent` so the UI/Chronicle can pick it up ("Miner Bob has mutated!").
*   **Visuals:** Updates to the Pop's description or glyph color in the UI to reflect mutation.
*   **Gene Banks:** Future integration with Spec 165 to store/clone these traits.

## Acceptance Criteria

- [ ] `EnvironmentalExposure` component exists and tracks heat/cold.
- [ ] Pops accumulate exposure when in extreme temperatures.
- [ ] Exposure decays when in safe temperatures.
- [ ] Gaining `HeatAdapted` trait prevents gaining `ColdAdapted`.
- [ ] `HeatAdapted` trait increases max safe temp and increases min safe temp (tradeoff).
- [ ] `ColdAdapted` trait decreases min safe temp and decreases max safe temp.
- [ ] Tests pass verifying accumulation and mutation trigger.

## Questions
- *Builder: Should mutations happen instantly or require a "Sickness" phase?*
  *Architect:* For MVP, mutations happen instantly when the `Exposure` reaches 100.0.
- *Builder: Can traits be removed?*
  *Architect:* No, adaptive traits are permanent once acquired.
- *Builder: Should clothing affect exposure?*
  *Architect:* Yes, but that is out of scope for MVP. Assume base pop bodies for now.
