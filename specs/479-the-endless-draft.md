# 479: The Endless Draft

## 1. Overview
An allied Layer 3 empire enters a "Total War". They periodically issue a "Draft Order", demanding a specific number of Layer 1 Pops with high physical stats. Complying grants massive Diplomatic and Trade currency. Refusing incurs severe embargoes. Occasionally, a "Veteran" Pop returns years later, possessing elite combat skills but severe PTSD (Stress penalties). This forces players to balance immediate economic and diplomatic survival against the systematic, generational draining of their colony's strongest citizens.

## 2. Dependencies
- `067` Militia System (Implemented)
- `469` The Galactic Council (Backlog)

## 3. RED Phase: Tests First
```rust
#[test]
fn test_draft_order_generation() {
    let mut app = App::new();
    // Arrange: Mock Layer 3 Empire in Total War
    // Act: Trigger draft order generation system
    // Assert: Verify draft order demands Pops with high physical stats
}

#[test]
fn test_draft_compliance_rewards() {
    let mut app = App::new();
    // Arrange: Setup colony with required Pops
    // Act: Process draft compliance
    // Assert: Pops removed from Layer 1, Trade and Diplomatic currency increased
}

#[test]
fn test_draft_refusal_penalties() {
    let mut app = App::new();
    // Arrange: Setup colony
    // Act: Process draft refusal
    // Assert: Severe embargo modifier applied
}

#[test]
fn test_veteran_return_event() {
    let mut app = App::new();
    // Arrange: Trigger veteran return
    // Act: Spawn veteran Pop
    // Assert: Pop has elite combat skills and PTSD trait/stress modifier
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// Minimal systems for DraftOrderEvent, Compliance processing, Refusal processing, and VeteranReturnEvent.
```

## 5. REFACTOR Phase: Quality & Design
- Ensure integration with Layer 3 diplomacy and trade systems uses Bevy Events rather than direct mutation to avoid tight coupling.
- Refactor Pop selection logic to efficiently find Pops meeting the physical stat requirements.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for new code.
- [ ] Draft orders demonstrably exchange Pops for currency, and refusal demonstrably triggers embargoes.

## 7. Technical Guidance
- Use `EventWriter<DraftOrderEvent>` to broadcast the demand.
- The `Veteran` trait should hook into the existing stress and morale systems to apply chronic PTSD penalties.
- Ensure the UI clearly shows the cost/benefit of the Draft choice.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
