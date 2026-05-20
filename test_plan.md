# Review Plan

The task is to implement the "Rocket Equation" feature as defined in `specs/1060-the-rocket-equation.md`.

## Proposed Plan

1. **Claim Task**: Move `1060` from `BACKLOG.md` to `IN_PROGRESS.md` and commit.
2. **RED Phase**: Create `src/layer2/logistics.rs` (or modify `src/layer2/ship.rs` based on module organization) and add the failing tests from the spec. Note: I will create `src/layer2/logistics.rs` and add `pub mod logistics;` to `src/layer2/mod.rs`, and export components from there. Include tests `test_ship_consumes_delta_v_on_movement`, `test_ship_becomes_stranded_when_delta_v_depleted`, and `test_tanker_refuels_stranded_ship`.
3. **Run tests to confirm failure**: Ensure tests fail.
4. **GREEN Phase**: Implement the minimal components (`DeltaV`, `Moving`, `Position`, `Stranded`, `DistressBeacon`, `Tanker`, `RefuelingTarget`) and systems (`movement_consumes_delta_v_system`, `stranded_system`, `refueling_system`) to make the tests pass.
5. **Run tests to confirm passing**: Run `cargo test` and `cargo clippy`.
6. **REFACTOR Phase**: Enhance the distance check to use Bevy `Transform` if needed, or keep `Position` if it's the 2D equivalent used in tests. In this case, I'll stick to the spec's `Position(Vec2)` but ensure we are ready to use `Transform` or `Vec3` if required, but the tests define `Position(Vec2)` so I will just use `Position(Vec2)` as requested by the test. I will ensure systems are correctly registered if needed, but the spec says they can just be functions.
7. **Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.**
8. **Final Verification**: Check coverage (`cargo llvm-cov`), fmt, clippy, and run all tests.
9. **Complete Task**: Move from `IN_PROGRESS.md` to `COMPLETED.md` and submit.
