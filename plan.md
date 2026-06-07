1. **Claim the Task:** Update `design/BACKLOG.md` to remove task 1289 and add it to `design/IN_PROGRESS.md`, then commit.
2. **RED Phase (Tests):**
   - Create a new module `src/layer1/bureaucracy_of_scarcity.rs`.
   - Add the test module described in the `RED Phase` of `specs/1289-the-bureaucracy-of-scarcity.md`.
   - Ensure the module is registered in `src/layer1/mod.rs` (e.g. `pub mod bureaucracy_of_scarcity;`).
   - Run `cargo test` and verify that the tests fail, then commit the RED phase.
3. **GREEN Phase (Implementation):**
   - Implement the `Colony`, `ResourceStorage`, `JobType`, `JobBoard`, `Pop`, `JobAssignment`, `ConsumptionRate`, and `GlobalRationingModifier` components and the two systems (`evaluate_scarcity_system`, `apply_rationing_buff_system`) in `src/layer1/bureaucracy_of_scarcity.rs` as specified in the `GREEN Phase` of the spec.
   - Run `cargo test` and verify that the tests pass.
   - Check with `cargo clippy -- -D warnings` and fix any warnings.
   - Commit the GREEN phase.
4. **REFACTOR Phase (Quality Improvements):**
   - Implement Job Cleanup: In `evaluate_scarcity_system`, remove `JobType::RationingBureaucrat` from `JobBoard` when `storage.food >= storage.population_demand / 2` (or the condition specified in REFACTOR phase to clear jobs).
   - Implement Dynamic Base Rate: Modify `apply_rationing_buff_system` to calculate the new rate based on the Pop's base rate (we will need to add a `base_food_per_tick` field to `ConsumptionRate` to track this without losing precision over multiple ticks).
   - *Skip generalizing resource types* if it requires significant changes to existing architecture or if `ResourceStorage` can be kept specific to the current tests.
   - Run `cargo test` to ensure changes haven't broken the tests. Add tests for the refactoring changes if needed to hit coverage.
   - Commit the REFACTOR phase.
5. **Coverage:** Run `cargo llvm-cov --lib --bins` and ensure the coverage is >= 85%.
6. **Pre-commit:** Run pre-commit checks.
7. **Complete & Submit:** Move task to `design/COMPLETED.md`, run `cargo fmt`, commit, and submit via `submit` tool.
