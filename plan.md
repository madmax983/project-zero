1. **Bridge PopDied and Flora Placed events to Diplomatic Traits events**: Implemented `diplomatic_reflection_kill_bridge` and `diplomatic_reflection_plant_bridge` in `src/layer1/integration.rs`. Registered them in `src/layer1/systems/observation.rs`.
2. **Add integration tests**: Added tests in `tests/integration/diplomacy_reflection.rs` to verify that `PopDied` triggers `EntityKilledEvent` and new `Flora` triggers `FloraPlantedEvent`. Linked it in `tests/integration.rs`.
3. **Pre-commit**: Follow testing and formatting rules before submission.
4. **Submit**: Create PR to finalize INT-657 integration.
