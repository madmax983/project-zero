**Removed O(n) Vec::contains overhead from hot fire loops**
**Learning:** Found an accidental O(n^2) scaling loop via `Vec::contains()` inside `fire_spread_system` and `fire_damage_system`. When a single tick spawns many fires, iterating through a Vec to check uniqueness scales poorly.
**Action:** Replaced `Vec` with `bevy_utils::HashSet` to eliminate O(n) lookups. By defaulting to `.insert()`, uniqueness is guaranteed in O(1) time without manual `.contains()` checks.
**Removed intermediate `.collect::<Vec<_>>()` allocation in hot paths**
**Learning:** Found `.collect::<Vec<_>>()` operations like in `filtered_palette_commands` taking extra heap allocations and compute cycles unnecessarily. Another case is when initializing `Vec::new()` without specifying capacity and mapping iterators.
**Action:** Replaced `.collect::<Vec<_>>()` with direct mapping or collecting where needed, and used `Vec::with_capacity` when iterating collections using Bevy ECS queries to optimize capacity upfront.
**Pre-allocating Vecs with Bevy Query Iterators**
**Learning:** Bevy's `QueryIter` does not implement `ExactSizeIterator` (i.e. no `.len()`), making it impossible to directly pre-allocate using `.len()`. However, `size_hint().1` provides a safe upper bound.
**Action:** When pre-allocating a `Vec` driven by a query iteration, use `Vec::with_capacity(query.iter(world).size_hint().1.unwrap_or_default())` to avoid intermediate heap reallocations while collecting query results.
