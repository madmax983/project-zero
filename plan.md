1. **Add `process_biomass_tariff_system` (Spec 548) to Simulation Schedule:**
   - Register `TradeDeal` event using `init_resource::<Events<crate::layer2::trade::biomass_tariff::TradeDeal>>()` in `run_simulation_tick`.
   - Add `crate::layer2::trade::biomass_tariff::process_biomass_tariff_system` to `build_simulation_schedule` in `src/simulation.rs`.
   - Update `test_schedule_runs_on_fresh_world` to initialize `Events<TradeDeal>`.
2. **Add `enforce_resolutions_system` (Spec 469) to Simulation Schedule:**
   - Register `GalacticCouncil` resource using `init_resource::<crate::layer3::council::GalacticCouncil>()` in `run_simulation_tick`.
   - Add `crate::layer3::council::enforce_resolutions_system` to `build_simulation_schedule` in `src/simulation.rs`.
   - Update `test_schedule_runs_on_fresh_world` to initialize `GalacticCouncil`.
3. **Verify Disconnected/Missing Seams Check:**
   - Review `src/layer2/trade/biomass_tariff.rs` and `src/layer3/council.rs` tests. Both appear isolated, but we should make sure they are connected in `src/simulation.rs` to ensure the systems actually run during the game loop.
4. **Complete Pre-Commit Steps**
   - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
5. **Submit**
   - Submit the change with an appropriate commit message.
