**[AHash vs SipHash for Coordinates]
**Learning:** `std::collections::HashMap` uses SipHash, which is secure but slow for integer tuples like `(i32, i32)`. The codebase frequently accesses spatial structures using grid coordinates.
**Action:** Always prefer `bevy::utils::HashMap` and `bevy::utils::HashSet` (which use AHash) over the `std::collections` equivalents when indexing spatial data, especially on hot paths like pathfinding and AI lookups.
**[Optimizing HashMaps for Primitive Keys]**
**Learning:** For game engines and pathfinding systems using integer or tuple coordinates, `std::collections::HashMap` with default SipHash is excessively slow. Replacing it with `bevy::utils::HashMap` uses AHash which is significantly faster for primitive keys. Combining this with `.with_capacity(size)` entirely eliminates intermediate heap reallocations.
**Action:** Always favor `bevy::utils::HashMap::with_capacity` when accumulating grid coordinates during a system tick to minimize per-frame GC pressure and hashing overhead.
**[Optimizing HashMap Capacity from Queries]**
**Learning:** When using `bevy::utils::HashMap::with_capacity()` based on a Bevy `Query`, using `query.iter().count()` forces a full `O(N)` traversal of the ECS query, defeating the purpose of the optimization by iterating twice.
**Action:** Always use `query.iter().size_hint().0` to safely estimate the capacity of a collection built from an ECS query without triggering an additional traversal overhead.

## [UI Shell Optimization]
**Learning:** Avoid intermediate `.collect::<Vec<_>>()` when an iterator could be collected directly into a pre-allocated vector to prevent reallocation and copying, particularly in frequently executing queries or UI update loops.
**Action:** Replace `iter().filter(...).collect::<Vec<_>>()` chains where possible with `Vec::with_capacity(n)` and `.extend()` or directly applying `.collect()` if the target capacity can be accurately provided or bounds checked via `.size_hint()`.

## Removed `.clone()` of `work_designations` in `evaluate_work_and_taming`
**Learning:** Found unnecessary `.clone()` cloning large `work_designations` list in utility AI hot loop (`src/layer1/mind/utility_ai.rs`). Replaced with a direct reference slice in `evaluate_simple_action`, avoiding massive heap allocations for every pop eval cycle.
**Action:** Always check `Vec` clones that are passed to functions requiring slice `&[T]` or `&Vec<T>`.

## Pre-allocated vector capacity in `process_cargo_transfers_system`
**Learning:** Cargo contents drained into a dynamic vector in `src/layer1/core/integration.rs` triggered intermediate reallocations.
**Action:** Use `Vec::with_capacity(len)` when iterating over a known length collection via `.drain(..)`.

## [Performance Optimization: Pre-allocated String concatenation in UI render loop]
**Learning:** `get_status_string` was using `.collect::<String>()` from a map iterator to concatenate strings every frame during the UI render loop. This caused unnecessary memory reallocations since it did not pre-allocate enough capacity.
**Action:** Replaced the intermediate map-collect chain with a single loop that pushes strings to a `String` pre-allocated with `String::with_capacity(256)`. Also converted `.collect::<Vec<_>>()` when truncating lines to `Vec::with_capacity(size)` + manual loop to eliminate intermediate allocations in `status.rs`.
