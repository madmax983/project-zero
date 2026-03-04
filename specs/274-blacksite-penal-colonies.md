# Spec 274: Blacksite Penal Colonies

## 1. Overview
Allows the colony to host dangerous political prisoners from Layer 3 empires for massive payouts, but introduces risks of prison breaks and radicalization.

## 2. Dependencies
- `215` Safehouse Contracts
- `068` Pop Factions
- `135` Trash Cannon Defense

## 3. RED Phase: Tests First

```rust
#[test]
fn test_blacksite_contract_payout() {
    // Arrange: Setup world with a blacksite contract and a prisoner
    // Act: Advance time to trigger payout
    // Assert: Colony resources (credits) increase significantly
}

#[test]
fn test_prisoner_radicalization() {
    // Arrange: Setup a warden pop near a prisoner pop
    // Act: Advance time
    // Assert: Warden's ideology shifts towards the prisoner's ideology
}

#[test]
fn test_prison_break_event() {
    // Arrange: Setup unstable blacksite
    // Act: Trigger prison break
    // Assert: Prisoners become hostile, gain weapons
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// Minimal implementation of Blacksite component and systems
// `BlacksiteContract` resource to track payouts
// `Prisoner` component with high instability
// Systems to process payouts and handle radicalization chance
```

## 5. REFACTOR Phase: Quality & Design
- Integrate radicalization with the existing `SocialMimicry` system.
- Ensure the `PrisonBreak` event uses the `Major` chronicle importance.
- Balance the payout vs. the security risk.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Blacksites generate high income but risk radicalizing nearby pops.

## 7. Technical Guidance
- Create `src/layer1/social/blacksite.rs`.
- Use the `Aura` system for the radicalization effect on wardens.
- Hook into the economic system for payouts.

## 8. Questions
*Builder: add questions here if spec is unclear.*
*Architect:* See related specifications for design details. MVP implementation should follow standard conventions.
