# Specification: Sub-Surface Geodes

## 1. Overview
**Layer:** 1
**Fantasy:** Finding a massive, hollow crystal deep underground and moving into it.
**Mechanic:** Rare, massive hollow geode structures spawn deep underground. The inside is perfectly insulated, pre-lit, and beautiful, but the crystal walls are fragile and impossible to replace once broken.
**Emergence:** You build your elite science team's quarters inside a geode. During a minor tremor, a hauler drops a heavy crate, shattering a wall section and exposing the pristine interior to toxic deep-cave spores.
**Tension:** Unmatched natural beauty and insulation vs. extreme fragility of the habitat.

## 2. Dependencies
- Map generation system (`src/layer1/map.rs`)
- Terrain system (`src/layer1/terrain.rs`)
- Event/Damage system (`src/layer1/events.rs` or `damage.rs`)

## 3. RED Phase: Tests First

```rust
#[test]
fn test_geode_generation() {
    // Arrange: Generate a deep underground map chunk
    // Act: Check for Geode presence
    // Assert: Geode exists, walls are marked as Fragile, interior is hollow and has HighBeauty
}

#[test]
fn test_geode_wall_fragility() {
    // Arrange: spawn Geode Wall tile
    // Act: Apply minor damage (e.g., from tremor or dropped crate)
    // Assert: Geode Wall is instantly destroyed, replaced by normal cavern floor
}

#[test]
fn test_geode_insulation() {
    // Arrange: spawn Geode interior tile, set extreme outside temperature
    // Act: run temperature diffusion step
    // Assert: Geode interior temperature remains stable
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// Minimal implementation
// 1. Add `GeodeWall` to TerrainType
// 2. Add `Fragile` component to GeodeWall entities
// 3. Update map generation to sometimes spawn hollow clusters of GeodeWalls
// 4. Update damage system: if entity has `Fragile`, any damage > 0 destroys it
// 5. Update temperature diffusion to treat GeodeWall as a perfect insulator
```

## 5. REFACTOR Phase: Quality & Design
- Create a robust `Fragile` component that can be reused for other delicate structures (e.g., glass windows, ancient relics).
- Ensure map generation logic for geodes doesn't overlap or break existing cavern generation.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Geodes spawn during map generation with hollow interiors
- [ ] Geode walls are destroyed by minimal damage
- [ ] Geode interiors insulate against external temperature

## 7. Technical Guidance
- The map generator should carve the geode using a cellular automata or similar blob-generation method, then hollow it out.
- Ensure the `HighBeauty` modifier is correctly applied to the interior tiles.

## 8. Questions
*Builder: add questions here if spec is unclear.*
