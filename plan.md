1. Add FactionId for CryoMutineers
   - Modify `src/layer1/social/factions.rs` to add `CryoMutineers` to `FactionId` enum.
2. Implement Integration Bridge for Cryo Mutiny
   - Add a bridge system in `src/layer2/integration.rs` to listen for new `MutineerPop` entities and assign them to `FactionId::CryoMutineers`.
   - Also, the bridge should emit an `AddChronicleEvent` to document the mutiny.
   - Register this system in `register_layer2_integration_systems` or appropriately in `src/simulation.rs`.
3. Write Tests
   - Add a test file `tests/integration/cryo_mutiny_bridge.rs` testing the bridge connection.
4. pre-commit
   - Complete pre-commit steps to make sure proper testing, verifications, reviews and reflections are done.
5. Submit
   - Submit the change to complete the task.
