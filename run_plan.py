plan = """1. **Claim the issue**
   - Update `design/BACKLOG.md` to remove `1132` The Deep Crust Resonance using `replace_with_git_merge_diff`.
   - Update `design/IN_PROGRESS.md` to add `- [ ] \`1132\` The Deep Crust Resonance - \`specs/1132-deep-crust-resonance.md\` - claimed 2026-05-18` using `replace_with_git_merge_diff`.
   - Use `run_in_bash_session` to verify the updates with `cat`.
2. **Implement the TDD test**
   - Use `write_file` to create `src/layer1/deep_crust_resonance.rs` containing the specific failing RED phase tests defined in the spec.
   - Use `run_in_bash_session` to verify the file contents with `cat` and run `cargo test` to confirm the tests fail.
3. **Implement GREEN phase**
   - In `src/layer1/deep_crust_resonance.rs` using the `write_file` tool, create `ResonantOre` and `ResonantInfection` components as shown in spec.
   - Implement `resonant_ore_exposure_system` and `resonance_social_spread_system` systems as shown in spec.
   - Use `run_in_bash_session` (with `cat`) to verify the file was updated.
4. **Refactor and add module**
   - Add module declaration `pub mod deep_crust_resonance;` to `src/layer1/mod.rs` using `replace_with_git_merge_diff`.
   - Verify the modification of `src/layer1/mod.rs` using `cat src/layer1/mod.rs`.
   - Use `run_in_bash_session` to run `cargo llvm-cov --lib` to check coverage.
5. **Add Systems**
   - Add `resonant_ore_exposure_system` to `src/layer1/systems/execution.rs` and `resonance_social_spread_system` to `src/layer1/systems/observation.rs` using `replace_with_git_merge_diff`.
   - Verify file modifications using `cat`.
6. **Mark as Completed**
   - Update `design/IN_PROGRESS.md` to remove the item using `replace_with_git_merge_diff`.
   - Update `design/COMPLETED.md` to add the item using `replace_with_git_merge_diff`.
   - Verify file modifications using `cat`.
7. **Verification**
   - Run `cargo fmt`, `cargo check`, `cargo clippy -- -D warnings`, `cargo test` using `run_in_bash_session`.
8. **Pre-commit**
   - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
9. **Finish**
   - Submit the changes using the `submit` tool.
"""
print(plan)
