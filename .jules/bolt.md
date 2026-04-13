**Removed O(n) Vec::contains overhead from hot fire loops**
**Learning:** Found an accidental O(n^2) scaling loop via `Vec::contains()` inside `fire_spread_system` and `fire_damage_system`. When a single tick spawns many fires, iterating through a Vec to check uniqueness scales poorly.
**Action:** Replaced `Vec` with `bevy_utils::HashSet` to eliminate O(n) lookups. By defaulting to `.insert()`, uniqueness is guaranteed in O(1) time without manual `.contains()` checks.

**Direct Entity Lookup vs HashMap Aggregation**
**Learning:** In Bevy ECS, aggregating data from multiple components into an intermediate `std::collections::HashMap` grouped by a target `Entity` causes unnecessary heap allocations per frame.
**Action:** Replace intermediate `HashMap` collections by iterating directly over the source components and performing a direct O(1) `Query::get_mut(target_entity)` lookup to update the target in place. This avoids overlapping mutability issues with the borrow checker while eliminating heap allocations on hot paths.
