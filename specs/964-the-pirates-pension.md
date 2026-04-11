# 964: The Pirate's Pension

## 1. Overview
Your colony becomes the retirement home for the galaxy's most wanted criminals. You can enact a policy to offer "Amnesty Visas" to Layer 3 Pirate Fleets. The fleets disband, and the pirate crews land as Pops on your Layer 1 colony. They arrive with massive amounts of stolen Credits and rare goods, instantly boosting your local economy, but they possess terrible work ethics and severe "Criminal History" traits. The immediate, massive economic bailout comes with the long-term diplomatic consequences and the destruction of your colony's productive work culture.

## 2. Dependencies
- `084` Pop Traits
- `159` Fleet Combat

## 3. RED Phase: Tests First
```rust
#[test]
fn test_pirate_pension_amnesty() {
    // Arrange: A colony with the "Amnesty Visa" policy enacted, and an approaching Layer 3 Pirate Fleet.
    let mut app = App::new();

    // Act: Process diplomacy tick where the pirate fleet accepts amnesty.
    app.update();

    // Assert: The pirate fleet is removed. New pops are spawned in the colony with the "Pirate" trait and high credits.
}

#[test]
fn test_pirate_work_ethic() {
    // Arrange: A colony with "Pirate" pops assigned to work.
    let mut app = App::new();

    // Act: Process a work shift.
    app.update();

    // Assert: The "Pirate" pops refuse to work, steal items, or start brawls instead of completing tasks.
}
```

## 4. GREEN Phase: Minimal Implementation
```rust
// Add a `Pirate` marker component for traits.
// Add a system `evaluate_pirate_amnesty_system` in Layer 3 diplomacy that disband Pirate fleets if "Amnesty Visa" policy is active.
// In Layer 1, spawn new `Pop` entities with the `Pirate` trait, adding large credit balances to their `Wallet`.
// Modify the `work_execution_system` or utility AI to greatly lower work scores for pops with the `Pirate` trait, and randomly trigger crimes or brawls instead.
```

## 5. REFACTOR Phase: Quality & Design
- Integrate with existing crime and justice systems (e.g. `src/layer1/justice`) so they don't just infinitely brawl without consequences.
- Consider UI notifications for the arrival of the pirates and warnings about the subsequent drop in productivity.
- Cleanly separate the Layer 3 fleet disbanding logic from the Layer 1 pop generation logic.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage >= 85% for the new module.
- [ ] Pirate fleets can accept amnesty and turn into pops.
- [ ] Pirate pops receive a massive initial credit boost.
- [ ] Pirate pops frequently shirk work and engage in brawls/crime.

## 7. Technical Guidance
- The Layer 3 -> Layer 1 transition might need an integration bridge event like `PirateAmnestyEvent` to cleanly decouple the layers.
- Check `src/layer1/economy/mod.rs` for `Wallet` interaction.

## 8. Questions
*Builder: add questions here if spec is unclear.*
