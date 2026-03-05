# Specification 311: The Orbital Ring

## 1. Overview
This feature simulates "The Orbital Ring," a massive Layer 2 orbital structure that provides significant logistics and energy buffs to Layer 1. However, it casts a permanent, slowly moving "Shadow Band" across the planetary surface where temperature drops and solar power fails, causing dynamic localized environmental devastation.

## 2. Dependencies
- Layer 2 structures (assuming orbital infrastructure).
- `LightingSystem` / Day-night cycle (`src/layer1/lighting.rs` or `src/layer1/day_night.rs`).
- Temperature grid (`src/layer1/temperature.rs` or similar).

## 3. RED Phase: Tests First

```rust
// src/layer2/orbital_ring.rs
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, update_shadow_band_system);
        app.insert_resource(OrbitalRing { active: true, rotation_angle: 0.0 });
        app.init_resource::<ShadowBand>();
        app
    }

    #[test]
    fn test_shadow_band_rotates() {
        let mut app = setup_app();
        app.update(); // Tick 1

        let ring = app.world().resource::<OrbitalRing>();
        assert!(ring.rotation_angle > 0.0, "The orbital ring should rotate over time.");
    }

    #[test]
    fn test_shadow_band_affects_lighting() {
        let mut app = setup_app();
        app.update();

        let shadow = app.world().resource::<ShadowBand>();
        assert_eq!(shadow.active, true, "The shadow band should be active when the ring is built.");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer2/orbital_ring.rs
use bevy::prelude::*;

#[derive(Resource)]
pub struct OrbitalRing {
    pub active: bool,
    pub rotation_angle: f32, // Simplified representation of the ring's position
}

#[derive(Resource, Default)]
pub struct ShadowBand {
    pub active: bool,
    pub current_angle: f32,
}

pub fn update_shadow_band_system(
    mut ring: ResMut<OrbitalRing>,
    mut shadow: ResMut<ShadowBand>,
) {
    if ring.active {
        ring.rotation_angle += 0.01; // Arbitrary rotation speed
        shadow.active = true;
        shadow.current_angle = ring.rotation_angle;
    } else {
        shadow.active = false;
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Integration**: The `ShadowBand` must interact with the existing `LightingSystem` to actually dim tiles along its path, effectively negating solar power output on those tiles.
- **Performance**: Avoid recalculating the entire shadow band every tick. Update only the leading and trailing edges of the band as it sweeps across the grid.
- **Lore**: Trigger a chronicle event when the ring is first constructed and when major anomalies happen due to the shadow.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >=85% for `orbital_ring.rs`.
- [ ] `ShadowBand` state accurately tracks the `OrbitalRing`'s activity.

## 7. Technical Guidance
- The angle can be used to determine a linear band across the 2D grid of Layer 1. The band's width can be a constant parameter.
- Make sure that tiles entering the shadow band apply a "cool down" effect to their temperature state.

## 8. Questions
*Builder: add questions here if spec is unclear.*
