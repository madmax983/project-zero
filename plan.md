1. **Explore & Verify Scope**: Read `specs/1309-the-asteroid-hermits.md` and check `src/layer2/asteroid_hermits.rs` for outputs like `Added<HermitOutpost>` and `DiscoveryEvent` to ensure they are available for integration.
2. **Claim Task**: Update `design/IN_PROGRESS.md` with `INT-1309` and commit it.
3. **Write RED Tests**: Create `tests/integration/asteroid_hermits_bridge.rs` testing that `Added<HermitOutpost>` and `DiscoveryEvent` successfully emit `AddChronicleEvent`. Register the test in `tests/integration.rs`.
4. **Implement Glue (GREEN)**: Create `asteroid_hermit_exodus_chronicle_bridge` and `asteroid_hermit_discovery_chronicle_bridge` in `src/layer2/integration.rs`. Register these systems in `src/layer2/systems/observation.rs` or `src/simulation.rs` (checking how other layer 2 integrations are registered). Ensure `Events<DiscoveryEvent>` is initialized in `src/simulation.rs`.
5. **Pre-commit Checks**: Complete pre-commit steps to make sure proper testing, verifications, reviews and reflections are done.
6. **Update SEAM_MAP and COMPLETED**: Move `INT-1309` to `design/COMPLETED.md` and update `design/SEAM_MAP.md` describing the integration.
7. **Submit**: Commit and submit the code.
