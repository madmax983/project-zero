1. **Format and Lint**
   - Run `cargo fmt --all` and `cargo clippy --all-targets --all-features -- -D warnings` using `run_in_bash_session`.

2. **Test Changes**
   - Use `run_in_bash_session` to run `cargo test --lib -- physics::pressure && cargo test --lib -- physics::structural_integrity && cargo test --lib -- physics::vent`.

3. **Commit Changes**
   - Use `run_in_bash_session` to run `git add . && git commit --amend --no-edit`

4. **Pre-Commit Steps**
   - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.

5. **Submit**
   - Use `submit` to finalize the PR.
