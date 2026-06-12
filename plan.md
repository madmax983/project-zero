1. **Understand the Goal**: Implement `specs/1300-phantom-commutes.md`. The feature makes pops with `HabituatedRoute` follow their old cached paths, overriding the normal `movement_system`.

2. **Components (`src/layer1/execution/components.rs`)**:
    *   Add `HabituatedRoute` to `src/layer1/execution/components.rs`:
        Write a Python script to append `HabituatedRoute` to `components.rs` with `path`, `urgency`, `frustration`, and `last_pos` fields.
    *   Execute the python script, then run `tail -n 20 src/layer1/execution/components.rs` to verify.

3. **Systems (`src/layer1/execution/phantom_commutes.rs`)**:
    *   Create `src/layer1/execution/phantom_commutes.rs` using `cat << 'EOF' >`.
    *   Implement `apply_phantom_commute_system` which overrides `target_position` in `MovementTarget` with the next step in `HabituatedRoute` path, and tracks `frustration` if `last_pos` matches current position.
    *   Implement `remove_resolved_phantom_commute_system` which removes `HabituatedRoute` if the pop reaches the last step or frustration exceeds 10.
    *   Implement `tests` module containing RED phase tests from spec (adapted to codebase components).
    *   Verify by running `cat src/layer1/execution/phantom_commutes.rs`.

4. **Integration**:
    *   Use a Python script to add `pub mod phantom_commutes; pub use phantom_commutes::*;` to `src/layer1/execution/mod.rs`. Verify with `cat src/layer1/execution/mod.rs`.
    *   Use a Python script to register `apply_phantom_commute_system.before(movement_system)` and `remove_resolved_phantom_commute_system.after(movement_system)` in `src/layer1/systems/execution.rs`. Verify with `cat src/layer1/systems/execution.rs`.

5. **Testing and Linting**:
    *   Run `cargo test --lib scale` to ensure all tests pass (GREEN phase).
    *   Run `cargo clippy -- -D warnings` and `cargo fmt`.
    *   Run `cargo llvm-cov --lib` to verify coverage.

6. **Git and Docs**: Update `IN_PROGRESS.md` and `COMPLETED.md` to reflect the completed task. Run `git add .` and `git commit` including the exact tag `-m 'Co-Authored-By: google-labs-jules[bot] <161369871+google-labs-jules[bot]@users.noreply.github.com>'`.

7. **Pre-commit**: Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.

8. **Submit PR**: Submit the changes.
