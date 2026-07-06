1. **Claim the task `1009`**:
   - Move `1009` from `BACKLOG.md` to `IN_PROGRESS.md`.
   - Commit the claim.

2. **RED Phase**:
   - Create `src/layer1/feral_overlord.rs`.
   - Implement the tests `test_excavating_ancient_server_awakens_feral_ai` and `test_feral_ai_issues_high_priority_work_orders`.
   - Since `WorkOrder` and `Priority` don't exist in `src/layer1/utility_ai` (or anywhere), we will define them right in `src/layer1/feral_overlord.rs` or a dummy module to satisfy the spec and the tests. Wait, the spec explicitly says:
     "You may need to create dummy `TaskTypes` for the AI to issue if things like "BuildStatue" don't exist yet in the codebase."
     And "WorkOrder" might need to be created too. I'll define `WorkOrder` and `Priority` in `feral_overlord.rs` for now.

3. **GREEN Phase**:
   - Implement `FeralOverlordAI` and systems `evaluate_excavation_discoveries_system`, `feral_ai_directive_system`.
   - Make the tests pass.

4. **REFACTOR Phase**:
   - Clean up code. Add to schedules. (Wait, the spec doesn't strictly say we must register in a schedule in the implementation file, but we should make sure tests pass.)

5. **Test and Verify**:
   - Run `cargo test feral_overlord`.
   - Check test coverage using `cargo llvm-cov`.
   - Run `cargo clippy`.

6. **Pre-commit Step**:
   - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.

7. **Submit**:
   - Move task to `COMPLETED.md`.
   - Commit and submit.
