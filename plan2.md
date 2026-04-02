1. **Goal**: Fulfill the missing integration for `570` "The Bio-Acoustic Miasma" by linking `ParanoiaTracker` to `StressTracker` so paranoia hits can trigger mental breakdowns.

2. **Actions**:
   - In `src/layer1/integration.rs`, add a new bridge system: `pub fn paranoia_stress_bridge_system(mut query: Query<(&mut crate::layer1::stress::StressTracker, &mut crate::layer1::bio_acoustic_miasma::ParanoiaTracker)>) { ... }`
     - Inside, iterate over the query and add `paranoia.level as f32` to `stress.accumulated_stress`. Then set `paranoia.level = 0`.
   - In `src/layer1/systems/observation.rs`, register this bridge system in `Layer1SystemSet::Observation`:
     - Run `crate::layer1::integration::paranoia_stress_bridge_system.after(crate::layer1::bio_acoustic_miasma::broadcast_miasma_secrets).before(crate::layer1::stress::check_stress_breakdown_system)`
   - Create `tests/integration/bio_acoustic_miasma_bridge.rs` testing that `ParanoiaTracker` effectively increases `accumulated_stress` and can trigger a `Breakdown`.
     - The test should spawn an entity with `Needs`, `StressTracker`, `Traits`, and `ParanoiaTracker`. Run `paranoia_stress_bridge_system` and `check_stress_breakdown_system`, and assert that the entity gets a `Breakdown` component.
   - Register the test module in `tests/integration/mod.rs` if needed (seems to be picked up automatically usually, but let's check).
   - Update `design/SEAM_MAP.md` and `design/COMPLETED.md` with the new seam. Wait, `570` is already in `COMPLETED.md`. I should add `INT-570` to `design/COMPLETED.md` and `SEAM_MAP.md`.

3. **Pre-commit**:
   - Ask for pre-commit instructions and run tests, lint, etc.
