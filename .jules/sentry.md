## 2025-04-25 - Haunted Assembly Lines Test Coverage
**Learning:** Writing tests for standalone ECS systems requires registering resources manually (like `Events<PopDiedInAccidentEvent>`) to avoid panics. When dealing with Bevy bundles or specific generic parameters in tests, it is critical to construct the component explicitly rather than just importing the generic definition (e.g. `Building { building_type: BuildingType::Farm }` instead of just `Building`).
**Action:** When writing tests that spawn entities and assign them, always look up the structure of required components (`AssignedTo`, `Building`) in their respective source files using `grep` or `cat` before writing the assertions, instead of assuming empty struct derivations exist.

## 2024-05-25 - TechState Corruption and Fallback coverage
**Learning:** `unwrap_or` calls when resolving active enum states (`TechStatus::Active`) and handling sorting logic (`partial_cmp` on floating points) within core manager structures like `TechState` can lack explicit tests.
**Action:** When finding `unwrap_or` on `partial_cmp` or fallback map fetches in manager structs, write targeted unit tests that construct edge-case states (like identically costed structs forcing a stable sort resolution, or fetching unknown keys) to explicitly prove the fallback works safely without panic.
