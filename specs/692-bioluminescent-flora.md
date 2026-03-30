# Spec 692: Bioluminescent Flora

## 1. Overview
The alien night is not dark; it is alive with strange lights. Specific plants glow at night, providing small radii of light. Light reduces stress and monster spawn rates. Harvesting the plant removes the light. This creates a tension between resource extraction (wood/space) and natural safety (light).

## 2. Dependencies
- Base simulation loop (`SimulationTime` or similar time-tracking)
- `TerrainGrid` and `LightingSystem` (to project light from flora)
- Resource extraction/harvesting system

## 3. RED Phase: Tests First
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_bioluminescent_flora_emits_light_at_night() {
        let mut app = App::new();

        // Setup mock environment, lighting grid, and time
        // Add a BioluminescentFlora component to an entity
        // Set time to "night"
        // Run systems
        // Assert that the grid tile corresponding to the flora's position has a light level > 0
    }

    #[test]
    fn test_bioluminescent_flora_light_removed_on_harvest() {
        let mut app = App::new();
        // Setup flora that emits light
        // Harvest/destroy the flora entity
        // Run systems
        // Assert that the grid tile's light level returns to 0
    }

    #[test]
    fn test_bioluminescent_flora_light_reduces_stress() {
         // Setup a Pop near the BioluminescentFlora
         // Set time to night, verify light is active
         // Verify that the Pop's stress decreases or doesn't increase as much as it would in darkness
    }
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// Minimal component definition
#[derive(Component)]
pub struct BioluminescentFlora {
    pub light_radius: u32,
    pub light_intensity: u32,
}

// Minimal system to update the lighting grid based on flora and time of day
```

## 5. REFACTOR Phase: Quality & Design
- Optimize the lighting update so it only recalculates when flora is added, removed, or when day/night transitions occur.
- Ensure the light from flora correctly interacts with other light sources (e.g., additive blending up to a cap).
- Extract hardcoded radius and intensity values into a configuration resource.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Flora correctly emits light only during "night" hours and stops emitting when destroyed.

## 7. Technical Guidance
- Hook into the existing `LightingSystem` if one exists, otherwise create a minimal `FloraLightingSystem`.
- Ensure that the logic determining "night" is synced with the global `SimulationTime` or day/night cycle system.

## 8. Questions
*Builder: add questions here if spec is unclear.*
