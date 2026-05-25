1. **Claim the task**:
   - Move `1126` from `BACKLOG.md` to `IN_PROGRESS.md`
   - Commit and push the change.
2. **Implement RED Phase (Tests)**:
   - Create `src/layer2/orbital_drydocks.rs` (or add to `src/layer2/station.rs` or a new module `src/layer2/orbital_drydock.rs` depending on standard). Actually, let's create `src/layer2/orbital_drydock.rs`.
   - Add tests based on the spec, modifying `ResourceStack` to `CargoStack`.
   - Run tests to see them fail.
3. **Implement GREEN Phase (Implementation)**:
   - Add `OrbitalDrydock` to `StationType` enum in `src/layer2/station.rs`.
   - Implement `cost`, `label`, `char` for `OrbitalDrydock`.
   - Implement `ShipConstruction` component and `process_drydock_construction_system` in `src/layer2/orbital_drydock.rs`.
   - Add module to `src/layer2/mod.rs`.
   - Make tests pass.
4. **Implement REFACTOR Phase**:
   - Ensure the code handles types properly and logic is sound.
   - Run clippy and tests.
5. **Pre-commit**:
   - Ensure proper testing, verification, review, and reflection are done by calling `pre_commit_instructions`.
6. **Complete task**:
   - Move `1126` from `IN_PROGRESS.md` to `COMPLETED.md`.
   - Commit using the required `submit` tool.
