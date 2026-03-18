# Specification: The Propaganda Monolith

## 1. Overview
**Layer:** Cross-layer (1 -> 3)
**Fantasy:** Erasing the sins of the empire by building a statue so big it overwrites reality.
**Mechanic:** A massive Layer 1 megastructure that consumes astronomical amounts of power. When active, it slowly deletes negative "Memories" (like famines, wars, massacres) from all Pops in the system and replaces them with a generic "Golden Age" memory. However, its massive broadcast signature angers neighboring Layer 3 empires who see it as aggressive cultural warfare.
**Emergence:** You brutally crush a worker rebellion and build a Monolith to make them forget it happened. The workers go back to the mines humming imperial anthems. But a neighboring democratic empire is so horrified by the mass brainwashing that they declare a crusade to destroy the Monolith, forcing you into a war you didn't want.
**Tension:** Absolute, instantaneous domestic compliance vs. extreme diplomatic hostility from the rest of the galaxy.

## 2. Dependencies
- Building/Megastructure system (`src/layer1/building.rs`)
- Pop Memory system (`src/layer1/memory.rs` - Spec 036)
- Layer 3 Diplomacy system (`src/layer3/diplomacy.rs`)

## 3. RED Phase: Tests First

```rust
#[test]
fn test_monolith_erases_negative_memories() {
    // Arrange: spawn active Propaganda Monolith and a pop with a negative memory ("Massacre Survivor").
    // Act: advance simulation.
    // Assert: the negative memory is removed, replaced with "Golden Age" memory.
}

#[test]
fn test_monolith_triggers_diplomatic_penalty() {
    // Arrange: active Propaganda Monolith on Layer 1, neighboring Empire on Layer 3.
    // Act: advance simulation.
    // Assert: Neighboring empire's diplomatic relation with player decreases due to "Cultural Warfare".
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// Minimal implementation
// 1. Add `PropagandaMonolith` to BuildingType.
// 2. Add `propaganda_broadcast_system`:
//    - Finds active `PropagandaMonolith`s.
//    - Queries all pops on the planet. If they have negative `Memory` components, remove one and add a `Memory { kind: GoldenAge }`.
// 3. Add `propaganda_diplomatic_fallout_system`:
//    - Every period (e.g., year) the Monolith is active, emit an event.
//    - A layer 3 system catches the event and applies a negative modifier `DiplomaticModifier::CulturalWarfare` to all neighboring non-authoritarian empires.
```

## 5. REFACTOR Phase: Quality & Design
- Make the memory replacement gradual rather than instantaneous so it feels like a process.
- The diplomatic penalty should scale with the power/duration of the broadcast.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Active Monolith successfully replaces negative memories with positive ones.
- [ ] Active Monolith generates measurable diplomatic friction on Layer 3.

## 7. Technical Guidance
- Overwriting memories across potentially thousands of pops could be a performance bottleneck if done all at once. Process this in chunks or batches over multiple ticks.

## 8. Questions
*Builder: add questions here if spec is unclear.*
