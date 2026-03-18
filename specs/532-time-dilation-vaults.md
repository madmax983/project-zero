# Specification: Time-Dilation Vaults

## 1. Overview
**Layer:** 1
**Fantasy:** Putting your best people in a box where time stands still until they are needed.
**Mechanic:** Extremely expensive endgame vaults that slow time inside to a crawl. Pops placed inside age 1 day for every year outside, consuming almost no resources.
**Emergence:** You put your legendary founding scientists in the vault to preserve their genius. 100 years later, you wake them up to solve an energy crisis, only to find their knowledge is hopelessly obsolete and the colony speaks a dialect they don't understand.
**Tension:** Perfect preservation of talent vs. the inevitable march of progress rendering that talent obsolete.

## 2. Dependencies
- Needs/Aging system (`src/layer1/needs.rs`)
- Building system (`src/layer1/building.rs`)
- Skill/Tech system (`src/layer1/skills.rs` or `knowledge.rs`)

## 3. RED Phase: Tests First

```rust
#[test]
fn test_time_dilation_vault_preservation() {
    // Arrange: setup Pop in TimeDilationVault, advance simulation by 10 years
    // Act: Check Pop's age and needs
    // Assert: Pop's age increased by a tiny fraction (e.g., 10 days), needs are almost unchanged
}

#[test]
fn test_time_dilation_obsolescence() {
    // Arrange: Pop with maxed "Current Era" tech skill placed in vault. Advance colony tech era.
    // Act: Remove Pop from vault, assign to new era job
    // Assert: Pop's skill is considered "Obsolete," providing a massive efficiency penalty until retrained
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// Minimal implementation
// 1. Add `TimeDilationVault` to BuildingType.
// 2. Add `InStasis` component to Pops inside the vault.
// 3. In the metabolism/aging system: if Pop has `InStasis`, multiply aging and needs decay by 0.001.
// 4. Add `TechEra` tracking to Pops. When a Pop exits stasis, if their `TechEra` is < current colony `TechEra`, apply an `ObsoleteSkills` debuff.
```

## 5. REFACTOR Phase: Quality & Design
- Ensure `InStasis` is a generic component that could also be used for Cryo-Pods or other suspending mechanics, differentiated perhaps by the multiplier or secondary effects (e.g., cryo-sickness vs. time dilation).
- The `ObsoleteSkills` system should hook cleanly into the general skill calculation system without needing special casing in every job.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Pops in the vault barely age or consume resources
- [ ] Pops retrieved after a significant time gap suffer from obsolete skills

## 7. Technical Guidance
- Be careful with floating-point math when applying the 0.001 multiplier to needs; ensure it doesn't underflow or cause zero-division issues elsewhere.
- Consider adding a "Culture Shock" mood debuff upon exiting the vault.

## 8. Questions
*Builder: add questions here if spec is unclear.*
