**[Vec Capacity Allocation Optimization]**
**Learning:** Found an empty initialization of `Vec::new()` inside `update_drone_clusters` which then proceeded to push dynamically populated elements from `query.iter()`, leading to unneeded allocations. Replacing with `Vec::with_capacity(query.iter().len())` provides immediate allocation sizes, preventing the resize and reallocation cost, significantly improving memory and processing.
**Action:** Always pre-allocate vectors when the bounds or capacities are known (such as using `.len()` on a `query.iter()`) instead of initializing with `Vec::new()`.
**[Bevy SystemState Vec Elimination]**
**Learning:** Found an exclusive system (`fauna_behavior_system` taking `&mut World`) that executed `world.query().iter().collect::<Vec<_>>()` on every single frame just to pass data to helper functions without violating the borrow checker.
**Action:** Use `bevy_ecs::system::SystemState` inside the exclusive system (or refactor to a standard system) to extract disjoint queries safely. This completely eliminates the `Vec` allocation and enables O(1) lookups via `query.get()` instead of O(N) `.find()` linear searches on the vector.
