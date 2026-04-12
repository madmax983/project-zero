# 969: Predatory Weather

## 1. Overview
The planet is hunting you. Storms on the planetary map aren't random; they pathfind towards high Energy or Heat concentrations. This creates a deadly feedback loop: if you turn on the planetary shield or run massive power generators (creating a massive energy spike), a "Great Eye" or similar storm may immediately U-turn and smash into your capital. The colony must balance power usage and heat generation against weather aggro.

## 2. Dependencies
- `115` Power Grid
- `116` Heat System
- `125` Planetary Weather Map

## 3. RED Phase: Tests First
```rust
#[test]
fn test_storm_pathfinding_towards_energy_spike() {
    // Arrange: Set up a Layer 2 weather grid with a Storm entity and a Layer 1 colony tile generating massive energy.
    let mut app = App::new();

    // Act: Advance simulation time to process weather movement.
    app.update();

    // Assert: The Storm's movement vector shifts towards the colony tile rather than following random or global wind patterns.
}

#[test]
fn test_storm_ignores_low_energy() {
    // Arrange: Set up a storm and a colony running on stealth/low power mode.
    let mut app = App::new();

    // Act: Advance simulation time.
    app.update();

    // Assert: The Storm continues on its natural path without homing in on the colony.
}

#[test]
fn test_storm_impact_on_colony() {
    // Arrange: A storm positioned directly over the colony tile.
    let mut app = App::new();

    // Act: Process the storm impact event.
    app.update();

    // Assert: The colony infrastructure takes damage, and outdoor activities are halted.
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// Add an `AggroTarget` component or query to calculate the highest `EnergyEmission` or `HeatSignature` on the map.
// Modify the `weather_movement_system` (Layer 2) to factor in an attraction vector towards the `AggroTarget` if its emission exceeds a threshold.
// Integrate storm impact: if a storm reaches the colony tile, emit a `StormImpactEvent` that damages Layer 1 structures.
```

## 5. REFACTOR Phase: Quality & Design
- Optimize the emission calculations so we don't scan every building every tick. Use chunking or aggregate tracking for the colony's total signature.
- Provide UI feedback: players should see a heat/energy map layer and a warning if they are drawing the attention of nearby storms.
- Balance the attraction weights so storms aren't trivially easy to dodge but also aren't heat-seeking missiles that ignore all terrain.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for the new module.
- [ ] Storms actively alter their course towards high energy/heat sources.
- [ ] Low energy operations avoid storm attraction.
- [ ] Storms deal appropriate damage upon reaching the colony.

## 7. Technical Guidance
- The Layer 2 planetary map will need a way to query Layer 1 aggregate data. Ensure this data is synced in `src/layer1/core/integration.rs` or via a shared resource `ColonyEmissionSignature`.
- Weather movement logic likely resides in `src/layer2/weather/mod.rs` or similar.

## 8. Questions
*Builder: Add questions here regarding the threshold for emission detection or the speed modifier of the storms when homing.*
