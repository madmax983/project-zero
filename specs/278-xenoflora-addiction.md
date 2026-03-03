# Spec 278: Xenoflora Addiction

## 1. Overview
A native plant provides food and stress relief but creates a dangerous dependency. Extinction causes violent withdrawal.

## 2. Dependencies
- `048` Hostile Fauna (or Flora)
- `127` Stress Breakdowns
- `181` Chemical Regulation

## 3. RED Phase: Tests First

```rust
#[test]
fn test_xenoflora_consumption_reduces_stress() {
    // Arrange: Setup pop with high stress, xenoflora food available
    // Act: Pop consumes xenoflora
    // Assert: Stress is reduced, Pop gains `Addicted` component
}

#[test]
fn test_addicted_pop_withdrawal() {
    // Arrange: Setup addicted pop, no xenoflora available
    // Act: Advance time
    // Assert: Pop enters `Withdrawal` state, stress skyrockets
}

#[test]
fn test_withdrawal_causes_violence() {
    // Arrange: Setup pop in withdrawal
    // Act: Advance time
    // Assert: Pop attacks infrastructure or other pops
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// Minimal `Xenoflora` food item
// `Addicted` component applied on consumption
// `Withdrawal` state when xenoflora is absent, increasing stress
```

## 5. REFACTOR Phase: Quality & Design
- Integrate the `Withdrawal` state into the existing `MentalBreakType` system.
- Ensure the `Addicted` trait persists correctly through save/load.
- The `Xenoflora` should be a distinct item from standard crops to force the dependency logic.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Xenoflora consumption leads to addiction and violent withdrawal when absent.

## 7. Technical Guidance
- Create `src/layer1/needs/addiction.rs`.
- Hook into the `metabolism_system` and `stress_system`.
- The `Withdrawal` state should force a mental break of type `Violent`.

## 8. Questions
*Builder: add questions here if spec is unclear.*
