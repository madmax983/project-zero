1. **Identify the core issue**:
   - `broadcast_miasma_secrets` correctly increments `ParanoiaTracker.level` in `ParanoiaTracker` components when miasma broadcasts a secret.
   - However, `ParanoiaTracker` is *never* wired to the actual stress/mental break logic in `src/layer1/stress.rs`. `check_stress_breakdown_system` doesn't know about `ParanoiaTracker` at all.
   - According to spec `570`: "Integration Points: Link ParanoiaTracker increases directly to the existing mental break threshold logic."

2. **Plan**:
   - Instead of modifying `check_stress_breakdown_system` directly to read `ParanoiaTracker`, which would make the generic stress system depend on the specific Miasma feature, I will create a bridge system in `src/layer1/integration.rs`.
   - The bridge system `paranoia_stress_bridge_system` will query `(&mut StressTracker, &mut ParanoiaTracker)`.
   - It will add `ParanoiaTracker.level` directly to `StressTracker.accumulated_stress`.
   - Then it will reset `ParanoiaTracker.level` to 0. (or some decay over time? No, the spec says "causing localized paranoia, spontaneous arrests, and massive morale hits" and "Link ParanoiaTracker increases directly to the existing mental break threshold logic" so it should probably just be an immediate addition to stress).
   - Alternatively, it could modify morale or just add to accumulated_stress directly. Since `ParanoiaTracker` represents a level that is being increased by +10 during broadcast, adding it directly to `accumulated_stress` effectively pushes them closer to mental break.
   - Let's look at `check_stress_breakdown_system`. It uses `tracker.accumulated_stress`. Adding `paranoia.level` to `tracker.accumulated_stress` and resetting `paranoia.level` to 0 will cause a spike in stress that can immediately trigger a breakdown if it exceeds `BREAKDOWN_TICKS_REQUIRED`.
   - The bridge system should be placed in `src/layer1/integration.rs` and registered in `src/layer1/systems/observation.rs` (or `execution.rs` depending on where the miasma runs).
   - Miasma broadcast runs in `Observation` set (`crate::layer1::bio_acoustic_miasma::broadcast_miasma_secrets`). The stress system also runs in `Observation` set (`check_stress_breakdown_system.after(decay_needs_system)`).
   - We will run the bridge system after `broadcast_miasma_secrets` and before `check_stress_breakdown_system`.
   - Then, write an integration test in `tests/integration/bio_acoustic_miasma_bridge.rs` (or similar).

3. **Pre-commit**: Follow pre-commit instructions.
