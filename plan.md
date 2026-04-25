1. **Explore & Review**: Read the `specs/1117-sonic-suppression.md` spec to understand the `Nausea` effects.
2. **Implement `Nausea` System**: Add `apply_nausea_penalties_system` in `src/layer1/sonic_suppression.rs`.
   - Iterate over `&mut Speed`, `&mut StressTracker`, and `&Nausea` for pops.
   - If `nausea.level > 0.0`:
     - Reduce `speed.current` by `0.5` (a movement speed debuff based on nausea level).
     - Add stress based on nausea level `stress_tracker.accumulated_stress += nausea.level * 0.1` (stress to own pops).
     - Slowly decay `nausea.level` over time (`nausea.level = (nausea.level - 1.0).max(0.0);`).
3. **Refactor Phase**:
   - Ensure a generic `Fragile` tag isn't explicitly required by tests, but use it if `GlassStructure` is too specific. Let's stick to `GlassStructure` and `Shattered` as specified by the test.
   - Add `DestroyBuildingEvent` if necessary, but just adding the `Shattered` component is what the RED phase tests for.
4. **Register System**: Register `apply_nausea_penalties_system` in `src/layer1/systems/execution.rs` to run alongside other modifier systems like `apply_chemical_speed_modifiers_system`.
5. **Tests & Lints**: Run `cargo test`, `cargo clippy`, and generate coverage (`cargo llvm-cov`) to ensure tests are green and coverage is >= 85%.
6. **Pre-commit**: Complete the `pre_commit_instructions` steps.
