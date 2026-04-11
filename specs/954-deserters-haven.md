# 954: The Deserter's Haven

## 1. Overview

Your colony becomes a refuge for those fleeing the wars of giants, turning your home into a powder keg. When a massive war breaks out between two major Layer 3 empires, your neutral, backwater Layer 1 colony starts receiving hidden, untracked shuttles. These contain highly trained "Deserters" from both sides. They offer incredible skills (master engineers, elite soldiers) and demand no pay, only secrecy. You welcome the deserters, drastically boosting your colony's tech and defense capabilities. However, you inadvertently mix veterans from opposing sides in your residential blocks. A tavern brawl between two rival deserters escalates, and suddenly your colony is fighting a miniature proxy war in the streets, using the advanced weapons they smuggled in, threatening to expose your haven to the empires they fled.

## 2. Dependencies

- `010` Pop System
- `114` Faction Wars
- `032` Social Combat

## 3. RED Phase: Tests First

```rust
#[test]
fn test_deserter_arrival_during_war() {
    // Arrange: A neutral colony, and two Layer 3 factions engaged in a war.
    let mut app = App::new();

    // Act: Advance simulation time.
    app.update();

    // Assert: "Deserter" pops from both factions arrive at the colony.
}

#[test]
fn test_deserters_have_advanced_skills() {
    // Arrange: A newly arrived Deserter pop.
    let mut app = App::new();

    // Act: Inspect the pop's stats.
    app.update();

    // Assert: The pop has high-tier skills (e.g., Engineering, Combat) without requiring standard training.
}

#[test]
fn test_deserter_conflict_escalation() {
    // Arrange: A colony containing Deserters from opposing factions in close proximity (e.g., same residential block or tavern).
    let mut app = App::new();

    // Act: Process social interactions over time.
    app.update();

    // Assert: An escalating conflict event occurs (e.g., TavernBrawl -> ProxyWar), dealing damage or creating unrest.
}

#[test]
fn test_haven_exposure_risk() {
    // Arrange: An escalating conflict between Deserters.
    let mut app = App::new();

    // Act: The conflict reaches a critical threshold (e.g., use of smuggled advanced weapons).
    app.update();

    // Assert: A "HavenExposed" event is triggered, potentially alerting the parent factions and worsening diplomatic relations.
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// The SIMPLEST code that makes tests pass
// In a new `deserter_arrival_system`, check if two major factions are at war and the player's colony is neutral.
// Randomly spawn `Pop` entities with the `Deserter(FactionID)` component and boosted skill levels.
// In `social_interaction_system`, if two `Pop` entities with opposing `Deserter` factions interact, increase an `EscalatingTension` resource or local component.
// Create a `proxy_war_escalation_system` that checks if `EscalatingTension` exceeds a threshold. If so, spawn a combat event in the colony.
// If combat involves high-tier weapons (simulated by the deserters' stats), emit a `HavenExposedEvent`.
```

## 5. REFACTOR Phase: Quality & Design

- Ensure `Deserter` status is hidden from casual UI inspection unless the player has specific security/intelligence buildings, adding mystery.
- The conflict shouldn't be instant; let the tension simmer. Deserters might initially form cliques or gangs before open violence erupts.
- `HavenExposedEvent` should hook into the diplomacy system to drop relations with the warring empires, who view you as harboring criminals.

## 6. Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for the new module.
- [ ] Neutral colonies receive high-skill deserters during external wars.
- [ ] Proximity of opposing deserters leads to violent social conflicts.
- [ ] Escalated conflicts can expose the haven to the parent empires.

## 7. Technical Guidance

- Place logic in `src/layer1/social/deserters.rs`.
- `Deserter(FactionID)` should be a component on the `Pop`.
- You can leverage the existing `SocialCombat` or `Brawl` mechanics to handle the proxy war in the streets.
- Consider adding a `SmuggledWeapon` component or tag to the deserters' inventory to act as the trigger for exposure.

## 8. Questions

*Builder: add questions here if spec is unclear.*
