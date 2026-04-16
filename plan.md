1. **Goal**: Implement `Social Debt` (Spec 894). The `SocialDebt` component already exists in `src/layer1/social/debt.rs` but is not fully integrated/registered in the main game loops, and spec 894 defines new tests and systems (`LifeSavedEvent`, `CallInFavorEvent`, `ActiveSupport`, `process_life_saved_system`, `evaluate_faction_support_system`, `process_favors_system`).

2. **RED Phase**:
   - Add `LifeSavedEvent`, `CallInFavorEvent`, `ActiveSupport` to `src/layer1/social/debt.rs`.
   - Add the tests from the spec (adapted for `SocialDebt` with `f32` instead of `HashMap<Entity, u32>` if I want to re-use existing `SocialDebt`).
   - Wait, `SocialDebt` uses `f32` (amounts like 50.0). The spec uses `u32` (amounts like 1). I will adapt the tests to use `f32` and `SocialDebt`'s methods like `add_debt(savior, 1.0)`.
   - The spec tests will be added to the end of `src/layer1/social/debt.rs`.
   - `cargo test --lib layer1::social::debt` to verify RED phase failure.

3. **GREEN Phase**:
   - Implement `process_life_saved_system` which maps `LifeSavedEvent` to adding debt to `SocialDebt` (can just emit `FavorChange` or modify `SocialDebt` directly).
   - Implement `process_favors_system` which decrements debt from `SocialDebt`.
   - Implement `evaluate_faction_support_system` which updates `ActiveSupport`.
   - Make tests pass.

4. **Integration**:
   - Register the new systems and events in `src/layer1/systems/observation.rs` (or `economy.rs`, `debt.rs` isn't registered currently anywhere in `systems/`). Wait! If `accrue_debt_system` isn't registered, I should probably register all `debt.rs` systems (`accrue_debt_system`, `debt_decay_system`, `debt_impact_system`, `process_life_saved_system`, `evaluate_faction_support_system`, `process_favors_system`) in `src/layer1/systems/observation.rs`. And register the events in `src/setup.rs`.

5. **REFACTOR Phase & Pre-Commit**:
   - Run formatting, clippy.
   - Use `cargo llvm-cov --lib | grep debt` to verify 85% coverage.
   - Pre-commit step and submit.
