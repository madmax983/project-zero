Plan:
1. I will implement `1249` Ship Personalities.
2. I will claim `1249` in `design/IN_PROGRESS.md` and commit.
3. I will write the failing RED phase tests in `src/layer2/ship.rs`.
   - The test will need to be slightly adapted for existing `Ship` structure (like `Ship { health, max_health, .. }` instead of separate `Health` component, or however it's modeled in the code). Wait! `src/layer2/ship.rs` has `health` directly inside the `Ship` struct! So I'll just use that.
4. I will implement the GREEN phase (the system `ship_quirk_system` that handles `BattleSurvivedEvent` and modifies `Quirks`).
5. I'll add `Quirks` and `BattleSurvivedEvent` to `src/layer2/ship.rs`.
6. Run tests to ensure it's green.
7. Refactor if needed.
8. Update `COMPLETED.md` and PR.
