1. **Claim the Task**: Use `run_in_bash_session` to run a python script to append `- [ ] \`INT-1306\` Integration: Negative Events -> Architectural Superstition` to `design/IN_PROGRESS.md` and then use `run_in_bash_session` to commit the claim.
2. **RED Phase (Tests First)**:
   - Use `write_file` to create `tests/integration/architectural_superstition_bridge.rs` testing that when `PopDied`, `BuildingRemovedEvent`, or `PopDiedInAccidentEvent` are fired, nearby buildings have `NegativeEvent` added to their `NegativeEventHistory`.
   - Use `run_in_bash_session` to append `#[path = "integration/architectural_superstition_bridge.rs"] mod architectural_superstition_bridge;` to `tests/integration.rs` via a Python script.
   - Use `run_in_bash_session` to run `cargo test --test integration architectural_superstition_bridge` to verify it fails.
3. **GREEN Phase (Implementation)**:
   - Use `run_in_bash_session` to run a python patch script that injects `track_negative_events_bridge_system` into `src/layer1/core/integration.rs`. The system will listen to `PopDied` (resolving location via `GridPosition`), `BuildingRemovedEvent` (using `.position`), and `PopDiedInAccidentEvent` (using `.location` to lookup `GridPosition`), and append `NegativeEvent` to buildings within a radius of 5 tiles.
   - Use `run_in_bash_session` to run a python patch script to register the system in `src/layer1/systems/observation.rs` before `evaluate_architectural_superstition`.
4. **Verification Step**: Use `run_in_bash_session` to run `git diff` to confirm the edits to `src/layer1/core/integration.rs` and `src/layer1/systems/observation.rs` were applied correctly.
5. **Update Maps**: Use `run_in_bash_session` with a python script to update `design/SEAM_MAP.md` documenting the new bridge and to move the task to `design/COMPLETED.md`.
6. **Testing**: Use `run_in_bash_session` to run `cargo test --lib` and `cargo test --test integration` to ensure everything passes and the test is now green, and no regressions exist.
7. **Pre-commit**: Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
8. **Submit**: Use `default_api:submit` to submit the changes. The commit message must be exactly:
`feat(integration): connect negative events to architectural superstition`
with the description: `Co-Authored-By: google-labs-jules[bot] <161369871+google-labs-jules[bot]@users.noreply.github.com>`.
