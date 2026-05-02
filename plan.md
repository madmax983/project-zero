1. **Move task to IN_PROGRESS**
   - Use `sed` to update `design/BACKLOG.md` and `design/IN_PROGRESS.md`.
   - Run `git add` and `git commit` to mark the task as claimed.

2. **Create `PetrificationSickness` module (RED Phase)**
   - Use `cat << 'EOF' > src/layer1/culture/petrification_sickness.rs` to write the complete RED phase and tests based on the spec, using the proper components available in the codebase (`Speed` instead of `MovementStats`, creating a minimal `DamageResistance` component locally, and using `ExcavationEvent` with `ResonantOre` as the trigger).
   - Use `cat src/layer1/culture/petrification_sickness.rs` to verify the file was created properly.

3. **Implement GREEN Phase**
   - Add the required implementations into `src/layer1/culture/petrification_sickness.rs` via another `cat << 'EOF' >>` command (or just include it directly in the step 2 heredoc for efficiency, along with the tests!).
   - Include components: `PetrificationSickness`, `DamageResistance`.
   - Include systems: `petrification_exposure_system`, `petrification_progression_system`, `petrification_transformation_system`.

4. **Register Module and Systems**
   - Use `cat << 'EOF' >> src/layer1/culture/mod.rs` to export `petrification_sickness`.
   - Use `replace_with_git_merge_diff` to add the systems to the schedule in `src/layer1/systems/execution.rs` (or `economy.rs` where culture systems typically go - I will use `grep` to find where `art_generation_system` is and put it there, wait `art_generation_system` is in `src/layer1/systems/economy.rs`, I will register it there).

5. **Run Tests in Background**
   - Run `cargo test &> test_output.log &`
   - Monitor tests with `sleep 10 && tail -n 50 test_output.log`
   - Run `cargo clippy -- -D warnings` and `cargo llvm-cov` if needed to verify 85% coverage.
   - Clean up with `rm test_output.log`.

6. **Submit Prep (COMPLETED.md and git commit)**
   - Move task from `IN_PROGRESS.md` to `COMPLETED.md`.
   - Commit the final implementation with standard format `feat(layer1): complete petrification sickness system...`

7. **Complete pre-commit steps**
   - Run `pre_commit_instructions` and follow them to ensure proper testing, verification, review, and reflection are done.

8. **Submit**
   - Call the `submit` tool to finish.
