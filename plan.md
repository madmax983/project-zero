1. *Register the task as in-progress*
   - Use `run_in_bash_session` and `sed` to update `design/BACKLOG.md` to remove task 1121.
   - Use `run_in_bash_session` and `echo` to update `design/IN_PROGRESS.md` to add task 1121.
   - Run `git commit` to commit the changes.
2. *Implement the missing features for Generational Grudges*
   - Update `src/layer1/social/inherited_grudges.rs` to add `prevent_grudge_work_system`.
   - Update `src/layer1/social/mod.rs` to register the new system.
   - Use `run_in_bash_session` to check `cargo check` periodically to ensure code compiles.
   - Write tests for `prevent_grudge_work_system` as specified in RED phase (Work refusal).
   - Ensure Grudge formation and inheritance tests are complete.
3. *Run Tests*
   - Use `run_in_bash_session` to run `cargo test --lib layer1` and `cargo llvm-cov --lib --bins`.
4. *Finalize task*
   - Use `run_in_bash_session` to update `design/IN_PROGRESS.md` and `design/COMPLETED.md` with completion info.
5. *Pre-commit Step*
   - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
6. *Submit changes*
   - Run `git add` and `git commit` with the final commit message.
   - Use `submit` to push the branch.
