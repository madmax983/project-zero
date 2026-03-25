# 593: Solar Cycles

## 1. Overview
**Layer:** 2 -> 1
**Fantasy:** The star is not a static lightbulb; it is a volatile nuclear furnace that dictates your survival.
**Mechanic:** The local star cycles between "Solar Maximum" and "Solar Minimum" over years. Max = High Solar Power, High Radiation (sickness/mutation), Comms Interference. Min = Low Temp, Low Solar Power, Clear Comms.

## 2. Dependencies
- 002-terrain-grid.md
- 004-pop-entity.md

## 3. RED Phase: Tests First
```rust
#[test]
fn test_solar_maximum_effects() {
    // Arrange: Solar cycle set to Maximum
    // Act: Process environment tick
    // Assert: Solar power generation increases, radiation levels rise
}

#[test]
fn test_solar_minimum_effects() {
    // Arrange: Solar cycle set to Minimum
    // Act: Process environment tick
    // Assert: Temperatures drop, solar power generation decreases
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// Add SolarCycle resource
// Implement system to transition between Maximum and Minimum over time
// Implement system to apply global effects (power, temperature, radiation)
```

## 5. REFACTOR Phase: Quality & Design
- Abstract the cycle transition into a flexible generic global event system.
- Integrate with Layer 2 systems to ensure consistency across the planetary system.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Solar cycles transition correctly and apply expected global effects

## 7. Technical Guidance
- The SolarCycle resource should be global and influence multiple Layer 1 systems.
- Balance the cycle duration to create long-term strategic planning challenges.

## 8. Questions
*Builder: add questions here if spec is unclear.*
