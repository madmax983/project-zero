# 852 - Volumetric Gas Pockets

## 1. Overview
Gases like CO2 or Methane have density and pool in low-lying areas (like mines or valleys) or rise to ceilings. Pops breathing bad gas suffocate or get sick. Ventilation systems are required to pump the gas out.

## 2. Dependencies
- `src/layer1/map.rs` (AtmosphereGrid / PressureGrid)
- `src/layer1/needs.rs` (Health / Suffocation)

## 3. RED Phase: Tests First

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    fn setup_app() -> App {
        let mut app = App::new();
        // Setup minimal systems
        app
    }

    #[test]
    fn test_heavy_gas_pools_downward() {
        let mut app = setup_app();
        // Arrange: Grid with heavy gas (CO2) in a column
        // Act: Run gas diffusion/gravity system
        // Assert: Gas concentration is highest at the lowest elevation tile
    }

    #[test]
    fn test_pop_suffocates_in_gas_pocket() {
        let mut app = setup_app();
        // Arrange: Pop in a tile with high CO2 concentration
        // Act: Run breathing/health system
        // Assert: Pop's health decreases or suffocation need increases
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

// Need to hook into existing AtmosphereGrid
// Add gas density to diffusion logic

pub fn gas_gravity_system(
    // mut grid: ResMut<AtmosphereGrid>,
) {
    // Shift heavier-than-air gas downward based on tile elevation/depth
}

pub fn pop_breathing_system(
    // pops: Query<(&Position, &mut Health)>,
    // grid: Res<AtmosphereGrid>,
) {
    // If pop position has toxic gas concentration > threshold, apply damage
}
```

## 5. REFACTOR Phase: Quality & Design
- The gas gravity algorithm needs to be performant, likely running as part of the existing diffusion pass.
- Consider adding a `Ventilation` component for buildings to remove gas from specific tiles.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Heavy gas sinks to lower elevations over time.
- [ ] Pops in hazardous gas take damage or suffer debuffs.

## 7. Technical Guidance
- If `AtmosphereGrid` already handles diffusion, add a gravity vector to the diffusion based on the gas type's density relative to "air".
- Ensure the map supports the concept of elevation or "depth" for gas to sink into.

## 8. Questions
*Builder: add questions here if spec is unclear.*
