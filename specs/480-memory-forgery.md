# 480: Memory Forgery

## 1. Overview
Gaslighting your own citizens to maintain absolute peace, rewriting history to erase your mistakes. A high-tech "Mnestic Archiver" building allows you to literally erase "Bad Memories" (famines, deaths, disasters) from Pops and replace them with fabricated "Good Memories" (bountiful harvests, heroic victories). This instantly maxes out their Morale and resets Stress. You erase a massive famine from the colony's memory. A year later, a single unmodified journal entry is found by an archivist. The "Truth" spreads like a virus. Every Pop who had their memory forged suffers a "Reality Collapse" mental break simultaneously, turning your peaceful utopia into an instantaneous, hyper-violent mob. This creates a tension between perfect, instantaneous morale control via deception vs. the catastrophic, colony-ending risk of the truth getting out.

## 2. Dependencies
- `036` Pop Memory (Implemented)
- `174` Memetic Hazards (Implemented)

## 3. RED Phase: Tests First
```rust
#[test]
fn test_mnestic_archiver_erases_bad_memories() {
    let mut app = App::new();
    // Arrange: Spawn a Pop with a bad memory (e.g. Famine)
    // Act: Trigger the Mnestic Archiver action on the Pop
    // Assert: Verify bad memory is removed and replaced with a good fabricated memory
}

#[test]
fn test_mnestic_archiver_boosts_morale() {
    let mut app = App::new();
    // Arrange: Spawn a stressed Pop with low morale
    // Act: Trigger Mnestic Archiver action
    // Assert: Pop's morale is maxed out and stress is 0
}

#[test]
fn test_truth_outbreak_triggers_reality_collapse() {
    let mut app = App::new();
    // Arrange: Setup colony with forged memory Pops and trigger Truth discovery event
    // Act: Process truth virus propagation
    // Assert: Forged Pops immediately transition to 'RealityCollapse' state/Violent Mob behavior
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// Minimal systems for MnesticArchiver actions, memory replacement logic, and TruthOutbreakEvent processing.
```

## 5. REFACTOR Phase: Quality & Design
- Isolate the 'RealityCollapse' mental break logic into a new system that easily integrates with existing `050 Civil Unrest` mechanisms.
- Make the 'Truth' spread similar to `055 Rumor Web` mechanics or existing `174 Memetic Hazards`.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Memory forging demonstrably resets stress and replaces memories, and Truth Outbreak triggers mass unrest for forged Pops.

## 7. Technical Guidance
- Create a `MnesticArchiver` component for the building and a corresponding Action.
- Mark fabricated memories with a hidden `forged: true` flag.
- The Truth Outbreak should query all Pops with `forged: true` memories and trigger the mental break.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
