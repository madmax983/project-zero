# 853 - Thermal Bloom

## 1. Overview
Layer 1 industrial heat generation creates a "Thermal Signature" visible on Layer 2. A high signature attracts hostile space fauna or pirates. Players must balance industrial output with stealth.

## 2. Dependencies
- `src/layer1/infrastructure/` (Industrial buildings)
- `src/layer2/` (Fleet / Pirate mechanics)
- `src/layer1/map.rs` (PressureGrid/Temperature)

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
    fn test_industry_increases_thermal_bloom() {
        let mut app = setup_app();
        // Arrange: Planet with high industrial activity (heat)
        // Act: Run thermal bloom aggregation system
        // Assert: Layer 2 ThermalSignature component increases
    }

    #[test]
    fn test_high_thermal_bloom_spawns_hostiles() {
        let mut app = setup_app();
        // Arrange: Layer 2 node with massive ThermalSignature
        // Act: Run hostile attraction system
        // Assert: Pirate/Fauna fleet spawned targeting the node
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct ThermalSignature {
    pub value: f32,
}

pub fn aggregate_thermal_bloom_system(
    // Layer 1 heat sources
    // mut layer_2_nodes: Query<&mut ThermalSignature>,
) {
    // Sum total heat from Layer 1 and update ThermalSignature on the corresponding Layer 2 node
}

pub fn thermal_attraction_system(
    nodes: Query<&ThermalSignature>,
    // mut commands: Commands, to spawn hostiles
) {
    // If ThermalSignature > threshold, spawn threat (chance based on bloom size)
}
```

## 5. REFACTOR Phase: Quality & Design
- Create an integration bridge between Layer 1 temperature and Layer 2 signature.
- Introduce buildings (e.g., `HeatSink`) that can lower the thermal bloom at the cost of power or water.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Thermal bloom aggregates accurately from colony heat.
- [ ] High bloom triggers hostile encounters.

## 7. Technical Guidance
- The aggregation system should probably run on a slower timer, not every frame.
- Ensure cross-layer communication is clean (perhaps via an Event or a shared Resource representing the planet's status).

## 8. Questions
*Builder: add questions here if spec is unclear.*
