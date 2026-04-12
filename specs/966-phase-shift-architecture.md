# 966: Phase-Shift Architecture

## 1. Overview
A "Phase Shifter" building allows pops to enter a "Shadow Layer" of the map. You can build there (doubling space), but "Shadow" entities slowly drain Sanity from any Pop present. Hiding your ugly industry in this "closet of reality" is great for space efficiency, but the engineers who maintain them come back changed.

## 2. Dependencies
- `002` Terrain Grid (Multiple Layers/Z-Levels)
- `010` Pops
- `050` Mental Health/Sanity
- `090` Building Placement

## 3. RED Phase: Tests First
```rust
#[test]
fn test_phase_shifter_allows_shadow_layer_access() {
    // Arrange: A Pop near a `PhaseShifter` building.
    let mut app = App::new();

    // Act: The Pop uses the building to change layers.
    app.update();

    // Assert: The Pop's coordinates now reflect they are on the "Shadow Layer".
}

#[test]
fn test_shadow_layer_sanity_drain() {
    // Arrange: A Pop placed in the "Shadow Layer".
    let mut app = App::new();

    // Act: Advance simulation time.
    app.update();

    // Assert: The Pop's `Sanity` or mental health metric is significantly reduced over time.
}

#[test]
fn test_building_in_shadow_layer() {
    // Arrange: An engineer Pop with building materials in the "Shadow Layer".
    let mut app = App::new();

    // Act: Process work execution to build a structure.
    app.update();

    // Assert: The structure is successfully built and only exists on the "Shadow Layer", not the normal layer.
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// Add a `ShadowLayer` Z-level or dimension flag to the `TerrainGrid` to effectively double the map size.
// Add a `PhaseShifter` building component that allows Pops with tasks in the Shadow Layer to transition their Z-level/layer.
// In `mental_health_system`, check if a Pop's current layer is `ShadowLayer`. If so, apply a continuous negative modifier to their Sanity need.
// Allow construction system queries to target and place `Blueprint` entities specifically on the `ShadowLayer`.
```

## 5. REFACTOR Phase: Quality & Design
- Create visual/UI indicators (e.g., screen tint, whispers in audio) when the camera views the Shadow Layer.
- Ensure pathfinding cleanly handles cross-layer travel only via Phase Shifter nodes to prevent Pops from getting stuck or magically teleporting.
- Add "Shadow-Touched" memory or trait for Pops whose sanity breaks while in the Shadow Layer to trigger specific behavioral emergencies.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for the new module.
- [ ] Pops can move to the Shadow Layer using a Phase Shifter.
- [ ] Buildings can be placed and function in the Shadow Layer.
- [ ] Pops in the Shadow Layer suffer continuous Sanity drain.

## 7. Technical Guidance
- The `TerrainGrid` in `src/layer1/map.rs` (or equivalent) will need to support distinct map planes or an explicit Shadow Z-level.
- Integrate the sanity drain into `src/layer1/needs.rs` or `src/layer1/social/mental_health.rs`.

## 8. Questions
*Builder: add questions here if spec is unclear.*
