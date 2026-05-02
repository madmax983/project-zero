1. **Explore and Identify Disconnected Seams**: Look for completed features that aren't integrated into the schedule or don't trigger their expected effects. I've identified four:
    - INT-1235: The Gastronomers -> Chronicle
    - INT-1248: Red Tape Defense -> Simulation Schedule
    - INT-636: Orphan Fleet -> Simulation Schedule
    - INT-1082: Resonant Architecture -> Simulation Schedule
2. **Implement Integration Glue**:
    - `INT-1235`: `gastronomer_chronicle_bridge` was already added, need to mark it as complete.
    - `INT-1248`: Ensure `process_bureaucracy_delays` from `red_tape_defense.rs` is added to `SimulationSchedule` in `simulation.rs`.
    - `INT-636`: Ensure orphan fleet systems are added to `SimulationSchedule`.
    - `INT-1082`: Ensure `apply_resonant_architecture_system` is added to `SimulationSchedule`.
3. **Write Integration Tests**: Add integration tests for `INT-1248`, `INT-636`, and `INT-1082` to verify correct system ordering and logic. Add them to `tests/integration.rs`.
4. **Update Trackers**: Update `design/IN_PROGRESS.md`, `design/COMPLETED.md`, and `design/SEAM_MAP.md` to reflect these integrations.
5. **Run Pre-Commit Checks**: Complete pre commit steps to make sure proper testing, verifications, reviews and reflections are done.
6. **Submit**: Create a PR with all the integration updates.

*(I've already implemented steps 1-4 and verified everything passes.)*
