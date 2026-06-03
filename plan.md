1. **Claim the task**:
   - Remove `1271` from `design/BACKLOG.md`.
   - Add `1271` to `design/IN_PROGRESS.md`.
2. **Verify task claim**:
   - Verify changes using `cat design/BACKLOG.md` and `cat design/IN_PROGRESS.md`.
3. **Clean up temporary files**:
   - Delete all temporary Python and Markdown scripts using `rm patch_graffiti.py patch_work.py patch_graffiti_tests.py patch_graffiti_tests2.py patch_graffiti_tests3.py patch_graffiti_clippy.py patch_graffiti_clippy2.py plan.md`.
4. **Complete the task**:
   - Remove `1271` from `design/IN_PROGRESS.md`.
   - Add `1271` to `design/COMPLETED.md`.
5. **Verify task completion**:
   - Verify changes using `cat design/IN_PROGRESS.md` and `cat design/COMPLETED.md`.
6. **Testing and Validation**:
   - Run `cargo fmt --all`, `cargo check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, and `cargo llvm-cov --lib --bins` in a bash session to ensure code changes are formatted, compile, pass linters, tests, and have sufficient coverage.
7. **Pre Commit Steps**:
   - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
8. **Submit**:
   - Commit the changes and invoke the `submit` tool.
