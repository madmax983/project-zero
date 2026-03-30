1. **Claim the Task**: Move task `541` (Xeno-Artifacts) from `design/BACKLOG.md` to `design/IN_PROGRESS.md` and commit.
2. **RED Phase**:
   - Add failing tests to `src/layer1/artifacts/mod.rs` to verify that `Artifact` tiles spawn `Artifact` entities with `Aura` components via a startup system, and that they apply their specific aura effects.
   - Add a failing test to `src/layer1/execution/tests/work_tests.rs` to verify `TerrainType::Artifact` is indestructible and ignored by mining logic.
3. **GREEN Phase**:
   - Update `src/layer1/nature/terrain.rs`: Add `TerrainType::Artifact` to the `TerrainType` enum and update its associated methods (`name`, `movement_cost`, `is_walkable`, `heat_retention`). Update `generate_terrain` to randomly place a few `Artifact` tiles on the grid. Verify changes using `read_file`.
   - Update `src/ui/map.rs`: Add rendering support by updating `get_terrain_char`, `get_terrain_color`. Update `src/bin/headless.rs` to handle `TerrainType::Artifact` in `get_terrain_color_headless`. Verify changes using `read_file`.
   - Update `src/layer1/artifacts/mod.rs`: Create a startup system `spawn_artifacts_from_grid_system` that reads `TerrainGrid`, finds cells with `TerrainType::Artifact`, and spawns an entity with `Artifact`, `GridPosition`, and `Aura`. Register this system in `src/simulation.rs`. Verify changes using `read_file`.
   - Update `src/layer1/execution/mining.rs`: Update `handle_mining_work` to return early or ignore positions with `TerrainType::Artifact`. Update `src/layer1/designation.rs` to reject `DesignationType::Mine` on `TerrainType::Artifact` cells. Verify changes using `read_file`.
4. **REFACTOR Phase**:
   - Ensure the aura effects like `StressModifier` or `SkillXpBoost` are configured correctly when spawning artifact entities.
   - Run `read_file` to verify any refactoring changes made.
5. **Run Tests**: Run `cargo test` and `cargo clippy -- -D warnings` locally to confirm all tests pass. Check test coverage with `cargo llvm-cov`.
6. **Pre commit checks**: Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
7. **Submit**: Move task `541` from `design/IN_PROGRESS.md` to `design/COMPLETED.md` and commit.
