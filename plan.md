1. **Update `src/layer1/psychology/traits.rs`**:
   - Use `replace_with_git_merge_diff` on `src/layer1/psychology/traits.rs`.
   - Update `get_job_efficiency_modifier` for `Trait::MoleEyes` to grant a bonus for `AssignmentType::DeepMining` and a penalty for others.
   - Update the test `test_job_efficiency_modifiers` in `src/layer1/psychology/traits.rs` to reflect the new functionality.
2. **Verify changes to `traits.rs` and commit**:
   - Use `run_in_bash_session` to run `cargo test -- psychology::traits::tests && cargo test -- specialization && cargo clippy -- -D warnings && git add . && git commit -m "feat(layer1): implement Hyper-Specialized Evolution (GREEN phase)"`.
3. **Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.**
4. **Submit the change.**
