**Tuple Limit Tangle**
**Tangle:** Bevy's `add_systems()` macro has a maximum tuple size limit of 21 elements. `src/layer1/systems/observation.rs` exceeded this limit (22 items), breaking compilation.
**Blueprint:** Split the overloaded tuple in `add_systems()` into two separate tuples, safely maintaining the `.in_set(Layer1SystemSet::Observation)` schedule association for all included systems.