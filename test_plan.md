1. **Refactor `inspector_outcome_bridge_system` in `src/layer1/core/integration.rs`**
   - Extract the memory application into a helper function `apply_inspector_memory` to remove the duplicated `pop_memories.par_iter_mut().for_each` blocks.
2. **Refactor `public_grievance_grudge_bridge` in `src/layer1/core/integration.rs`**
   - Extract the logic of adding/updating a grudge into a helper function `upsert_grudge` to remove the duplicated loop logic for existing and new entities.
3. **Refactor `track_negative_events_bridge_system` in `src/layer1/core/integration.rs`**
   - Refactor nested `for` loops by using `.iter().filter().for_each` or keeping the logic flatter. Wait, the logic is okay but can be slightly cleaner.
4. **Refactor `fleet_unload_system` in `src/layer1/core/integration.rs`**
   - Extract the logic of unloading an individual stack to a helper or just simplify.
