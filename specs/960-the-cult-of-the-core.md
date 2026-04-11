# 960: The Cult of the Core

## 1. Overview
Pops assigned to deep crust mining (Z-levels near the mantle) for extended periods accumulate "Core Reverence." They begin to view the planetary heat as a divine presence. They work faster in extreme heat but will actively sabotage cooling systems, vents, and air conditioning units, seeking to "bring the warmth to the surface."

## 2. Dependencies
- `010` Pops
- `023` Environment Heat
- `114` Cults/Factions

## 3. RED Phase: Tests First
```rust
#[test]
fn test_core_reverence_accumulation() {
    // Arrange: A miner Pop working in a high-heat deep crust zone.
    let mut app = App::new();

    // Act: Advance time.
    app.update();

    // Assert: The Pop gains the `CoreReverence` component or its value increases.
}

#[test]
fn test_core_cultist_efficiency_in_heat() {
    // Arrange: A Pop with `CoreReverence` working in a high heat zone.
    let mut app = App::new();

    // Act: Process work tick.
    app.update();

    // Assert: Work speed is multiplied compared to a normal worker.
}

#[test]
fn test_core_cultist_sabotage_cooling() {
    // Arrange: A Pop with `CoreReverence` near a cooling vent.
    let mut app = App::new();

    // Act: Process utility AI.
    app.update();

    // Assert: The Pop performs a `Sabotage(CoolingSystem)` action.
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// In `environment_exposure_system`, if a Pop is in `ZLevel::Mantle` and `Heat > Threshold`, increment a `CoreReverence` counter.
// At a certain threshold, add `CultOfTheCore` component.
// In `work_execution_system`, multiply work progress by 1.5 if `CultOfTheCore` is present and local heat is high.
// In `utility_ai.rs`, give `CultOfTheCore` Pops a high score for the `SabotageBuilding` action if the target has a `Cooling` component.
```

## 5. REFACTOR Phase: Quality & Design
- Use `evaluate_actions_system` to handle sabotage seamlessly, making it a valid choice over normal work if their reverence is high enough.
- Add a visual warning or UI notification when cooling systems mysteriously fail, to hint at the cult.
- Emit a `CoreCultSabotageEvent` when a critical infrastructure piece (like a reactor cooler) is destroyed.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for the new module.
- [ ] Pops gain reverence from deep mining.
- [ ] Cultists work faster in heat.
- [ ] Cultists actively sabotage cooling equipment.

## 7. Technical Guidance
- Create `src/layer1/social/core_cult.rs` for reverence growth.
- Use the existing AI scoring system (`UtilityAI`) for the sabotage behavior to blend it organically into their daily routine.

## 8. Questions
*Builder: add questions here if spec is unclear.*
