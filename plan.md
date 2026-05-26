1. **Fix `Flesh Tax` memory timestamp again**
   - Use `replace_with_git_merge_diff` on `src/layer3/diplomacy/flesh_tax.rs` to fetch `Res<SimulationTime>` and use `time.tick` instead of `0` for the memory timestamp. Last time the merge diff failed to apply correctly.
   - Also, fix `FleshTaxPlugin` by removing it, as we successfully registered the systems manually in `simulation.rs`.
2. **Fix RED tests**
   - Update tests in `flesh_tax.rs` to use `SimulationTime` properly.
3. **Verify quality**
   - Run tests `cargo test --lib layer3::diplomacy::flesh_tax`.
   - Run `cargo clippy -- -D warnings`.
4. **Re-request code review**
   - Call `request_code_review` again.
5. **Pre-commit checks**
   - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
