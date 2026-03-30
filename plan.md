1. **Remove Duplicate Event**
   - Delete `ExplosionEvent` from `src/layer1/environment/ignition.rs`.
   - Update `process_ignition` to import and use `crate::layer1::volatile::ExplosionEvent`.
   - Map properties: `position` -> `center`, `damage` -> `damage`, set `radius: 1` or whatever is appropriate (spec says "detonates in a massive Air-Burst explosion... destroying nearby structures". A radius of `1` matches Chebyshev distance 1, hitting all 8 adjacent tiles).

2. **Clean up `src/simulation.rs` & `src/layer1/systems/cleanup.rs`**
   - Remove redundant `app.add_event::<ExplosionEvent>()` in `ignition.rs` tests.
   - Remove redundant initializations of `Events<crate::layer1::environment::ignition::ExplosionEvent>` in `src/simulation.rs`.

3. **Bridge System for Chronicle**
   - Create `ignition_chronicle_bridge` in `src/layer1/integration.rs` which reads `SparkEvent` and queries `VolatileVapor`. If a spark overlaps with vapor, it emits `AddChronicleEvent` indicating an industrial disaster.
   - Register this system in `src/layer1/systems/execution.rs` ensuring it runs `.before(crate::layer1::environment::ignition::process_ignition)` so the vapor is still present.
     Wait, actually, I can just register it in `execution.rs` where `process_ignition` is:
     ```rust
     schedule.add_systems((
         crate::layer1::integration::ignition_chronicle_bridge,
         crate::layer1::environment::ignition::process_ignition.after(crate::layer1::integration::ignition_chronicle_bridge),
     )...
     ```
     This perfectly fits the Integrator style without changing builder logic.

4. **Integration Tests**
   - Create `tests/integration/atmospheric_ignition.rs`.
   - Ensure the test covers `ignition_chronicle_bridge`, `process_ignition`, and `handle_explosion_system` running together.

5. **Complete pre-commit steps**
   - Run tests (`cargo test`) and check format/clippy.
   - Ensure proper testing, verification, review, and reflection are done.

6. **Submit**
   - Update `design/IN_PROGRESS.md` or `COMPLETED.md`.
   - Commit and submit.
