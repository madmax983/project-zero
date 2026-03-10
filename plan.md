1. **Understand Seam**:
   - Source: `MachineRhythm` component, specifically `last_sync_bonus` in `src/layer1/tech/rhythm.rs`.
   - Sink: `Morale` component of Pops working on/at those machines.

2. **Identify Problem**:
   - `update_rhythm_system` calculates `last_sync_bonus` for `MachineRhythm` entities but it is never consumed or applied to workers near these machines to increase their morale.
   - We need an integration to bridge the gap.

3. **Bridge System Creation (`src/layer1/integration.rs`)**:
   - Create a new system `industrial_rhythm_morale_bridge` or similar.
   - It will query Pops `(Entity, &GridPosition, &mut Morale), With<Pop>`.
   - It will also query `(&GridPosition, &MachineRhythm)`.
   - Since `last_sync_bonus` is updated when the cycle ends, we might want to check the radius.
   - Actually, wait... the spec says: "Workers standing near high-rhythm machines should get `Effect::InTheZone` (Work Speed +20%)." But it also says: "High Rhythm boosts worker Morale (the satisfying "thrum" of efficiency) and may slightly improve production speed."
   - We can either:
     a) Apply a `MoodModifier` with name "Industrial Rhythm" when they are near.
     b) Apply it directly to `Morale`.
   - Let's look at how Morale works. Usually, a temporary `MoodModifier` is used.
   - Wait, `MachineRhythm.last_sync_bonus` is a persistent score on the building, updated every cycle end.
   - A better way: In `industrial_rhythm_morale_bridge`:
     - Iterate through Pops.
     - Find the maximum `last_sync_bonus` of machines within distance (e.g., distance <= 2).
     - If `max_bonus > 0.0`, apply a `MoodModifier` to the Pop's `Morale`.

4. **Red Phase (Integration Tests)**:
   - Create `tests/integration/rhythm_morale.rs` testing that a Pop near a synchronized `MachineRhythm` receives a Morale modifier.
   - Run tests (should fail).

5. **Green Phase**:
   - Implement `industrial_rhythm_morale_bridge`.
   - Register it in `src/layer1/systems/observation.rs` (or execution).

6. **Refactor Phase / Completion**:
   - Update `design/SEAM_MAP.md`
   - Complete task `INT-260` in `IN_PROGRESS.md` & `COMPLETED.md`.
