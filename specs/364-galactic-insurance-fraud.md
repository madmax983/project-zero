# 364 - Galactic Insurance Fraud

## 1. Overview
**Layer:** 2 (Interacts with 1 and 3)
**Fantasy:** Balancing the books by intentionally crashing your own ships or sabotaging outposts to collect massive payouts from the core world bureaucracies.
**Mechanic:** The player can purchase expensive insurance policies on fleets and orbital structures. Once insured, if the asset is "accidentally" destroyed by space hazards or pirates (but not clearly scuttled by the player), a massive credit payout is generated. However, "Insurance Investigators" will periodically arrive to audit suspicious destruction.

## 2. Dependencies
- Trade and Credits (`039-trade-system`, `194-company-scrip`)
- Fleet Management & Movement (`099-fleet-movement`, `158-fleet-management`)
- Events / The Inspector (`091-the-inspector`)

## 3. RED Phase: Tests First

```rust
// tests/layer2/economics/insurance_fraud_tests.rs

#[test]
fn test_insured_ship_destruction_yields_payout() {
    // Arrange: Create a Fleet, purchase an InsurancePolicy for it.
    // Act: Have the Fleet destroyed by an external source (e.g., Space Hazard).
    // Assert: ColonyResources credits increase by the policy payout amount.
}

#[test]
fn test_intentional_scuttling_voids_insurance() {
    // Arrange: Create an insured Fleet.
    // Act: Player issues direct "Scuttle/Deconstruct" command.
    // Assert: Fleet is destroyed, but no insurance payout is awarded.
}

#[test]
fn test_multiple_claims_trigger_investigator_arrival() {
    // Arrange: Simulate multiple insured asset destructions within a short timeframe.
    // Act: Process the `InsuranceClaimEvent`s.
    // Assert: An `InsuranceInvestigator` NPC/ship is scheduled to arrive at the colony.
}

#[test]
fn test_failed_investigation_results_in_fines_and_blockade() {
    // Arrange: InsuranceInvestigator arrives. Set evidence of fraud (e.g., player ordered fleet into known unwinnable pirate ambush).
    // Act: Investigator completes audit.
    // Assert: Colony loses credits (fine) and a temporary `TradeBlockade` is applied.
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer2/economics/insurance.rs

use bevy_ecs::prelude::*;
// Implement InsurancePolicy component, claim processing system, and investigator triggers.
```

## 5. REFACTOR Phase: Quality & Design
- Create an `InsurancePolicy` component that can attach to any high-value entity (Ships, Orbital Stations).
- Hook into the entity destruction pipeline. When an entity with an `InsurancePolicy` is destroyed, emit an `InsuranceClaimEvent` instead of resolving immediately.
- A central `insurance_processing_system` should evaluate the cause of death (from a `DamageEvent` or `DestructionCause` tag) to differentiate between "accidents" and "intentional scuttling".
- Reuse `The Inspector` logic for the `InsuranceInvestigator` to avoid duplicating pathfinding and NPC assessment code.

## 6. Acceptance Criteria
- [ ] All RED phase tests pass.
- [ ] Test coverage ≥85% for the insurance module.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Players can purchase policies using UI/Edicts.
- [ ] Payouts are granted only for "accidental" or hostile destruction.
- [ ] Investigators arrive if a hidden "Suspicion" threshold is crossed.

## 7. Technical Guidance
- Add a field `suspicion_score` to a global `InsuranceManager` resource. Increment it slightly for every claim, and significantly if the destroyed asset was obsolete or heavily damaged prior to the event.
- Ensure the `DamageEvent` contains the `source` of the damage, so the system knows if the player shot their own ship vs. a pirate shooting it.

## 8. Questions
*Builder: add questions here if spec is unclear.*
