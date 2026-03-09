**Tuple Limit Tangle**
**Tangle:** Bevy's `add_systems()` macro has a maximum tuple size limit of 21 elements. `src/layer1/systems/observation.rs` exceeded this limit (22 items), breaking compilation.
**Blueprint:** Split the overloaded tuple in `add_systems()` into two separate tuples, safely maintaining the `.in_set(Layer1SystemSet::Observation)` schedule association for all included systems.

**Nature Sub-module Extracted**
**Tangle:** `src/layer1/mod.rs` was a 220+ file monolithic module containing environment, physics, weather, and basic survival systems deeply coupled with game logic.
**Blueprint:** Extracted 13 foundational logic modules (`terrain`, `water`, `weather`, `atmosphere`, `temperature`, `seasons`, `wind`, `erosion`, `fertility`, `solar`, `ecology`, `radioactive`, `fire`) into a new `src/layer1/nature` module, simplifying the top-level Layer 1 namespace and enforcing a stronger domain boundary around environmental physics.
