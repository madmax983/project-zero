# 1374: Atmospheric Resonance

## 1. Overview
**Layer:** 1

**Fantasy:** The air sings.

**Mechanic:** High wind speeds on specific map geometries create "Howling". The noise causes Stress but can be harnessed by "Resonance Crystals" to generate power.

**Emergence:** You build a canyon city to shelter from the wind. The wind hits the canyon just right and creates a sonic boom that deafens everyone.

**Tension:** Power generation vs. Noise pollution.

## 2. Dependencies
- Base ECS system
- Wind/Atmosphere grid system
- Map/Geometry layout system
- Noise/Acoustic and Power systems

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, (
            calculate_howling_system,
            process_resonance_power_system,
        ));
        app
    }

    #[test]
    fn test_canyon_geometry_creates_howling_noise() {
        let mut app = setup_app();

        // Spawn a canyon tile with high wind
        let tile = app.world_mut().spawn((
            GridPosition { x: 5, y: 5 },
            WindSpeed { value: 20.0 }, // High wind
            Geometry { is_canyon: true },
            AcousticNoise { level: 0.0 }, // Initially zero
        )).id();

        app.update();

        let noise = app.world().get::<AcousticNoise>(tile).unwrap();

        // Noise should increase due to howling
        assert!(noise.level > 0.0);
    }

    #[test]
    fn test_resonance_crystal_generates_power_from_howling() {
        let mut app = setup_app();

        // Spawn a Resonance Crystal in a howling tile
        let tile = app.world_mut().spawn((
            GridPosition { x: 5, y: 5 },
            WindSpeed { value: 20.0 },
            Geometry { is_canyon: true },
            AcousticNoise { level: 50.0 }, // Loud howling
        )).id();

        let crystal = app.world_mut().spawn((
            ResonanceCrystal,
            PowerOutput { current: 0.0 },
            GridPosition { x: 5, y: 5 },
        )).id();

        app.update();

        let power = app.world().get::<PowerOutput>(crystal).unwrap();

        // Power should be generated based on the noise
        assert!(power.current > 0.0);
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
use bevy::prelude::*;

#[derive(Component, PartialEq, Eq, Hash, Clone, Copy)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct WindSpeed {
    pub value: f32,
}

#[derive(Component)]
pub struct Geometry {
    pub is_canyon: bool,
}

#[derive(Component)]
pub struct AcousticNoise {
    pub level: f32,
}

#[derive(Component)]
pub struct ResonanceCrystal;

#[derive(Component)]
pub struct PowerOutput {
    pub current: f32,
}

pub fn calculate_howling_system(
    mut query: Query<(&WindSpeed, &Geometry, &mut AcousticNoise)>,
) {
    for (wind, geometry, mut noise) in query.iter_mut() {
        if geometry.is_canyon && wind.value > 15.0 {
            // Wind speed creates noise in canyons
            noise.level = (wind.value - 15.0) * 10.0;
        }
    }
}

pub fn process_resonance_power_system(
    mut crystals: Query<(&mut PowerOutput, &GridPosition), With<ResonanceCrystal>>,
    noise_tiles: Query<(&AcousticNoise, &GridPosition)>,
) {
    for (mut power, crystal_pos) in crystals.iter_mut() {
        for (noise, tile_pos) in noise_tiles.iter() {
            if crystal_pos == tile_pos {
                // Generate power based on noise level
                power.current = noise.level * 0.5;
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- Calculate `is_canyon` dynamically based on adjacent terrain height differences rather than a static boolean.
- Distribute noise to adjacent tiles to create an area-of-effect stress impact on pops.
- Connect the generated power to the global `PowerGrid`.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code
- [ ] Tiles with `Geometry.is_canyon` and high `WindSpeed` generate `AcousticNoise`.
- [ ] `ResonanceCrystal` entities on tiles with `AcousticNoise` generate `PowerOutput`.

## 7. Technical Guidance
- The calculation of dynamic canyons should run sparingly (e.g. only when terrain is modified) to save performance.

## 8. Questions
*Builder: add questions here if spec is unclear.*
