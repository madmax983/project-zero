1. **Handle Dependencies for Spec Tests:**
   - Use `run_in_bash_session` to create `src/layer1/jobs.rs` with mock definitions for `CurrentTask` and `ResourceType` since the codebase doesn't have these, but the spec explicitly uses them in the RED phase tests.
   - Use `cat` in `run_in_bash_session` to verify the creation and content of `src/layer1/jobs.rs`.
   - Use `run_in_bash_session` to add `pub mod jobs;` to `src/layer1/mod.rs` and export it.
   - Use `cat` in `run_in_bash_session` to verify `src/layer1/mod.rs`.

2. **Create Somnambulism Module:**
   - Use `run_in_bash_session` to create `src/layer1/somnambulism.rs`.
   - Use `run_in_bash_session` to populate the file with the `Somnambulist` component, `trigger_somnambulism_system`, and `process_somnambulist_work_system` as dictated by the RED/GREEN phases of the spec.
   - Use `cat` in `run_in_bash_session` to verify the contents of `src/layer1/somnambulism.rs`.
   - Use `run_in_bash_session` to add `pub mod somnambulism;` to `src/layer1/mod.rs`.
   - Use `cat` in `run_in_bash_session` to verify `src/layer1/mod.rs`.

3. **RED Phase Tests:**
   - Use `run_in_bash_session` to append the RED phase tests verbatim from the spec into `src/layer1/somnambulism.rs`.
   - Use `cat` in `run_in_bash_session` to verify the appended tests.
   - Use `run_in_bash_session` to execute `cargo test` to ensure they fail initially (RED phase).

4. **GREEN Phase Implementation:**
   - Use `run_in_bash_session` to add the minimal required logic to the systems in `src/layer1/somnambulism.rs` to make the tests pass. Note that `Needs` does not have a `set` or `get` method in the real code (it has direct public fields), so I will mock `Needs` logic using public fields to match the test structure without altering `Needs` definition. Wait, the spec tests call `needs.set(NeedType::Rest, 5.0)`. If `Needs` does not have `set`, the RED tests will fail to compile. I will add a `NeedType` enum and `get`/`set` methods to `src/layer1/needs.rs` to allow the exact spec tests to compile and run.
   - Use `cat` in `run_in_bash_session` to verify the changes.
   - Use `run_in_bash_session` to execute `cargo test` to ensure all tests pass.

5. **REFACTOR Phase (Integration):**
   - Use `replace_with_git_merge_diff` to update `calculate_work_amount` in `src/layer1/execution/general_work.rs` to multiply efficiency by `5.0` if `Somnambulist` is present.
   - Use `cat` in `run_in_bash_session` to verify `src/layer1/execution/general_work.rs`.
   - Use `replace_with_git_merge_diff` to update `decay_needs_system` in `src/layer1/needs.rs` to ignore pops with the `Somnambulist` component (to freeze needs).
   - Use `cat` in `run_in_bash_session` to verify `src/layer1/needs.rs`.
   - Use `replace_with_git_merge_diff` to register the new systems in `src/simulation.rs`.
   - Use `cat` in `run_in_bash_session` to verify `src/simulation.rs`.

6. **Verify Coverage and Correctness:**
   - Use `run_in_bash_session` to run `cargo test` and `cargo llvm-cov` to ensure all changes are correct, test coverage is >=85%, and no regressions were introduced.

7. **Pre-commit Checks:**
   - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
