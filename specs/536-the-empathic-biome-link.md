# Specification: The Empathic Biome Link

## 1. Overview
**Layer:** 1
**Fantasy:** To a true empath, cutting down a 500-year-old tree sounds like a scream.
**Mechanic:** Certain Pops are born with the "Biome Empath" trait. They gain massive mood boosts from being near untouched nature (high Beauty/Purity tiles). However, if any natural tile (Tree, Rock, Geode) on the map is destroyed or mined, they instantly suffer a severe Stress spike, regardless of where they are on the map.
**Emergence:** You need wood to survive the winter. You send loggers into the ancient forest. Your best scientist, a Biome Empath working in a sealed lab on the other side of the colony, suddenly collapses in psychic agony, halting critical research on the heaters you need to survive.
**Tension:** The physical necessity of resource extraction vs. the psychological well-being of your most sensitive, often highly skilled colonists.

## 2. Dependencies
- Traits system (`src/layer1/traits.rs`)
- Terrain destruction/harvesting system (`src/layer1/terrain.rs` or `harvest.rs`)
- Stress/Mood system (`src/layer1/needs.rs`)

## 3. RED Phase: Tests First

```rust
#[test]
fn test_biome_empath_proximity_boost() {
    // Arrange: spawn Pop with `Trait::BiomeEmpath` on a tile with high nature/beauty.
    // Act: advance simulation.
    // Assert: Pop gains a positive mood modifier.
}

#[test]
fn test_biome_empath_destruction_stress() {
    // Arrange: spawn Pop with `Trait::BiomeEmpath`. Destroy a natural entity (Tree) anywhere on the map.
    // Act: trigger destruction event.
    // Assert: Pop receives an immediate `StressSpike` or negative mood modifier.
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// Minimal implementation
// 1. Add `BiomeEmpath` to `Trait` enum/component.
// 2. Add an event `NatureDestroyedEvent` that fires whenever a natural resource node is mined/destroyed.
// 3. Add system `process_empath_nature_link_system`:
//    - Listens for `NatureDestroyedEvent`.
//    - For every event received, query all Pops with `Trait::BiomeEmpath`.
//    - Apply a `MoodModifier::PsychicAgony` (short duration, high penalty) to each.
// 4. Update existing mood system to give `BiomeEmpath`s a bonus based on the cell's `Beauty` score.
```

## 5. REFACTOR Phase: Quality & Design
- Ensure `NatureDestroyedEvent` is generic enough to be used for other purposes (e.g., spawning hostile fauna, tracking environmental degradation).
- The proximity boost should probably only evaluate periodically, not every tick, for performance.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Empaths get mood boosts from nature.
- [ ] Empaths suffer map-wide stress when nature is destroyed.

## 7. Technical Guidance
- Be careful with mass-destruction events (e.g., a meteor strike destroying 50 trees). You don't want to apply 50 separate stress spikes in one frame and instantly kill the empath. Cap the stress or use a debuff that stacks duration, not intensity.

## 8. Questions
*Builder: add questions here if spec is unclear.*
