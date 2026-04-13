1. **Explore & Write `tests/integration/predatory_weather.rs`** (RED Phase):
   - Add two tests: one for checking that `predatory_weather_emission_bridge_system` sums up `PowerSource` and `HeatSource` and puts it into an `AggroTarget` in the Bevy ECS.
   - The other for checking that `predatory_weather_impact_bridge_system` catches a `StormImpactEvent` and applies damage to Layer 1 `Structure` components, plus sends an `AddChronicleEvent`.
2. **Implement Bridge Systems** (GREEN Phase):
   - In `src/layer1/core/integration.rs`, add:
     - `predatory_weather_emission_bridge_system`: Query `PowerSource` and `HeatSource` and update/insert a single `AggroTarget` in the world with the total emissions. If one doesn't exist, spawn it at `Vec2::new(0.0, 0.0)`.
     - `predatory_weather_impact_bridge_system`: Listen for `StormImpactEvent`. Iterate events, subtract damage from `Structure` components (all of them since `target` doesn't directly map to a specific building in Layer 1, or perhaps just to the first found structure or a random one, but "damages Layer 1 structures" suggests spreading or applying to something. Wait, the spec says "damages Layer 1 structures". We can iterate over all `Structure` components and apply damage / N, or a flat damage). Let's just deduct it from all, or one. Actually, "if a storm reaches the colony tile, emit a StormImpactEvent that damages Layer 1 structures." We'll just damage a random one or all of them slightly. Or better, we can iterate over all and apply the damage scaled down.
     - Also send `AddChronicleEvent` with `EventImportance::Major`.
3. **Register Bridge Systems**:
   - In `src/layer1/systems/observation.rs`, register them under `Layer1SystemSet::Observation` or similar.
4. **Pre-commit Instructions**:
   - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
5. **Submit**: Wait for all tests to pass.
