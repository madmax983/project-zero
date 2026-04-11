# 958: The Scapegoat Ritual

## 1. Overview
When global Unrest reaches critical levels due to compounding disasters, Pops form a "Mob" faction. They select a low-status or outsider Pop and demand their execution. Approving the execution instantly resets Unrest and provides a massive, temporary "Catharsis" morale boost, but permanently damages relationships with the victim's family and certain ethical factions.

## 2. Dependencies
- `010` Pop System
- `114` Factions
- `060` Crime and Punishment

## 3. RED Phase: Tests First
```rust
#[test]
fn test_scapegoat_mob_formation_under_high_unrest() {
    // Arrange: Colony with critical unrest and compounding disaster events.
    let mut app = App::new();

    // Act: Process faction sentiment.
    app.update();

    // Assert: A new `MobFaction` is temporarily formed with a `ScapegoatDemand(TargetEntity)`.
}

#[test]
fn test_executing_scapegoat_resets_unrest() {
    // Arrange: A MobFaction demanding a scapegoat execution.
    let mut app = App::new();

    // Act: The player (or AI) approves the execution.
    app.update();

    // Assert: Global unrest drops to 0, and a temporary `Catharsis` morale boost is applied to the colony.
}

#[test]
fn test_scapegoat_execution_damages_family_relations() {
    // Arrange: Scapegoat executed.
    let mut app = App::new();

    // Act: Process post-execution social relations.
    app.update();

    // Assert: The executed Pop's family members gain a severe negative sentiment toward the ruling faction.
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// In `unrest_evaluation_system`, if `GlobalUnrest` > 90%, identify the lowest status pop and spawn a `MobDemandEvent(TargetEntity)`.
// If the player issues an `ExecutePopCommand(TargetEntity)` matching the demand, reset `GlobalUnrest` to 0.
// Apply `Catharsis` modifier to all standard pops.
// In `social_relations_system`, look up family relationships for the `TargetEntity` and apply a permanent `Resentment` marker toward the administration.
```

## 5. REFACTOR Phase: Quality & Design
- Target selection should prioritize outcasts, minorities, or pops with high `SocialFriction` to fit the narrative.
- Ensure the `Catharsis` modifier is temporary but strong enough to stabilize the colony.
- Warn the player about the long-term consequences of indulging the mob.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for the new module.
- [ ] High unrest triggers the Scapegoat demand.
- [ ] Executing the scapegoat resolves the unrest entirely.
- [ ] Long-term relationship damage is applied to the victim's close relations.

## 7. Technical Guidance
- Integrate into `src/layer1/justice/scapegoat.rs`.
- `MobDemandEvent` must be visible via UI or Chronicle so the player can react.

## 8. Questions
*Builder: add questions here if spec is unclear.*
