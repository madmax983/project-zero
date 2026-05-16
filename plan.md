1. **Claim the task**
   - Use `sed` to remove the line `- [ ] \`1126\` Orbital Drydocks — \`specs/1126-orbital-drydocks.md\`` from `design/BACKLOG.md`.
   - Use `sed` to add the line `- [ ] \`1126\` Orbital Drydocks — \`specs/1126-orbital-drydocks.md\` — claimed 2026-03-24` to `design/IN_PROGRESS.md`.
   - Use `run_in_bash_session` to execute `git add design/`, `git commit -m 'claim: 1126 orbital drydocks'`, and `git push`.

2. **Add RED Phase Tests**
   - Note: The test file `tests/integration/orbital_drydocks.rs` has already been created, and the `tests/integration/mod.rs` file has already been updated.
   - Run `cargo test --test integration orbital_drydocks` to verify the tests fail.
   - Use `run_in_bash_session` to execute `git add tests/integration/orbital_drydocks.rs tests/integration/mod.rs`, `git commit -m 'test(layer2): add RED phase tests for orbital drydocks'`.

3. **Implement GREEN Phase**
   - Edit `src/layer2/station.rs`.
   - Add `OrbitalDrydock` to the `StationType` enum.
   - Update implementations of methods `cost()`, `label()`, and `char()` on `StationType` to handle `OrbitalDrydock`. Provide values like `vec![(ResourceType::Metal, 1000.0)]`, `"Orbital Drydock"`, and `'D'` respectively.
   - Create `src/layer2/orbital_drydock.rs`.
   - Add the `ShipConstruction` component.
   - Add the `process_drydock_construction_system`.
   - Use `run_in_bash_session` to safely append `pub mod orbital_drydock;` to `src/layer2/mod.rs` using `cat << 'EOF' >>`.
   - Register `process_drydock_construction_system` in `src/simulation.rs` in the `schedule.add_systems` block next to `crate::layer2::station::build_station_system`.
   - Run `cargo test --test integration orbital_drydocks` to verify the tests pass.
   - Use `run_in_bash_session` to execute `git add .`, `git commit -m 'feat(layer2): implement orbital drydocks system (GREEN phase)'`.

4. **Verify Implementation**
   - Run `cargo test` to verify all tests pass and no regressions are introduced system-wide.
   - Run `cargo clippy --all-targets --all-features -- -D warnings` and fix any issues.
   - Run `cargo llvm-cov --lib --bins | grep orbital_drydock` to ensure test coverage is over 85%.

5. **Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.**
   - Run `pre_commit_instructions` tool.
   - Follow formatting and other instructions.

6. **Submit**
   - Use `run_in_bash_session` with `sed` to move the task from `IN_PROGRESS.md` to `COMPLETED.md`.
   - Execute `git add .`.
   - Execute `git commit` providing the full multi-line final commit message exactly as outlined in the issue instructions.
   - Execute `git push`.
