1. **Understand Spec 281 (Atmospheric Feedback):** The spec requires heavy industry to generate Smog, trees to absorb it, and Smog to reduce solar panel efficiency. Refactoring dictates using a spatial grid (`AtmosphereGrid`).
2. **Current State:**
    - `AtmosphereGrid` already exists and tracks pollution.
    - `update_atmosphere_system` adds pollution from `BuildingType::Refinery`, `AncientReactor`, `Smelter`, `Generator`, `Smithy`. This serves as the smog generation part.
    - `simulate_diffusion_system` diffuses pollution over the grid.
    - `apply_smog_damage_system` damages health.
    - `update_solar_output_system` in `solar.rs` calculates solar panel output based on `SolarCycleState` and `DayNightCycle`.
3. **Implementation Plan:**
    - **Trees Absorb Smog:** Modify `update_atmosphere_system` or create an `absorb_smog_system` in `src/layer1/nature/atmosphere.rs` to have `TerrainType::Tree` tiles absorb pollution from the `AtmosphereGrid` at their position. (Requires reading `TerrainGrid`).
    - **Smog Reduces Solar Efficiency:** Modify `update_solar_output_system` in `src/layer1/nature/solar.rs`. Query for `&GridPosition` alongside `&mut PowerSource, &SolarPower`. Use `AtmosphereGrid.get(pos.x, pos.y)` to find local smog. Apply a penalty: e.g., `let penalty = (smog / 100.0).clamp(0.0, 0.5); total_modifier *= (1.0 - penalty);`.
    - **Add tests** in `src/layer1/nature/atmosphere_tests.rs` or inline to verify tree absorption and solar penalty based on local grid pollution instead of a global resource.
4. **Pre-commit checks:** Run `cargo test` and `cargo clippy`.
