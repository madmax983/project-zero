1. **Create RED Phase Tests:**
   - Create `src/layer1/anomalies/benevolent_malfunctions.rs`.
   - Write tests: `test_benevolent_malfunction_increases_output`, `test_repairing_removes_malfunction`, `test_malfunction_quirk_applies_penalty`.

2. **GREEN Phase Minimal Implementation:**
   - Define `BenevolentMalfunction` component with `efficiency_bonus: f32` and `quirk: MalfunctionQuirk`.
   - Define `MalfunctionQuirk` enum (`ExcessHeat`, `LoudNoise`, `Unstoppable`).
   - Implement `apply_malfunction_effects` to add efficiency bonuses.
   - Implement `apply_malfunction_quirks` to add `HeatSource` or `NoiseSource` to buildings depending on the quirk.
   - We need to hook into the repair logic. `process_repair` in `src/layer1/architecture/structure.rs` handles repairs.
   - Actually, wait, `process_repairs` in the spec's green phase uses `RepairJobTarget` which doesn't exist. I should look into how repairs actually work in the codebase.
