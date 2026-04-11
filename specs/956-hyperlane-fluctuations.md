# 956: Hyperlane Fluctuations

## 1. Overview
The geography of the galaxy is a breathing, shifting organism, not a static map. Hyperlanes have a "Stability" metric. Over decades, stable lanes may slowly decay into impassable "Dead Zones," while previously unconnected star systems might suddenly form rapid, temporary "Surge Lanes."

## 2. Dependencies
- `201` Layer 2 Map Generation
- `210` Fleet Movement
- `230` Trade Routes

## 3. RED Phase: Tests First
```rust
#[test]
fn test_hyperlane_stability_decay() {
    // Arrange: A star system map with a connected hyperlane.
    let mut app = App::new();

    // Act: Advance time over a long period.
    app.update();

    // Assert: The hyperlane's stability metric decreases.
}

#[test]
fn test_hyperlane_collapse_to_dead_zone() {
    // Arrange: A hyperlane with near-zero stability.
    let mut app = App::new();

    // Act: Advance time to trigger the collapse.
    app.update();

    // Assert: The hyperlane is removed or marked as impassable (Dead Zone).
}

#[test]
fn test_surge_lane_formation() {
    // Arrange: Two unconnected star systems in close proximity.
    let mut app = App::new();

    // Act: Process galaxy geography random events.
    app.update();

    // Assert: A temporary `SurgeLane` connects the two systems.
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// Add `Stability` and `SurgeLane(Duration)` components to the Layer 2 `Hyperlane` entity.
// In `hyperlane_decay_system`, slowly reduce `Stability` over time for standard lanes.
// When `Stability` <= 0, despawn the lane and emit `HyperlaneCollapseEvent`.
// In `surge_lane_formation_system`, periodically pick two random close systems without a lane. Connect them via a new entity with `SurgeLane`.
// Ensure `fleet_movement_system` respects missing lanes or temporary surge lanes.
```

## 5. REFACTOR Phase: Quality & Design
- Route finding (A*) for trade and fleet logic needs to be invalidated and recalculated when lanes collapse or form.
- Consider performance when recalculating routes: use events instead of checking lanes every frame.
- Add visual indicators for decaying lanes and surge lanes.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for the new module.
- [ ] Hyperlanes correctly lose stability and collapse into impassability.
- [ ] Temporary Surge Lanes dynamically form and allow FTL travel.
- [ ] Recalculation handles route disconnections correctly.

## 7. Technical Guidance
- Implement within `src/layer2/geography/hyperlanes.rs`.
- Integration tests must prove that a trade route is successfully severed when its underlying hyperlane collapses.

## 8. Questions
*Builder: add questions here if spec is unclear.*
