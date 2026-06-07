1. **Move task to IN_PROGRESS.md**
   - Read `design/BACKLOG.md` and move task `1289` The Bureaucracy of Scarcity to `design/IN_PROGRESS.md`.
   - Commit with message "claim: 1289 the bureaucracy of scarcity".

2. **RED Phase: Write Failing Tests**
   - Create `src/layer1/administration/bureaucracy_of_scarcity.rs`.
   - Copy the RED phase tests from `specs/1289-the-bureaucracy-of-scarcity.md`.
   - Include it in `src/layer1/administration/mod.rs`.
   - Run tests and see them fail.
   - Commit with message "test(layer1): add RED phase tests for bureaucracy of scarcity".

3. **GREEN Phase: Minimal Implementation**
   - Implement the types and systems requested by the GREEN phase:
     - Component `Colony`
     - Component `ResourceStorage`
     - Component `JobBoard` and enum `JobType` with variant `RationingBureaucrat`
     - Component `Pop`
     - Enum `JobAssignment`
     - Component `ConsumptionRate`
     - Component `GlobalRationingModifier`
     - System `evaluate_scarcity_system`
     - System `apply_rationing_buff_system`
   - Add systems to the application in the tests and see the tests pass.
   - Commit with message "feat(layer1): implement bureaucracy of scarcity (GREEN phase)".

4. **REFACTOR Phase: Quality & Design**
   - Handle the job cleanup logic: Clear `JobType::RationingBureaucrat` when `storage.food > storage.population_demand`.
   - Handle dynamic base rate: Compute reduction dynamically using the actual base rate, not hardcoding `2.0`.
   - Run tests again, check coverage (`cargo llvm-cov --lib --bins`). Add tests if needed to reach >= 85%.
   - Commit with message "refactor(layer1): improve bureaucracy of scarcity code quality".

5. **Pre-commit checks**
   - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.

6. **Submit**
   - Mark as completed in `COMPLETED.md` and commit.
   - Submit the branch.
