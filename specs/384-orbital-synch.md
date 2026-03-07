# 384: Orbital Synch

## 1. Overview
**Layer:** 2 -> 1
**Fantasy:** The movement of the heavens dictates your industry. You are a clockwork gear in a solar system.
**Mechanic:** Certain advanced buildings (Solar Arrays, Tidal Generators, Telescopes) only function when the planet is in a specific orbital arc or facing a specific body.
**Emergence:** Production happens in "Bursts". You scramble to run the smelters during the 3-day "Solar Window" and then hibernate for the long dark.
**Tension:** Steady, inefficient power (Fuel) vs. Burst, efficient power (Orbital).

## 2. Dependencies
- Layer 2 Orbital System (or a simulated planetary position/angle relative to a star).
- Layer 1 Buildings (Energy grid or active state).

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_building_activates_in_synch_window() {
        // Arrange
        let mut app = App::new();
        app.insert_resource(PlanetaryOrbit { angle: 90.0 });
        app.add_systems(Update, update_orbital_synch_system);

        let building_id = app.world_mut().spawn(OrbitalSynch {
            active: false,
            required_angle_start: 80.0,
            required_angle_end: 100.0,
        }).id();

        // Act
        app.update();

        // Assert
        let synch = app.world().get::<OrbitalSynch>(building_id).unwrap();
        assert!(synch.active, "Building should be active within its orbital window");
    }

    #[test]
    fn test_building_deactivates_outside_window() {
        // Arrange
        let mut app = App::new();
        app.insert_resource(PlanetaryOrbit { angle: 150.0 });
        app.add_systems(Update, update_orbital_synch_system);

        let building_id = app.world_mut().spawn(OrbitalSynch {
            active: true,
            required_angle_start: 80.0,
            required_angle_end: 100.0,
        }).id();

        // Act
        app.update();

        // Assert
        let synch = app.world().get::<OrbitalSynch>(building_id).unwrap();
        assert!(!synch.active, "Building should be inactive outside its orbital window");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Resource)]
pub struct PlanetaryOrbit {
    pub angle: f32, // 0 to 360
}

#[derive(Component)]
pub struct OrbitalSynch {
    pub active: bool,
    pub required_angle_start: f32,
    pub required_angle_end: f32,
}

pub fn update_orbital_synch_system(
    orbit: Res<PlanetaryOrbit>,
    mut buildings: Query<&mut OrbitalSynch>,
) {
    for mut building in buildings.iter_mut() {
        // Handle wraparound if necessary (simplified for minimal implementation)
        if orbit.angle >= building.required_angle_start && orbit.angle <= building.required_angle_end {
            building.active = true;
        } else {
            building.active = false;
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Refactoring Opportunities:** Handle 360-degree wrapping (e.g., window from 350 to 10 degrees).
- **Code Smells:** Direct mutation of `active` could cause flip-flopping; consider a buffer or state event to alert systems dependent on this power.
- **Performance:** System queries only `OrbitalSynch` buildings. Consider using a system set that only runs when the orbit actually changes significantly.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Buildings activate/deactivate based on the orbital angle correctly (even across the 360 wrap).

## 7. Technical Guidance
- Integrate with energy production components so that `OrbitalSynch.active == false` sets generation to zero.
- Ensure `PlanetaryOrbit` is advanced by Layer 2's simulation tick.

## 8. Questions
*Builder: Add questions here if spec is unclear.*
