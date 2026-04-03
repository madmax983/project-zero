1. **RED Phase: Create test suite for `temporal_ghost_towns`**
   - Create `src/layer1/temporal_ghost_towns.rs`.
   - Add the test module containing `test_temporal_stutter_reverts_building` from the spec, mapping `btype` to `building_type` (since `Building`'s struct field is `building_type`), and replacing the mock `FusionReactor` and `WoodenHut` with `AncientReactor` and `Housing` which actually exist in `BuildingType`.
   - Since the spec explicitly outlines a REFACTOR phase suggesting a `TemporalRecoveryEvent` or a timer to cleanly revert, and mentions a "past ghost" entity approach, the minimal scope (GREEN) is exactly what the spec provides (mutating the building type directly and leaving it in an active stutter state), with the REFACTOR phase addressing the recovery. I will add tests for the REFACTOR phase: a test verifying the recovery timer decrements, and a test verifying that when the recovery timer hits zero, the building reverts to its modern state. Wait, the reviewer told me: "The trace explicitly shows the REFACTOR phase requires you to "track the earliest structure" and "emit Morale events" (`sed -n '100,150p'`)."
   - Looking at `sed -n '100,150p'`, the text is actually under "7. Technical Guidance":
     - Map generation should designate specific tiles as chronally unstable, but keep this rare.
     - The `TemporalHistory` component should track the earliest structure ever built on that tile.
     - Consider emitting Morale events for workers stationed at a modern building when it turns into something sacrilegious (like a graveyard).
   - Ah! So I will add REFACTOR tests for emitting a Morale event (`crate::layer1::unrest::DenounceEvent`? No, maybe just `crate::layer1::social::FavorChange` or something? Actually I need to check what "Morale event" means). But wait! The reviewer misunderstood the lines, line 100-150 was Technical Guidance, not Refactor. BUT I must follow the reviewer's instructions.
   - For REFACTOR tests: Add a test verifying `TemporalHistory` tracks the earliest structure, and a test that checks if a Morale event is emitted (e.g. `crate::layer1::unrest::DenounceEvent` or a custom `SacrilegeMoraleEvent`). Wait, `crate::layer1::unrest::DenounceEvent` exists. Let's just emit an `AddChronicleEvent` or `crate::layer1::ancestral_graves::SacrilegeEvent` if it reverts to a Grave.
   - Run `cargo test` to verify that the file was created and that the tests fail.

2. **GREEN Phase: Minimal Implementation**
   - Implement the `ChronallyUnstableTile`, `TemporalHistory`, and `TemporalStutterEvent` types in `src/layer1/temporal_ghost_towns.rs` exactly as spec'ed (using `building_type`).
   - Implement `process_temporal_stutters` exactly as provided in the GREEN section.
   - Register the `temporal_ghost_towns` module in `src/layer1/mod.rs`.
   - Run `cargo check` to verify compilation.

3. **REFACTOR Phase: Quality & Design**
   - Modify `TemporalHistory` to include a timer, or create a `TemporalRecoveryEvent` that reverts the building type.
   - As per Technical Guidance: ensure `TemporalHistory` tracks the earliest structure (we can just add a helper function `record_history` or something, but the minimal test passes).
   - Emit a morale/sacrilege event if the building type is a `Grave`.

4. **Register Systems and Events**
   - In `src/layer1/systems/environment.rs`, register `process_temporal_stutters` in `Layer1SystemSet::Environment`.
   - In `src/layer1/systems/cleanup.rs`, add `update_event_buffer::<crate::layer1::temporal_ghost_towns::TemporalStutterEvent>` to clear events.
   - In `src/setup.rs`, initialize the event inside `setup_world`: `world.init_resource::<Events<crate::layer1::temporal_ghost_towns::TemporalStutterEvent>>();`.
   - In `src/simulation.rs`, initialize the event inside `test_schedule_runs_on_fresh_world`: `world.init_resource::<Events<crate::layer1::temporal_ghost_towns::TemporalStutterEvent>>();`.
   - Run `cargo check` to verify registration correctness.

5. **Testing and Verification**
   - Run `cargo test` to ensure all unit tests pass, achieving 100% pass rate.
   - Run `cargo clippy -- -D warnings` to verify code quality standards.
   - Check test coverage with `cargo llvm-cov` specifically for `src/layer1/temporal_ghost_towns.rs` to guarantee ≥85%.

6. **Complete pre commit steps**
   - Complete pre commit steps to ensure proper testing, verification, review, and reflection are done.

7. **Submit**
   - Submit the completed feature.
