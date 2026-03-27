# 660 - Orbital Mirrors

## 1. Overview
**Layer:** Cross-layer (2 -> 1)
**Fantasy:** Harnessing the power of a star. Playing god with the weather by constructing massive orbital mirrors to focus sunlight on specific parts of your colony.
**Mechanic:** Players can construct and position Orbital Mirrors in Layer 2 to focus concentrated solar energy on Layer 1. This increases light and heat, significantly boosting crop yields or solar power, but can also be used defensively to scorch enemies. Misalignment or solar flares can turn it into an uncontrollable death ray.
**Emergence:** You try to save your frozen crops with a mirror, but a solar flare amplifies the beam and sets the entire farm on fire.
**Tension:** High-yield farming and power generation (risk of catastrophic fire) vs. safe, slow survival.

## 2. Dependencies
- `053` Lighting System (Light levels)
- `140` Thermal Management (Temperature mapping and heat transfer)
- `033` Fire Propagation (Fire starting from excessive heat/light)
- `152` Orbital Stations (Orbital construction)

## 3. RED Phase: Tests First

```rust
// tests/layer2/orbital_mirrors.rs

use bevy::prelude::*;
use scale::layer1::environment::{TemperatureGrid, LightGrid};
use scale::layer2::orbital_mirrors::{OrbitalMirror, MirrorFocusEvent, orbital_mirror_focus_system};
use scale::layer1::nature::fire::Fire;

#[test]
fn test_orbital_mirror_increases_light_and_heat() {
    let mut app = App::new();
    app.add_systems(Update, orbital_mirror_focus_system);

    let mut temp_grid = TemperatureGrid::new(10, 10, 20.0);
    let mut light_grid = LightGrid::new(10, 10, 1.0);

    app.world_mut().insert_resource(temp_grid);
    app.world_mut().insert_resource(light_grid);

    // Spawn an orbital mirror targeting 5,5
    let mirror_id = app.world_mut().spawn(OrbitalMirror {
        target: Vec2::new(5.0, 5.0),
        intensity: 5.0,
        radius: 2.0,
        alignment_error: 0.0,
    }).id();

    app.update();

    let updated_temp = app.world().resource::<TemperatureGrid>();
    let updated_light = app.world().resource::<LightGrid>();

    // Target should be significantly hotter and brighter
    assert!(updated_temp.get_temp(5, 5) > 20.0);
    assert!(updated_light.get_light(5, 5) > 1.0);

    // Edges should be unaffected
    assert_eq!(updated_temp.get_temp(0, 0), 20.0);
}

#[test]
fn test_mirror_misalignment_causes_drift() {
    let mut app = App::new();
    app.add_systems(Update, orbital_mirror_focus_system);

    let mut temp_grid = TemperatureGrid::new(10, 10, 20.0);
    app.world_mut().insert_resource(temp_grid);

    app.world_mut().spawn(OrbitalMirror {
        target: Vec2::new(5.0, 5.0),
        intensity: 10.0,
        radius: 1.0,
        alignment_error: 2.0, // High error
    });

    app.update();

    let updated_temp = app.world().resource::<TemperatureGrid>();
    // The exact 5,5 might not be the hottest point due to drift
    // Test logic would check if the peak heat is within the error radius, not exactly at target
    // ...
}

#[test]
fn test_extreme_mirror_heat_starts_fire() {
    let mut app = App::new();
    app.add_systems(Update, (orbital_mirror_focus_system, scale::layer1::nature::fire::fire_ignition_system));

    let mut temp_grid = TemperatureGrid::new(10, 10, 20.0);
    app.world_mut().insert_resource(temp_grid);

    // Spawn highly intense mirror
    app.world_mut().spawn(OrbitalMirror {
        target: Vec2::new(5.0, 5.0),
        intensity: 500.0, // Flash ignition temperature
        radius: 1.0,
        alignment_error: 0.0,
    });

    app.update();

    // Check if Fire component exists at 5,5
    let mut fire_exists = false;
    for (transform, _fire) in app.world_mut().query::<(&Transform, &Fire)>().iter(app.world()) {
        if transform.translation.x == 5.0 && transform.translation.y == 5.0 {
            fire_exists = true;
            break;
        }
    }
    assert!(fire_exists);
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer2/orbital_mirrors.rs

use bevy::prelude::*;
use rand::Rng;
use crate::layer1::environment::{TemperatureGrid, LightGrid};
use crate::layer1::nature::fire::{Fire, Flammable};

#[derive(Component)]
pub struct OrbitalMirror {
    pub target: Vec2,
    pub intensity: f32,
    pub radius: f32,
    pub alignment_error: f32,
}

#[derive(Event)]
pub struct MirrorFocusEvent {
    pub mirror_id: Entity,
    pub new_target: Vec2,
}

pub fn orbital_mirror_focus_system(
    mut mirrors: Query<&mut OrbitalMirror>,
    mut temp_grid: ResMut<TemperatureGrid>,
    mut light_grid: ResMut<LightGrid>,
    time: Res<Time>,
) {
    let mut rng = rand::thread_rng();

    for mut mirror in mirrors.iter_mut() {
        // Calculate actual focus point with random drift
        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
        let drift = rng.gen_range(0.0..=mirror.alignment_error);
        let actual_focus = mirror.target + Vec2::new(angle.cos() * drift, angle.sin() * drift);

        // Apply heat and light to grid
        let tx = actual_focus.x as usize;
        let ty = actual_focus.y as usize;

        if temp_grid.is_in_bounds(tx, ty) {
            // Apply intense heat scaling with intensity and delta time
            let heat_added = mirror.intensity * time.delta_secs();
            temp_grid.add_temp(tx, ty, heat_added);

            // Minimal falloff for blast radius (simplification)
            let r = mirror.radius as i32;
            for dx in -r..=r {
                for dy in -r..=r {
                    let dist = ((dx*dx + dy*dy) as f32).sqrt();
                    if dist <= mirror.radius {
                        let falloff = 1.0 - (dist / mirror.radius);
                        let ax = (tx as i32 + dx) as usize;
                        let ay = (ty as i32 + dy) as usize;
                        if temp_grid.is_in_bounds(ax, ay) {
                             temp_grid.add_temp(ax, ay, heat_added * falloff * 0.5);
                             light_grid.add_light(ax, ay, mirror.intensity * falloff * 0.1);
                        }
                    }
                }
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design

- **Performance:** Iterating over grid tiles every frame for every mirror could be expensive. Optimize by capping the update rate or using a spatial caching mechanism.
- **Integration:** Link alignment error to tech level, weather events (storms obscuring beams), and maintenance debt on the Layer 2 station.
- **Refactoring:** The heat application should likely decay over time if the mirror is moved, relying on the existing `thermal_management` systems to diffuse the heat naturally.

## 6. Acceptance Criteria
- [ ] All RED phase tests pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `src/layer2/orbital_mirrors.rs`.
- [ ] Mirrors successfully increase light and temperature at their target location.
- [ ] Extreme heat from mirrors correctly interfaces with the fire ignition system.

## 7. Technical Guidance
- **Gotchas:** Be careful with delta time and temperature accumulation. If not balanced, a mirror will instantly vaporize the map. Rely on `TemperatureGrid`'s inherent diffusion to balance the added heat.
- **UI:** A visual beam or "lens flare" effect on Layer 1 is crucial so the player knows *where* the mirror is currently pointing.

## 8. Questions
*Builder: add questions here if spec is unclear.*
