1. **Explore & Analyze**:
   - I have explored the `sub_lithic.rs` implementation and identified `SabotageEvent`.
   - `SabotageEvent` is an Event Black Hole (sent but no listener).

2. **Integration Goal (INT-783)**:
   - Create a bridge system `sub_lithic_sabotage_bridge` in `src/layer1/core/integration.rs` that reads `SabotageEvent`.
   - Apply damage to the target entity's `Health`.
   - Emit an `AddChronicleEvent` to document the sabotage by the Sub-Lithic Cult.
   - Register the system in `src/layer1/systems/observation.rs` (after `update_event_buffer::<SabotageEvent>` and before execution maybe? Or just `register` it in `observation.rs`).

3. **RED Phase (Tests)**:
   - Write `tests/integration/sub_lithic_sabotage_bridge.rs` (and add it to `tests/integration/mod.rs`).
   - Setup a world with `SabotageEvent`, `Health`, `AddChronicleEvent`.
   - Test that `Health` decreases when a sabotage event is emitted and that `AddChronicleEvent` is dispatched.

4. **GREEN Phase (Implementation)**:
   - Implement `sub_lithic_sabotage_bridge` in `src/layer1/core/integration.rs`.
   - Insert it into `src/layer1/systems/observation.rs`.

5. **REFACTOR / Verification**:
   - Update `design/IN_PROGRESS.md` to claim `INT-783`.
   - Update `design/SEAM_MAP.md` to document the integration.
   - Move from `design/IN_PROGRESS.md` to `design/COMPLETED.md`.
   - Run `cargo test` and `cargo clippy`.

6. **Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done**.
