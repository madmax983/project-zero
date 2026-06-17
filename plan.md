1. **Implement TDD RED Phase**
   - We already have the module `src/layer1/tech/teleporter/psychosis.rs` with RED phase tests matching `specs/988-teleporter-psychosis.md`.
2. **Implement GREEN Phase**
   - Write the minimal code to pass the RED tests.
3. **Refactor Phase**
   - Apply any reasonable refactors like constants if needed.
4. **Integration**
   - Ensure the `TeleportEvent` resource and the 3 systems (`handle_teleport_system`, `process_psychosis_system`, `hunger_decay_system`) are registered in `src/simulation.rs`. (Actually, already did this locally but will formalize it.)
5. **Coverage & Checks**
   - Ensure `cargo llvm-cov` passes, and the codebase passes `cargo clippy -- -D warnings` and `cargo fmt`.
   - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
