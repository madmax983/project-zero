1. **Add `slippery_slope` to `src/layer1/social/mod.rs`**
   - Use `run_in_bash_session` to execute `echo 'pub mod slippery_slope;' >> src/layer1/social/mod.rs`.
   - Use `run_in_bash_session` to execute `tail -n 10 src/layer1/social/mod.rs` to verify the edit.

2. **Update `src/layer1/social/slippery_slope.rs` to include stress penalty dampening**
   - Use `run_in_bash_session` to execute `cat << 'EOF' > src/layer1/social/slippery_slope.rs` containing the complete implementation (`Desensitization`, `AtrocityEvent`, `MoraleBuffEvent`, `StressPenaltyEvent`, `process_atrocities`, `apply_morale_buffs`, `apply_stress_penalties`, and the RED-GREEN-REFACTOR tests achieving >85% coverage).
   - Use `run_in_bash_session` to execute `cat src/layer1/social/slippery_slope.rs` to verify its creation and contents.

3. **Update task trackers**
   - Use `run_in_bash_session` to execute `sed -i '/1228/d' design/BACKLOG.md` and `echo '- [x] \`1228\` The Slippery Slope — \`specs/1228-the-slippery-slope.md\` — completed 2026-06-28' >> design/COMPLETED.md`.
   - Use `run_in_bash_session` to read the files back using `tail -n 10 design/COMPLETED.md` and `git diff design/BACKLOG.md` to verify the changes.

4. **Verify tests and coverage locally**
   - Use `run_in_bash_session` to execute `cargo test --lib` and `cargo llvm-cov --lib` to ensure local tests pass and test coverage criteria are met.
   - Use `run_in_bash_session` to execute `cargo clippy -- -D warnings` and `cargo fmt`.

5. Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.

6. **Submit the PR**
   - Use `submit` with the following commit message format:
     ```
     feat(layer1): complete the slippery slope system

     Implements RED-GREEN-REFACTOR from spec 1228:
     - Added comprehensive test suite (RED phase)
     - Implemented Desensitization component, apply_morale_buffs, apply_stress_penalties (GREEN phase)
     - Refactored logic to include Stress dampening (REFACTOR phase)
     - Test coverage: >= 85%

     All acceptance criteria met:
     - Atrocities increase desensitization
     - High desensitization dampens Morale Buffs
     - High desensitization dampens Stress Penalties
     - cargo test passes
     - cargo clippy clean

     Co-Authored-By: google-labs-jules[bot] <161369871+google-labs-jules[bot]@users.noreply.github.com>
     ```
