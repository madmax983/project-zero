1. **RED Phase: Create test suite for `temporal_ghost_towns`**
   - Create `src/layer1/temporal_ghost_towns.rs`.
   - Add the test module containing `test_temporal_stutter_reverts_building` based on the exact spec definition, making adjustments for actual `BuildingType` values (e.g. `AncientReactor` instead of `FusionReactor` and `Housing` instead of `WoodenHut`, since they exist in the `BuildingType` enum from exploration).
   - Add REFACTOR phase tests verifying that a recovery mechanism works and restores the building's original state.
   - Run `cargo test` to verify that the file was created and that the tests fail, as required by the TDD RED phase.

2. **GREEN Phase: Minimal Implementation**
   - Implement the `ChronallyUnstableTile`, `TemporalHistory`, and `TemporalStutterEvent` types in `src/layer1/temporal_ghost_towns.rs` exactly as spec'ed (using `building_type` to match `Building`).
   - Implement `process_temporal_stutters` to mutate the building type and set `active_stutter = true`.
   - Add refactored fields (e.g. `recovery_ticks: u64` and `modern_building_type: BuildingType`) to `TemporalHistory` to keep track of state.
   - Implement `recover_temporal_stutters` system that decrements `recovery_ticks` while active, and reverts `building_type` when it hits 0.
   - Register the `temporal_ghost_towns` module in `src/layer1/mod.rs` (`pub mod temporal_ghost_towns; pub use temporal_ghost_towns::*;`).
   - Run `cargo check` to verify compilation.

3. **Register Systems and Events**
   - In `src/layer1/systems/environment.rs`, register `process_temporal_stutters` and `recover_temporal_stutters` in `Layer1SystemSet::Environment`.
   - In `src/layer1/systems/cleanup.rs`, add `update_event_buffer::<crate::layer1::temporal_ghost_towns::TemporalStutterEvent>` to clear events.
   - In `src/setup.rs`, initialize the event by adding `world.init_resource::<Events<crate::layer1::temporal_ghost_towns::TemporalStutterEvent>>();` inside the `setup_world` function.
   - In `src/simulation.rs`, initialize the event by adding `world.init_resource::<Events<crate::layer1::temporal_ghost_towns::TemporalStutterEvent>>();` inside the `test_schedule_runs_on_fresh_world` function.
   - Run `cargo check` to verify registration correctness.

4. **Testing and Verification**
   - Run `cargo test` to ensure all unit tests pass, achieving 100% pass rate.
   - Run `cargo clippy -- -D warnings` to verify code quality standards.
   - Check test coverage with `cargo llvm-cov` specifically for `src/layer1/temporal_ghost_towns.rs` to guarantee ≥85%.

5. **Complete pre commit steps**
   - Complete pre commit steps to ensure proper testing, verification, review, and reflection are done.

6. **Submit**
   - Submit the completed feature.
