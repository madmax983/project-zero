**Tuple Limit Tangle**
**Tangle:** Bevy's `add_systems()` macro has a maximum tuple size limit of 21 elements. `src/layer1/systems/observation.rs` exceeded this limit (22 items), breaking compilation.
**Blueprint:** Split the overloaded tuple in `add_systems()` into two separate tuples, safely maintaining the `.in_set(Layer1SystemSet::Observation)` schedule association for all included systems.

**Nature Sub-module Extracted**
**Tangle:** `src/layer1/mod.rs` was a 220+ file monolithic module containing environment, physics, weather, and basic survival systems deeply coupled with game logic.
**Blueprint:** Extracted 13 foundational logic modules (`terrain`, `water`, `weather`, `atmosphere`, `temperature`, `seasons`, `wind`, `erosion`, `fertility`, `solar`, `ecology`, `radioactive`, `fire`) into a new `src/layer1/nature` module, simplifying the top-level Layer 1 namespace and enforcing a stronger domain boundary around environmental physics.

**Tuple Limit Execution Tangle**
**Tangle:** Bevy's `add_systems()` macro has a maximum tuple size limit of 21 elements. `src/layer1/systems/execution.rs` exceeded this limit (22 items in one tuple), breaking compilation.
**Blueprint:** Split the overloaded tuple in `add_systems()` into two separate tuples, safely maintaining the `.in_set(Layer1SystemSet::Execution)` schedule association for all included systems.

**Secret Societies Encapsulation**
**Tangle:** The `SecretSocieties` logic (`src/layer1/society.rs`) was declared as a root-level module (`pub mod society`) directly under `layer1`, leaking social domain logic into the top-level namespace rather than being encapsulated within its functional domain.
**Blueprint:** Moved `src/layer1/society.rs` to `src/layer1/social/society.rs` and updated module declarations and imports. This enforces a stronger domain boundary by nesting the secret society mechanics entirely within the `social` subsystem.
**Geology Module Structural Tangle**
**Tangle:** The geology module was incorrectly split between `src/layer1/geology.rs` and `src/layer1/geology/tectonic.rs`, and tests were awkwardly injected via an `include!` hack in `src/layer1/mod.rs`.
**Blueprint:** Moved `src/layer1/geology.rs` to `src/layer1/geology/mod.rs` to establish a proper domain boundary and natively declared the test module in `tectonic.rs`.

**Building Blob Extracted**
**Tangle:** `src/layer1/building.rs` was a 3,300+ line monolithic module containing core structural logic, type definitions, inline tests, and placement logic deeply coupled together, exhibiting "The Blob" anti-pattern.
**Blueprint:** Extracted the file into a proper submodule `src/layer1/building/mod.rs` and moved the 1,000+ lines of test code into a separate `src/layer1/building/tests.rs` file. Flattened the `tests::tests` module structure in the new file to enforce a cleaner domain boundary without breaking downstream imports.
