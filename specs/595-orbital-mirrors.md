# 595: Orbital Mirrors

## 1. Overview
**Layer:** 2 -> 1
**Fantasy:** Harnessing the power of a star. Playing god with the weather.
**Mechanic:** Constructible mirrors in orbit (L2). Can be focused on specific L1 tiles to increase light/heat (boost crops in winter) or burn enemies (orbital laser). Misalignment burns the colony.

## 2. Dependencies
- 152-orbital-stations.md
- 008-building-farm.md

## 3. RED Phase: Tests First
```rust
#[test]
fn test_orbital_mirror_boosts_farm() {
    // Arrange: Orbital mirror focused on farm during winter
    // Act: Advance time
    // Assert: Farm crop growth rate increases due to light/heat
}

#[test]
fn test_misaligned_mirror_starts_fire() {
    // Arrange: Orbital mirror focused on non-farm colony tile
    // Act: Advance time
    // Assert: Tile catches fire due to intense heat
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// Add OrbitalMirror component
// Implement system to target Layer 1 tiles
// Implement system to apply heat/light or fire based on target
```

## 5. REFACTOR Phase: Quality & Design
- Abstract the heat application into the existing temperature system.
- Optimize Layer 2 to Layer 1 coordinate mapping.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Orbital mirrors apply heat/light buffs or start fires correctly

## 7. Technical Guidance
- Integrate with the existing grid environment to modify tile temperatures.
- Targeting UI needs cross-layer interaction capabilities.

## 8. Questions
*Builder: add questions here if spec is unclear.*
