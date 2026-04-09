# Rogue Planets

## 1. Overview
A world without a sun, drifting through the dark. A treasure chest frozen in time. Temporary System Nodes that appear and drift across the map before vanishing. They have 0 Solar Power and extreme Cold, but contain massive deposits of rare resources (having "vacuumed" them up over eons). You scramble a mining fleet to intercept the Rogue Planet. You have 6 months to strip-mine it before it drifts out of shuttle range. Your miners work in pitch blackness, powered by nuclear heaters. High risk/reward rush vs. Safe, steady planetary mining.

## 2. Dependencies
- Layer 2 System Map components
- Fleet movement systems
- Resource extraction systems

## 3. RED Phase: Tests First
```rust
#[test]
fn test_rogue_planet_drift_and_despawn() {
    let mut app = bevy::app::App::new();
    app.insert_resource(bevy::time::Time::default());

    // Arrange: Spawn a Rogue Planet node with a set duration
    // Act: Advance time beyond the drift duration
    // Assert: Verify the Rogue Planet entity is despawned (drifts away)
}

#[test]
fn test_rogue_planet_solar_power_and_resources() {
    let mut app = bevy::app::App::new();

    // Arrange: Spawn a Rogue Planet
    // Act: Query its components
    // Assert: Verify it has 0 Solar Power, extreme Cold, and high rare resources
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// Minimal implementation for RoguePlanet component and drifting/despawn system
```

## 5. REFACTOR Phase: Quality & Design
- Extract drift duration logic to a configurable resource.
- Ensure proper cleanup of any fleets attached to a despawning rogue planet.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified

## 7. Technical Guidance
- Implement within `src/layer2/`. Ensure the rogue planet interacts correctly with existing node types.

## 8. Questions
*Builder: add questions here if spec is unclear.*
