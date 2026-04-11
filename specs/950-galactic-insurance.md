# 950: Galactic Insurance

## 1. Overview

Scamming the galaxy. Players can pay monthly credits to a Layer 3 Corporate entity to insure their Layer 1 colony against disasters. If a specific disaster (e.g., Fire, Raid, Collapse) is detected, the colony receives a payout. However, premiums rise with claims, and fraud detection exists. If a player intentionally causes a disaster (e.g., manually commanding a pop to start a fire) to collect insurance, they risk triggering an investigation and facing severe penalties.

## 2. Dependencies

- `060` Layer 1 Hazards (Fires, Spills) — `specs/060-layer1-hazards.md`
- `112` Layer 3 Economy & Credits — `specs/112-layer3-economy.md`
- `072` Justice System — `specs/072-justice-system.md`

## 3. RED Phase: Tests First

```rust
#[test]
fn test_insurance_premium_deduction() {
    // Arrange: Create app with Layer 3 Treasury and an active Insurance Policy
    let mut app = App::new();
    // Insert resources, advance time past the monthly billing cycle

    // Act: Process economy tick
    app.update();

    // Assert: Treasury credits are reduced by the premium amount
}

#[test]
fn test_legitimate_claim_payout() {
    // Arrange: App with Insurance Policy and Treasury
    let mut app = App::new();

    // Act: Trigger a natural disaster event (e.g., spontaneous fire) and file claim
    app.update();

    // Assert: Treasury credits increase by the payout amount, and next month's premium increases
}

#[test]
fn test_fraudulent_claim_investigation() {
    // Arrange: App with Insurance Policy
    let mut app = App::new();

    // Act: Player issues a direct command (e.g., `ArsonCommand`) resulting in a disaster, then files a claim
    app.update();

    // Assert: Claim is denied or delayed, an `InvestigationEvent` is spawned, and a heavy fine or reputation penalty is applied
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// The SIMPLEST code that makes tests pass
// Add an `InsurancePolicy` component/resource with `premium` and `payout` fields.
// In the economy system, deduct `premium` every billing cycle.
// Listen for `DisasterEvent`. If an active policy covers it, add `payout` to Treasury and increase `premium`.
// Check if the `DisasterEvent` was caused by a `PlayerCommand` (fraud). If so, deny payout and trigger an `InsuranceInvestigationEvent`.
```

## 5. REFACTOR Phase: Quality & Design

- Ensure the fraud detection isn't trivial to bypass. It should trace causality (e.g., if a player locks the doors to a burning room, that might also count as fraud or negligence).
- Create a distinct `InsuranceCorp` faction in Layer 3 that tracks the player's policy history and reputation.
- UI should clearly display the current premium, covered disasters, and pending investigations.

## 6. Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Specific feature behavior verified: Premiums are paid, legitimate disasters yield payouts, and player-instigated disasters trigger investigations/penalties.

## 7. Technical Guidance

- Implement this as a cross-layer bridge system. It needs to read Layer 1 events (`FireStarted`, `BuildingCollapsed`) and mutate Layer 3 resources (`EmpireCredits`, `FactionRelations`).
- Use Bevy's `EventReader` to monitor for covered events.
- Consider adding an `Intent` or `Source` field to disasters to easily distinguish natural vs. player-caused events for the fraud detection logic.

## 8. Questions

*Builder: add questions here if spec is unclear.*
