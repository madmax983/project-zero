1. **Register `execute_bombardment_system` in `src/simulation.rs`:**
   - I will modify `src/simulation.rs` to add `crate::layer2::bombardment::execute_bombardment_system` to the schedule. It should be chained correctly. Since it's an action, it can be added around line 522 along with other Layer 2 integration systems.
2. **Update `design/IN_PROGRESS.md`:**
   - Add an entry for `INT-659` (Orbital Bombardment Integration) to track this work.
3. **Verify tests:**
   - Run `cargo test` to ensure that all tests pass, including `test_bombardment` integration tests.
4. **Complete Pre-Commit Steps:**
   - I will call `pre_commit_instructions` to ensure proper testing, verification, review, and reflection are done.
5. **Move to `COMPLETED.md`:**
   - Move the task from `IN_PROGRESS.md` to `COMPLETED.md`.
6. **Submit changes:**
   - Submit the branch using standard git messages.
