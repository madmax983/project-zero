1. Add task `- [ ] `INT-1306` Integration: Negative Events -> Architectural Superstition` to `design/IN_PROGRESS.md`.
2. Commit "claim: INT-1306 negative events to superstition integration"
3. RED PHASE: Write integration test `tests/integration/architectural_superstition_bridge.rs`. It tests `track_negative_events_bridge_system` connects `PopDied`, `BuildingRemovedEvent`, and `PopDiedInAccidentEvent` to `NegativeEventHistory`. Ensure it fails (RED).
4. GREEN PHASE: Implement `track_negative_events_bridge_system` in `src/layer1/core/integration.rs`. Register it in `src/simulation.rs` or `src/layer1/systems/observation.rs`.
5. Run tests. Ensure tests pass (GREEN).
6. Update `design/SEAM_MAP.md` and move task to `COMPLETED.md`.
7. Pre-commit instructions and run `cargo test --lib && cargo test --test integration`.
