import sys

plan = """# Implementation Plan for Spec 1205: The Scrap-Code Cult

1. **Claim Task**
   - Use `replace_with_git_merge_diff` to move task 1205 from `design/BACKLOG.md` to `design/IN_PROGRESS.md`.
   - Execute `git add design/` and `git commit -m "claim: 1205 scrap code cult"`.

2. **Update Traits**
   - Use `replace_with_git_merge_diff` to add the `Enlightened` variant to the `Trait` enum and update its corresponding string match arm in `src/layer1/psychology/traits.rs`.

3. **Create Test File and RED Phase Tests**
   - Use `write_file` to create `src/layer1/social/scrap_code_cult.rs`.
   - Define the minimal required structs (`InfrastructureNode`, `GridInstabilityEvent`) inside the `src/layer1/social/scrap_code_cult.rs` file to satisfy the tests.
   - Write the failing tests as described in the RED phase of the spec within this file.
   - Use `replace_with_git_merge_diff` to add `pub mod scrap_code_cult;` to `src/layer1/social/mod.rs` so the file is compiled.

4. **Verify RED Phase Tests Fail**
   - Run `cargo test scrap_code_cult` to ensure the new RED phase tests fail as expected.

5. **Commit RED Phase**
   - Execute `git add .` followed by `git commit -m "test(layer1): add RED phase tests for scrap code cult"`.

6. **Implement GREEN Phase**
   - Use `replace_with_git_merge_diff` to implement the required components (`ScrapDiscoveryProgress`, `ScrapLogicApplied`, `CommsRelay`, `CultInfluence`, `ScrapBroadcastEvent`) and systems (`discover_scrap_code_system`, `scrap_code_cult_gathering_system`, `apply_scrap_logic_system`, `attempt_relay_hijack_system`) in `src/layer1/social/scrap_code_cult.rs`.
   - The implementation will use `Trait::Enlightened`. The spec uses `PopTrait::Enlightened("Scrap-Code Cult".into())`, but since we added `Trait::Enlightened` to the `Trait` enum in `src/layer1/psychology/traits.rs`, we will use `traits.has(Trait::Enlightened)` and `traits.add(Trait::Enlightened)`.
   - The spec uses `JobType::Maintenance`, but `AssignmentType` doesn't have a `Maintenance` variant. I will use `AssignmentType::FarmWorker` or add `Maintenance` to `AssignmentType` in `src/layer1/mind/utility_types.rs`. For this plan, I'll add `Maintenance` to `AssignmentType`. I'll use `replace_with_git_merge_diff` to add `Maintenance` to `AssignmentType` in `src/layer1/mind/utility_types.rs` if needed, but I will first check what job to use. Let's just use `AssignmentType::LibraryWorker` to keep it simple, or add `Maintenance`. The spec asks for `Maintenance`. I will add `Maintenance` to `AssignmentType`.

7. **Add Maintenance to AssignmentType**
   - Use `replace_with_git_merge_diff` to add `Maintenance` to `AssignmentType` in `src/layer1/mind/utility_types.rs`.
   - Update `get_wage_for_job` in `src/layer1/economy/mod.rs` to handle `AssignmentType::Maintenance`.
   - Update `movement_system` in `src/layer1/execution/movement.rs` and `assign_pop` in `src/layer1/execution/arrival.rs` to handle `AssignmentType::Maintenance`.

8. **Verify GREEN Phase Tests Pass**
   - Run `cargo test scrap_code_cult` and `cargo clippy --all-targets --all-features -- -D warnings` to verify the implementation passes tests and lints cleanly.

9. **Commit GREEN Phase**
   - Execute `git add .` followed by `git commit -m "feat(layer1): implement scrap code cult system (GREEN phase)"`.

10. **Refactor Code**
    - Use `replace_with_git_merge_diff` to refactor `apply_scrap_logic_system` in `src/layer1/social/scrap_code_cult.rs` to select `InfrastructureNode`s near cult gatherings using `GridPosition` and `manhattan_distance`, improving realism per the REFACTOR phase of the spec.
    - Also use `Res<Time>` for the discovery progress in `discover_scrap_code_system`.

11. **Verify Refactor Tests Pass**
    - Run `cargo test scrap_code_cult` and `cargo clippy --all-targets --all-features -- -D warnings` to verify the refactor hasn't broken anything.

12. **Commit REFACTOR Phase**
    - Execute `git add .` followed by `git commit -m "refactor(layer1): improve scrap code cult code quality"`.

13. **Verify Coverage**
    - Run `cargo llvm-cov --lib --bins` to ensure the new module has at least 85% coverage.

14. **Run All Tests**
    - Run `cargo test --lib` and `cargo test --bins` to verify all tests pass and no regressions were introduced.

15. **Update Backlog**
    - Use `replace_with_git_merge_diff` to move `1205` from `design/IN_PROGRESS.md` to `design/COMPLETED.md`.

16. **Commit Completion**
    - Execute `git add .` followed by `git commit -m` with the exact multiline completion message:
```
feat(layer1): complete scrap code cult system

Implements RED-GREEN-REFACTOR from spec 1205:
- Added comprehensive test suite (RED phase)
- Implemented ScrapCode Cult logic and grid instability events (GREEN phase)
- Refactored logic to target specific infrastructure nodes (REFACTOR phase)
- Test coverage: >= 85%

All acceptance criteria met.

Co-Authored-By: google-labs-jules[bot] <161369871+google-labs-jules[bot]@users.noreply.github.com>
```

17. **Pre-commit Steps**
    - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.

18. **Submit**
    - Call the `submit` tool to finalize the task without making further commits.
"""
print(plan)
