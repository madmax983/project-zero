# 965: The Vertical Schism

## 1. Overview
The physical structure of your colony naturally breeds a catastrophic class war. In extremely deep or tall colonies, Pops track their "Average Z-Level" over their lifetime. Pops who live primarily in "High Altitude" develop a "Sky-Born" trait, while Pops who live in "Deep Depth" develop a "Core-Born" trait. These two groups develop mutual animosity. Segregating your colony for optimal industrial/residential efficiency will inevitably lead to a devastating social stratification.

## 2. Dependencies
- `002` Terrain Grid
- `084` Pop Traits

## 3. RED Phase: Tests First
```rust
#[test]
fn test_vertical_schism_trait_acquisition() {
    // Arrange: A pop living at high Z-level and a pop living at low Z-level.
    let mut app = App::new();

    // Act: Advance simulation time to allow Z-level tracking to accumulate.
    app.update();

    // Assert: The high Z-level pop gains the "Sky-Born" trait. The low Z-level pop gains the "Core-Born" trait.
}

#[test]
fn test_vertical_schism_animosity() {
    // Arrange: A "Sky-Born" pop and a "Core-Born" pop interact.
    let mut app = App::new();

    // Act: Process a social interaction tick.
    app.update();

    // Assert: The interaction generates negative social relationships, animosity, or brawls.
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// Add a `ZLevelHistory` component to track the rolling average Z-level of a pop.
// Add a system `update_zlevel_history_system` that runs periodically to update this value based on current position.
// Add a system `evaluate_vertical_traits_system` that assigns `SkyBorn` or `CoreBorn` components if the average passes a certain threshold.
// In the social interaction logic (e.g. `src/layer1/social`), add a check: if one pop is `SkyBorn` and the other is `CoreBorn`, apply a large negative modifier to their interaction score.
```

## 5. REFACTOR Phase: Quality & Design
- Ensure tracking average Z-level is performant (e.g., sample once per day instead of every tick).
- Allow the player to see these traits in the UI and clearly understand *why* the pops hate each other.
- Consider adding edicts to mediate the conflict (e.g. "Forced Integration Housing").

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for the new module.
- [ ] Pops living high up acquire `Sky-Born`.
- [ ] Pops living deep down acquire `Core-Born`.
- [ ] Interactions between these two groups are highly negative.

## 7. Technical Guidance
- The `TerrainGrid` in `src/layer1/map.rs` should contain the necessary Z-level information to sample.
- Integrate the animosity into whatever the existing social or relationship system is in `src/layer1/social`.

## 8. Questions
*Builder: add questions here if spec is unclear.*
