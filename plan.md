1. **Claim Task**: Update tracking files.
   - Run `sed -i '/692.*Procedural Dialects/d' design/BACKLOG.md`
   - Run `echo "- [ ] \`692\` Procedural Dialects — \`specs/692-procedural-dialects.md\` — claimed 2024-05-25" >> design/IN_PROGRESS.md`
2. **Verify Claim**: Read tracking files.
   - Run `cat design/BACKLOG.md | grep 692` to verify it was removed.
   - Run `cat design/IN_PROGRESS.md | grep 692` to verify it was added.
3. **RED Phase**: Write failing tests.
   - Run `cat << 'EOF' > src/layer1/culture/procedural_dialects.rs` to write the failing tests.
   - Run `echo "pub mod procedural_dialects;" >> src/layer1/culture/mod.rs` to export the module.
4. **Verify RED Phase**: Run tests to confirm they fail.
   - Run `cargo test procedural_dialects` to ensure the tests compile and fail.
5. **GREEN Phase**: Write minimal implementation.
   - Run `cat << 'EOF' > src/layer1/culture/procedural_dialects.rs` to overwrite with the full implementation and tests.
6. **Verify GREEN Phase**: Run tests to confirm they pass.
   - Run `cargo test procedural_dialects` to ensure the tests pass.
7. **REFACTOR Phase**: Check code quality and coverage.
   - Run `cargo clippy --all-targets --all-features -- -D warnings`
   - Run `cargo llvm-cov --lib --bins`
8. **Update Tracking Files**: Mark as completed.
   - Run `sed -i '/692.*Procedural Dialects/d' design/IN_PROGRESS.md`
   - Run `echo "- [x] \`692\` Procedural Dialects — \`specs/692-procedural-dialects.md\` — completed 2024-05-25" >> design/COMPLETED.md`
9. **Verify Tracking Updates**: Read tracking files.
   - Run `cat design/IN_PROGRESS.md | grep 692` and `cat design/COMPLETED.md | grep 692`.
10. **Pre-Commit**:
   - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
11. **Submit**: Run final tests and submit.
   - Run `cargo test` and submit.
