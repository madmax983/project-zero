# 697: Desire Paths

## 1. Overview
**Layer:** 1
**Fantasy:** The colony is shaped by its inhabitants, not just the architect. A mud path worn through the grass tells a story of daily life.
**Mechanic:** Walking on natural terrain (Grass) slowly converts it to "Dirt" or "Path". Paths have higher walk speed than Grass but lower Beauty. Paved roads prevent wear but cost resources.
**Emergence:** Players stop building grid-roads and start paving over the "natural" routes their pops take, creating organic city layouts.
**Tension:** Aesthetics (green grass) vs. Efficiency (speed).

## 2. Dependencies
- `002` Terrain Grid
- `004` Pop Entity (Movement)

## 3. RED Phase: Tests First
```rust
// Define the tests that will drive implementation
// These should FAIL initially

#[test]
fn test_walking_on_grass_increases_wear() {
    // Arrange: A Pop on a Grass tile
    // Act: Process movement system
    // Assert: The tile's `wear` value increases
}

#[test]
fn test_high_wear_converts_grass_to_dirt_path() {
    // Arrange: A Grass tile with wear value near threshold
    // Act: Increase wear past threshold
    // Assert: Tile type changes to Dirt Path
}

#[test]
fn test_dirt_path_has_higher_walk_speed_lower_beauty() {
    // Arrange: A Grass tile and a Dirt Path tile
    // Act: Retrieve their properties
    // Assert: Dirt Path speed > Grass speed; Dirt Path beauty < Grass beauty
}

#[test]
fn test_paved_road_prevents_wear() {
    // Arrange: A Pop on a Paved Road tile
    // Act: Process movement system
    // Assert: The tile's `wear` value does NOT increase
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// The SIMPLEST code that makes tests pass
// No optimization, no extras
// Just enough to turn RED -> GREEN
// e.g.
// struct TerrainWear(f32);
// fn process_desire_paths_system(...) { ... }
```

## 5. REFACTOR Phase: Quality & Design
- Review `UtilityWeights` for pop trait/needs scaling.
- Ensure event broadcasts don't cause performance issues when scaling.
- Eliminate duplicate spatial queries where possible.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage >=85% for new code
- [ ] Specific feature behavior verified

## 7. Technical Guidance
- Implement custom event/component in `layer1` as per standard architectural practices.
- Consider utilizing existing ECS query filters instead of creating new marker structs unnecessarily.
- Adhere strictly to RED-GREEN-REFACTOR.

## 8. Questions
*Builder: add questions here if spec is unclear.*
