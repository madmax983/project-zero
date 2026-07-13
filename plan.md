1. **Integration Implementation (`INT-1006: Terraforming Rejection -> Disasters / Chronicle`)**
    - Use `run_in_bash_session` with `cat << 'EOF' >> src/layer1/core/integration.rs` to append the bridge function `terraforming_rejection_chronicle_bridge` to `src/layer1/core/integration.rs`. The bridge will listen for `DisasterEvent` and if the `disaster_type` is `DisasterType::Fissure`, it will emit an `AddChronicleEvent` indicating an autoimmune response from the planet.
    - Use `run_in_bash_session` with `cat src/layer1/core/integration.rs` to verify the bridge was added correctly.

2. **Schedule and Resource Registration**
    - Use `run_in_bash_session` with `sed -i '/world\.init_resource::<crate::layer1::social::politics::ActiveMandate>();/a \    world.insert_resource(crate::layer1::terraforming_rejection::PlanetaryStress { value: 0.0, threshold: 100.0 });' src/setup.rs` to initialize `PlanetaryStress` in `setup_world`.
    - Use `run_in_bash_session` with `sed -i '/world\.init_resource::<bevy_ecs::event::Events<crate::layer1::tech::temporal_smuggling::PayTemporalDebtEvent>>();/a \    world.init_resource::<bevy_ecs::event::Events<crate::layer1::terraforming::TerraformEvent>>();' src/setup.rs` to initialize `Events<TerraformEvent>` in `setup_world`. (Wait, let me just add it to setup_world to be safe). Let's use specific line numbers or better patterns.
    - Let's use Python to safely append the registration to `src/setup.rs`, `src/simulation.rs`, `src/layer1/systems/environment.rs` and `src/layer1/systems/observation.rs`.
    - `src/simulation.rs`: Insert `world.init_resource::<bevy_ecs::event::Events<crate::layer1::terraforming::TerraformEvent>>();` inside `run_simulation_tick` around line 557.
    - `src/layer1/systems/environment.rs`: Insert `crate::layer1::terraforming_rejection::apply_terraforming_stress_system, crate::layer1::terraforming_rejection::trigger_autoimmune_response_system,` after `crate::layer1::environment::terraforming::apply_planetary_effects_system` (around line 238).
    - `src/layer1/systems/observation.rs`: Insert `crate::layer1::core::integration::terraforming_rejection_chronicle_bridge,` after `crate::layer1::core::integration::cassandra_syndrome_chronicle_bridge,` (around line 601).
    - Use `run_in_bash_session` with `echo "- [ ] \`INT-1006\` Integration: Terraforming Rejection -> Disasters / Chronicle - claimed $(date +%Y-%m-%d)" >> design/IN_PROGRESS.md`
    - Use `run_in_bash_session` with `echo "- [x] \`INT-1006\` Integration: Terraforming Rejection -> Disasters / Chronicle - completed $(date +%Y-%m-%d)" >> design/COMPLETED.md`
    - Use `run_in_bash_session` with `cat << 'EOF' >> design/SEAM_MAP.md` to append the INT-1006 documentation.

3. **Integration Tests**
    - Use `run_in_bash_session` with `cat << 'EOF' > tests/integration/terraforming_rejection_chronicle_bridge.rs` to create the test file.
    - Use `run_in_bash_session` with `cat tests/integration/terraforming_rejection_chronicle_bridge.rs` to confirm it was written correctly.
    - Use `run_in_bash_session` with `echo '#[path = "integration/terraforming_rejection_chronicle_bridge.rs"] pub mod terraforming_rejection_chronicle_bridge;' >> tests/integration.rs` to register the test correctly.

4. **Run all tests**
    - Use `run_in_bash_session` to run `cargo test` and verify that the integration is correct and no regressions were introduced.

5. **pre-commit**
    - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done by calling `pre_commit_instructions`.

6. **Submit**
    - Commit and submit.
