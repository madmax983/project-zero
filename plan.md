1.  **Refactor `TectonicStress` in `src/layer1/geology/tectonic.rs`**:
    *   Add logic to emit `DamageEvent` for all `Structure`s when a `MegaQuakeEvent` is triggered, as per the REFACTOR section of the spec: "Hook MegaQuake into the `DamageEvent` pipeline for all structures."

2.  **Add `apply_mega_quake_damage_system`**:
    *   Create a new system `apply_mega_quake_damage_system` that reads `MegaQuakeEvent`, iterates over all entities with a `Structure` component (or `Health`), and sends a `DamageEvent` to them.
    *   Register this new system in `src/layer1/systems/environment.rs` right after `check_quake_system`.

3.  **Ensure tests pass and maintain coverage**:
    *   Write a unit test in `src/layer1/geology/tectonic.rs` to verify that `DamageEvent`s are correctly sent to structures when a `MegaQuakeEvent` occurs.
    *   Ensure all tests pass and coverage is above 85%.

4.  **Pre-commit steps**:
    *   Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.

5.  **Submit Changes**:
    *   Submit the completed work following standard commit practices and update `BACKLOG.md`, `IN_PROGRESS.md`, and `COMPLETED.md`.
