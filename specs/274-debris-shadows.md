# Spec 274: Debris Shadows

## 1. Overview
Massive space battles or Kessler syndrome events on Layer 2 create dense "Debris Clouds". These clouds cast semi-permanent, moving "Debris Shadows" on the Layer 1 planet surface. Tiles in the shadow suffer severe penalties to Solar Power and crop growth, but gain a bonus to stealth/detection evasion.

## 2. Dependencies
- `184` Orbital Debris
- `042` Energy System (Solar Power)
- `109` Greenhouses / `008` Farm Building and Food Production
- `053` Lighting System

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debris_shadow_reduces_solar_power() {
        // Arrange: World with a Solar Panel and a Debris Cloud in orbit casting a shadow
        // Act: Run simulation tick
        // Assert: Solar panel power output is severely reduced or zero
    }

    #[test]
    fn test_debris_shadow_moves_over_time() {
        // Arrange: Debris cloud with a specific orbital velocity
        // Act: Advance time by X ticks
        // Assert: Shadow position on Layer 1 grid has shifted
    }

    #[test]
    fn test_debris_shadow_reduces_crop_growth() {
        // Arrange: Farm tile inside a debris shadow
        // Act: Run growth tick
        // Assert: Crop growth rate is penalized compared to a tile outside the shadow
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// Minimal components
#[derive(Component)]
pub struct DebrisShadow {
    pub penalty_modifier: f32, // e.g., 0.1 for 90% reduction
}

// Update solar and crop systems to query for DebrisShadow on their tile/region
```

## 5. REFACTOR Phase: Quality & Design
- **Integration**: Ensure the `DebrisShadow` integrates cleanly with existing environmental light modifiers (like Nighttime or Weather).
- **Performance**: Shadow mapping might be expensive if calculating per-tile every tick. Consider a low-resolution grid for shadows or only updating the shadow map periodically based on orbital velocity.
- **Visualization**: Add a visual overlay or darkening effect on the UI for tiles under a shadow.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85%.
- [ ] Solar panels output less power when in a debris shadow.
- [ ] Crops grow slower when in a debris shadow.
- [ ] The shadow moves dynamically based on the orbital debris.

## 7. Technical Guidance
- Extend the `OrbitalDebris` system to project a `DebrisShadow` onto the Layer 1 map.
- The shadow could be represented as an area/radius or a polygon, mapped to grid coordinates.
- Modify the `light_system` or directly adjust the `Sunlight` resource/component for affected tiles.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
