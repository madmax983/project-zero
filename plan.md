1. **Claim the Task**: Move `1285` from `design/BACKLOG.md` to `design/IN_PROGRESS.md` and use `git` to commit the change.
2. **Implement RED Phase Setup (Structs & Modules)**:
   - Create a new file `src/layer3/isolation.rs` containing the tests and minimal struct declarations for `ColonyNode`, `SilenceCult`, `ResidentOf`, `CultMember`, `SimulationTick`.
   - Update `src/layer3/mod.rs` to include `pub mod isolation;`.
   - Modify `src/layer1/psychology/needs.rs` to add `pub isolation: f32` to the `Needs` struct, setting its default to `0.0`. Add it to `NeedType`. Run `find src -type f -exec grep -l "hygiene:" {} \; | xargs sed -i 's/\(hygiene:[^,}]*\)[ \n]*}/\1, isolation: 0.0 }/g'` across all files to update existing `Needs` initializations that do not use `..Default::default()`.
3. **Implement RED Phase Logic (Write Failing Tests)**:
   - Add dummy logic (e.g. `colony.isolation_level = 0.0`) in `src/layer3/isolation.rs` so the compilation succeeds but the logical tests fail.
   - Run `cargo test` to verify tests fail and use `git` to commit the RED phase.
4. **Implement GREEN Phase (Make Tests Pass)**:
   - Use `replace_with_git_merge_diff` to implement the logic for `track_colony_isolation_system`, `process_isolation_needs_system`, and `spawn_silence_cult_system` in `src/layer3/isolation.rs` as specified in the spec.
   - Run `cargo test` and ensure the tests pass.
   - Run `cargo clippy -- -D warnings`.
   - Use `git` to commit the working implementation.
5. **Implement REFACTOR Phase**:
   - Use `replace_with_git_merge_diff` to introduce a configuration resource `SilenceCultConfig` to encapsulate the magic numbers `50.0`, `500.0`, and `0.1` inside `src/layer3/isolation.rs`.
   - Use `replace_with_git_merge_diff` to register the new systems via `schedule.add_systems(...)` within `src/simulation.rs`. Add `world.init_resource::<crate::layer3::isolation::SilenceCultConfig>();` to `setup_world_with_config` in `src/setup.rs`.
   - Use `git` to commit the refactored code.
6. **Final Verification**:
   - Run `cargo test`, `cargo clippy -- -D warnings`, and `cargo fmt` to perform a final verification of the system and ensure no regressions were introduced.
   - Ensure test coverage is >=85% using `cargo llvm-cov --lib --bins`.
7. **Pre-commit Checks**: Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
8. **Complete the Task**: Move `1285` from `design/IN_PROGRESS.md` to `design/COMPLETED.md` and use the `submit` tool to push changes.
