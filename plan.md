1. **Claim the integration task in `design/IN_PROGRESS.md`**:
   - Add `INT-1211 Integration: Heavy Industry -> Tectonic Stress`.

2. **RED Phase (Integration Test)**:
   - Create `tests/integration/tectonic_heavy_industry_bridge.rs`.
   - Write a test verifying that entities with `crate::layer1::environment::atmosphere::HeavyIndustry` increase the global `TectonicStress` resource over time.

3. **GREEN Phase (Implementation)**:
   - In `src/layer1/core/integration.rs`, add a new system `heavy_industry_tectonic_stress_bridge`:
     ```rust
     pub fn heavy_industry_tectonic_stress_bridge(
         mut stress: ResMut<crate::layer1::geology::tectonic::TectonicStress>,
         industry_query: Query<&crate::layer1::environment::atmosphere::HeavyIndustry>,
     ) {
         for industry in industry_query.iter() {
             // Each tick, heavy industry adds stress proportional to its size/smog output
             stress.current += industry.smog_output * 0.05;
         }
     }
     ```
   - Register this system in `src/layer1/systems/environment.rs` in the `Update` schedule, perhaps near `update_stress_system`. (Actually let's check `src/layer1/systems/environment.rs` to see where `update_stress_system` is registered).

4. **Run Pre-commit steps**:
   - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.

5. **Update Documentation & Complete**:
   - Update `design/SEAM_MAP.md` with the new INT-1211 seam.
   - Move the task to `design/COMPLETED.md`.
   - Submit the PR.
