# Specification: Architectural Sympathy

## 1. Overview
**Layer:** 1
**Fantasy:** The buildings remember the mood of the people who built them.
**Mechanic:** When a building is constructed, it permanently records the average `Mood` of the builders. A building constructed by ecstatic workers gains a permanent "Joyful" aura, slightly boosting the mood of anyone who works or sleeps there. A building built by terrified, starving workers gains a "Despair" aura.
**Emergence:** You force your colonists to build a massive hospital during a famine. The hospital is finished, but it radiates such profound despair that patients refuse to sleep in the beds, preferring to recover outside in the dirt.
**Tension:** Rushing emergency construction with miserable workers vs. waiting for a happy workforce to build permanent, high-quality infrastructure.

## 2. Dependencies
- Building construction system (`src/layer1/building.rs`)
- Pop mood/needs system (`src/layer1/needs.rs`)
- Aura/AoE system (`src/layer1/aura.rs` or `src/layer1/mood.rs`)

## 3. RED Phase: Tests First

```rust
#[test]
fn test_building_records_builder_mood() {
    // Arrange: Spawn a blueprint and a builder pop with extremely low mood.
    // Act: Advance simulation until building is complete.
    // Assert: Completed building entity has a `MoodAura { kind: Despair }` component.
}

#[test]
fn test_building_aura_affects_occupants() {
    // Arrange: Spawn a completed building with a `MoodAura { kind: Joyful }` and a pop assigned to work there.
    // Act: Advance simulation.
    // Assert: Pop's mood increases due to the building's aura.
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// Minimal implementation
// 1. Add `MoodAura` enum component: `Joyful`, `Neutral`, `Despair`.
// 2. In `construction_system`, track the average mood of all pops contributing labor to a `Blueprint`. Store this in a temporary component `BuilderMoodTracker` on the blueprint.
// 3. When blueprint completes and converts to a building, evaluate `BuilderMoodTracker`:
//    - Average mood < 30% -> Add `MoodAura::Despair`
//    - Average mood > 80% -> Add `MoodAura::Joyful`
// 4. Update the `apply_mood_system` (or similar) to query pops inside/working at buildings with `MoodAura` and apply a small periodic buff/debuff.
```

## 5. REFACTOR Phase: Quality & Design
- `BuilderMoodTracker` might just be an accumulator `(total_mood_samples, count)`. Update it every tick a builder works on it.
- Ensure the aura buff/debuff is distinct from the building's base quality so players understand *why* a high-tier hospital is causing despair.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Buildings record the mood of their builders accurately.
- [ ] The resulting `MoodAura` correctly influences pops using the building.

## 7. Technical Guidance
- Consider what happens if multiple builders with vastly different moods work on the same building. An average is good, but maybe extreme despair should drag the average down faster.

## 8. Questions
*Builder: add questions here if spec is unclear.*
