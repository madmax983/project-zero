1. **Create `src/layer1/physics/kinetic_strike.rs`**:
   - Implement `KineticStrikeEvent` containing `target_x: i32`, `target_y: i32`, and `accuracy_offset: f32`.
   - Implement `kinetic_strike_system` as specified in the RED/GREEN phase, modified to interface with actual `TerrainGrid` and `Building` structures (using `TerrainType` and `ResourceType`).
   - Handle the target coordinate logic using `TerrainGrid`. Specifically, hitting an `Ore` resource turns the tile into a crater (e.g. `TerrainType::Dirt` if `Crater` isn't available, but we might just use `TerrainType::Dirt` or whatever closest approximation the spec expects, although the RED phase explicitly uses `TerrainType::Crater`). Since `TerrainType::Crater` does not exist, I will use `TerrainType::Dirt` for crater and spawn `ResourceItem` or just follow the RED phase using an isolated `TerrainTile` component for the sake of strict TDD, and then in GREEN phase hook it up to real components. Wait, the Architect prompt insight says "Never guess on unclear specs" but we must implement the RED phase *exactly* as written, then adapt. Actually, the spec RED phase uses `TerrainTile`, `TerrainKind`, `SubsurfaceResource`, `ResourceKind`. I will implement the exact RED and GREEN code from the spec into `src/layer1/physics/kinetic_strike.rs`.
   - Add tests from RED phase to the file.

2. **Refactor Phase (`src/layer1/physics/kinetic_strike.rs`)**:
   - In REFACTOR, the spec states: "The current target resolution relies on exact coordinates. An actual kinetic strike should probably have a radius effect... Add logic to destroy any Building components on the affected tiles. Trigger a ChronicleEvent..."
   - Update `kinetic_strike_system` to apply a radius effect and destroy `Building` components.
   - Trigger `AddChronicleEvent`.

3. **Register module and events**:
   - Add `pub mod kinetic_strike;` and `pub use kinetic_strike::*;` to `src/layer1/physics/mod.rs`.
   - Register `KineticStrikeEvent` in `src/setup.rs` or `src/simulation.rs`.
   - Register `kinetic_strike_system` in `src/layer1/systems/execution.rs`.

4. **Verify coverage and style**:
   - Run `cargo llvm-cov` on `scale::layer1::physics::kinetic_strike`.
   - Run `cargo fmt`, `cargo check`, `cargo clippy`.

5. **Update task boards**:
   - Move task `1083` from BACKLOG.md to IN_PROGRESS.md to COMPLETED.md.

6. **Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done**.
   - Review and test before commit.

7. **Submit change**.
