1. **ColonyResources Update**:
   - In `src/layer1/economy/resources.rs`, update `ColonyResources` to fully support `HyperValuable`.
     - Add `hyper_valuable: f32` and `max_hyper_valuable: f32`.
     - Update `default()` to initialize `hyper_valuable` (0.0) and `max_hyper_valuable` (50.0).
     - Update `Mul<f32>` to multiply `hyper_valuable`.
     - Implement `add_hyper_valuable` helper function.
     - Update `consume`, `get_amount`, `has_room_for`, and `add_resource` to match the behavior of other resources.

2. **Ransom Broker Implementation**:
   - The file `src/layer1/ransom_broker.rs` is already created and tested, using `Metal` for testing purposes as `HyperValuable` wasn't working.
   - Update `src/layer1/ransom_broker.rs` to use `ResourceType::HyperValuable` instead of `Metal`.
   - Update the test to use `HyperValuable`.

3. **System Registration**:
   - In `src/layer1/systems/economy.rs`, import and register `ransom_demand_system` and `process_ransom_decisions_system` inside the `schedule.add_systems` block (in `Layer1SystemSet::Economy`).
   - In `src/layer1/systems/cleanup.rs`, register the `update_event_buffer` for `RansomDemandEvent`, `PayRansomEvent`, and `RefuseRansomEvent`.

4. **Resource/Event Initialization**:
   - In `src/simulation.rs`, register the new events with `world.init_resource::<Events<RansomDemandEvent>>()`, `Events<PayRansomEvent>`, and `Events<RefuseRansomEvent>` in `init_simulation_resources`.

5. **Pre-commit step**:
   - Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.

6. **Submit**:
   - Run `cargo test` and `cargo llvm-cov` to verify coverage.
   - `submit` branch.
