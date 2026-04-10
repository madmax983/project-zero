1. **Integration and Tests**
   - Use `replace_with_git_merge_diff` to add `ProductivityModifier` to `PopBundle` in `src/layer1/entities/pop.rs`.
   - Use `run_in_bash_session` with `cat` to verify changes to `pop.rs`.
   - Use `replace_with_git_merge_diff` to add comprehensive test cases to `src/layer1/social/strike/work_shift_cartel.rs` for cartel members and unaffected sectors.
   - Use `run_in_bash_session` with `cat` to verify changes to `work_shift_cartel.rs`.

2. **Validation & Coverage**
   - Use `run_in_bash_session` to run `cargo test --lib layer1::social::strike::work_shift_cartel` to ensure the new tests pass.
   - Use `run_in_bash_session` to run `cargo llvm-cov` to ensure coverage is high enough.

3. **Pre Commit**
   - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
