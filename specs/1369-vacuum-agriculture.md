# 1369: Vacuum Agriculture

## 1. Overview
**Layer:** 1

**Fantasy:** Life finds a way, even in the void.

**Mechanic:** Genetically modified "Void-Crops" that grow only in vacuum and feed on radiation. They die if exposed to atmosphere/oxygen. Allows farming on hull exteriors.

**Emergence:** You convert your air-filled greenhouses to vacuum farms to save atmosphere. A meteor cracks the dome, letting air *in*. The oxygen kills your entire harvest.

**Tension:** Farming in safety (indoors) vs. Farming in the wild (hull/vacuum).

## 2. Dependencies
- Base ECS system
- Atmosphere/Pressure grid system
- Farming/Crop growth system
- Radiation system (optional/if existing)

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, process_void_crops_system);
        app
    }

    #[test]
    fn test_void_crop_grows_in_vacuum() {
        let mut app = setup_app();

        // Spawn a void crop in a vacuum tile (0 pressure)
        let crop = app.world_mut().spawn((
            VoidCrop,
            Growth { progress: 0.0, rate: 1.0 },
            TileEnvironment { pressure: 0.0, radiation: 5.0 },
        )).id();

        app.update();

        let growth = app.world().get::<Growth>(crop).unwrap();

        // Growth should progress
        assert!(growth.progress > 0.0);
    }

    #[test]
    fn test_void_crop_dies_in_atmosphere() {
        let mut app = setup_app();

        // Spawn a void crop in an atmosphere tile (> 0 pressure)
        let crop = app.world_mut().spawn((
            VoidCrop,
            Growth { progress: 50.0, rate: 1.0 },
            TileEnvironment { pressure: 1.0, radiation: 5.0 }, // Has pressure
            Health { current: 100.0, max: 100.0 }
        )).id();

        app.update();

        let health = app.world().get::<Health>(crop).unwrap();

        // Crop should take damage
        assert!(health.current < 100.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct VoidCrop;

#[derive(Component)]
pub struct Growth {
    pub progress: f32,
    pub rate: f32,
}

#[derive(Component)]
pub struct TileEnvironment {
    pub pressure: f32,
    pub radiation: f32, // For future scaling of growth rate
}

#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

pub fn process_void_crops_system(
    mut query: Query<(&mut Growth, Option<&mut Health>, &TileEnvironment), With<VoidCrop>>,
) {
    for (mut growth, mut health, env) in query.iter_mut() {
        if env.pressure <= 0.01 { // Effectively vacuum
            // Grow normally, perhaps scaling with radiation
            let growth_multiplier = 1.0 + (env.radiation * 0.1);
            growth.progress += growth.rate * growth_multiplier;
        } else {
            // Take damage from atmosphere/oxygen toxicity
            if let Some(mut h) = health {
                h.current -= 10.0; // Fixed damage per tick
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Hook into the actual `AtmosphereGrid` or `PressureGrid` components instead of a mock `TileEnvironment`.
- Create a distinct visual state or component tag when the crop dies from oxygen toxicity.
- Require specific "Vacuum Planters" or allow designation directly on exterior hull tiles.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code
- [ ] `VoidCrop` entities placed in `pressure <= 0.01` tiles gain `Growth.progress`.
- [ ] `VoidCrop` entities placed in `pressure > 0.01` tiles lose `Health.current`.

## 7. Technical Guidance
- Ensure the damage rate from atmosphere kills the crop quickly, enforcing the "fragility to air" fantasy.
- The `radiation` stat can just be a placeholder if the radiation system isn't fully robust, but tying growth to it reinforces the "feeds on radiation" mechanic.

## 8. Questions
*Builder: add questions here if spec is unclear.*
