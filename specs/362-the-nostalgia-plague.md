# 362 - The Nostalgia Plague

## 1. Overview
**Layer:** Cross-layer (1 -> 3)
**Fantasy:** Dealing with a civilization-wide crisis not of biology, but of collective melancholia and obsession with the "good old days."
**Mechanic:** Pops can contract "Nostalgia" from viewing ancient relics, listening to old chronologies, or interacting with pops from older, core worlds. Infected pops suffer severe productivity drops in modern tasks, but gain immense bonuses when doing primitive jobs (like manual farming or low-tech crafting) and will actively try to dismantle advanced tech.

## 2. Dependencies
- Tech Level / Tier system (`011-tech-tree-backend`, `037-tech-tree-ui`)
- Pop Needs & Traits (`005-pop-needs`, `084-pop-traits`)
- Relics / Ruins (`052-archaeological-excavation`)

## 3. RED Phase: Tests First

```rust
// tests/layer1/contagion/nostalgia_plague_tests.rs

#[test]
fn test_exposure_to_ancient_relic_triggers_nostalgia_infection() {
    // Arrange: Create a Pop and an AncientRelic entity.
    // Act: Have the Pop interact with/view the AncientRelic for X ticks.
    // Assert: Pop gains the `NostalgiaInfection` component/trait.
}

#[test]
fn test_nostalgia_infected_pop_suffers_penalty_on_hightech_jobs() {
    // Arrange: Create an infected Pop and assign to a high-tech job (e.g., Fusion Reactor operation).
    // Act: Calculate job efficiency.
    // Assert: Efficiency is severely reduced (e.g., 10% of normal).
}

#[test]
fn test_nostalgia_infected_pop_gains_bonus_on_primitive_jobs() {
    // Arrange: Create an infected Pop and assign to a primitive job (e.g., Manual Farming).
    // Act: Calculate job efficiency.
    // Assert: Efficiency is massively boosted (e.g., 200% of normal).
}

#[test]
fn test_nostalgia_infection_spreads_via_social_interaction() {
    // Arrange: Create an infected Pop and an uninfected Pop in close proximity.
    // Act: Trigger social interaction event.
    // Assert: Uninfected Pop has a chance to gain the `NostalgiaInfection` component.
}

#[test]
fn test_nostalgia_infected_pops_attempt_to_sabotage_hightech() {
    // Arrange: Create an infected Pop near an unattended high-tech building.
    // Act: Run utility AI evaluation.
    // Assert: Pop selects `ActionType::Sabotage` targeting the high-tech building.
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/contagion/nostalgia_plague.rs

use bevy_ecs::prelude::*;
// Implement NostalgiaInfection component, infection triggers, and efficiency modifiers.
```

## 5. REFACTOR Phase: Quality & Design
- Centralize job efficiency modification so that both positive and negative effects of `NostalgiaInfection` are evaluated in the same pass.
- Tie the infection spread to existing `social_system` paths to avoid duplicate distance checks.
- Add an integration layer to allow curing the plague via "reality anchor" broadcasts or modern psychological treatments.

## 6. Acceptance Criteria
- [ ] All RED phase tests pass.
- [ ] Test coverage ≥85% for the new feature code.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Pops with `NostalgiaInfection` show modified job outputs based on the job's tech tier.
- [ ] Contagion properly spreads between interacting Pops.

## 7. Technical Guidance
- Create a `TechTier` enum or field on Jobs/Buildings if it doesn't already exist in a queryable format.
- Use a `Timer` or `Accumulator` to build up the infection risk before applying the actual `NostalgiaInfection` to prevent instant infection upon seeing a relic.
- Sabotage behaviors should plug into existing unrest/vandalism utility AI actions.

## 8. Questions
*Builder: add questions here if spec is unclear.*
