# Derelict Stations

## 1. Overview
Ghost towns in orbit. It's cheaper to fix the dead than build the new. Abandoned stations spawn on the System Map. You can "Claim" them. They start with a layout of damaged/ruined buildings. Repairing them is cheaper than building fresh, but they may have "Quirks" (Haunted, Gas Leaks, hidden pests). You move into an old Science Station. You fix the reactor, but you can't figure out why the previous crew welded the airlocks shut from the *outside*. Then you hear the vents rattle. Cheap reclamation (Unknown risks) vs. Expensive new construction (Safe/Clean).

## 2. Dependencies
- Layer 2 System Map
- Layer 1 Base Building / Structures
- Quirk / Hazard system

## 3. RED Phase: Tests First
```rust
#[test]
fn test_derelict_station_claiming() {
    let mut app = bevy::app::App::new();

    // Arrange: Spawn a Derelict Station node
    // Act: Perform Claim action via fleet/diplomacy
    // Assert: Verify the station changes ownership and layout becomes accessible
}

#[test]
fn test_derelict_quirk_activation_on_repair() {
    let mut app = bevy::app::App::new();

    // Arrange: A claimed Derelict Station with a Damaged Reactor and a hidden "Gas Leak" quirk
    // Act: Perform repair action on the reactor
    // Assert: Verify the Gas Leak quirk is revealed/activated upon successful repair
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// Minimal implementation for Derelict Station spawning, Claiming, and Quirk reveals.
```

## 5. REFACTOR Phase: Quality & Design
- Use a component to store hidden quirks that are revealed upon specific triggers.
- Ensure seamless transition from Layer 2 Node to Layer 1 map layout when claimed.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified

## 7. Technical Guidance
- This bridges Layer 2 and Layer 1. Station generation on Layer 2, layout/quirks on Layer 1.

## 8. Questions
*Builder: add questions here if spec is unclear.*
