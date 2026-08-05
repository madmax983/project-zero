**[Vec Capacity Allocation Optimization]**
**Learning:** Found an empty initialization of `Vec::new()` inside `update_drone_clusters` which then proceeded to push dynamically populated elements from `query.iter()`, leading to unneeded allocations. Replacing with `Vec::with_capacity(query.iter().len())` provides immediate allocation sizes, preventing the resize and reallocation cost, significantly improving memory and processing.
**Action:** Always pre-allocate vectors when the bounds or capacities are known (such as using `.len()` on a `query.iter()`) instead of initializing with `Vec::new()`.
## [Performance]
**Learning:** In Bevy, `query.iter().len()` is an O(1) operation (via ExactSizeIterator) because Archetypes maintain their counts.
**Action:** When collecting items from a query into a new vector, use `Vec::with_capacity(query.iter().len())` rather than an un-sized `Vec::new()` to safely avoid dynamic reallocation overhead. This is safe and circumvents borrow-checker issues if `world` ownership flows cleanly.
**[Fix fire logic overlapping accumulation]**
**Learning:** When moving from nested iteration to an intermediate HashMap to avoid O(N*M) lookups, using `HashMap::from_iter` or standard `.collect()` overwrites elements if multiple entities map to the same key, changing functional behavior.
**Action:** When grouping components by position or other keys where multiples might coexist, manually accumulate values (e.g. `*map.entry(pos).or_default() += val;`) to preserve stack behaviors. Always remove temporary scratchpad scripts (like `.patch` or `.py` files) before concluding a PR.
**Pre-allocated Rumor Exchange Pairs Vector**
**Learning:** In highly trafficked ECS system loops (e.g. `exchange_rumors_system`), initializing an un-sized `Vec` using `Vec::new()` inside the system call causes repeated heap allocations per frame, particularly when collecting pairwise combinations of entities. Pre-calculating the required capacity by summing over the combinatorial size hints (e.g., `n * (n - 1)`) and initializing with `Vec::with_capacity()` completely eliminates intermediate memory fragmentation without breaking borrow checker rules.
**Action:** Use `Vec::with_capacity` for system-local buffers whenever the expected collection size can be mathematically derived from `query.iter()` lengths, especially for polynomial or combinatorial combinations.

**[BFS Pathfinding O(N^2) Vector Cloning]**
**Learning:** In standard BFS algorithms, storing and cloning the entire `Vec` path in the queue at every step creates massive O(N^2) memory allocations. By using a `came_from` HashMap to store back-pointers, we only allocate a single `Vec` at the end and reconstruct the path backwards.
**Action:** When implementing pathfinding or BFS, never clone paths inside the loop. Use a `came_from` structure to track visited states and reconstruct the path once the destination is reached.
**[Iterator `max_by` Optimization]**
**Learning:** In hot loops tracking priority (like resolving the highest favored faction by social debt in `evaluate_faction_support_system`), fetching elements into a dynamically allocated vector (`.collect::<Vec<_>>()`) and sorting them entirely (`.sort_by()`) creates an `O(N log N)` bottleneck with constant memory reallocation. This can be perfectly bypassed by composing iterator adaptors like `.filter_map()` chained directly into `.max_by()`, which resolves the same deterministic answer in `O(N)` time with zero heap allocations.
**Action:** When you only need the "best" or "worst" element of a collection, always prefer `.max_by()` or `.min_by()` on iterators over `.collect::<Vec<_>>().sort_by()`. Remember to carefully trace the comparison logic: `max_by` seeks the highest element, so inverted tie-breaking (`e2.cmp(e1)` to get the lowest ID) is crucial to maintain original deterministic behavior.

**[A* Memory Optimization: Replacing large vectors with HashMaps]**
**Learning:** In A* pathfinding on large grids, allocating two vectors of size `width * height` (`came_from` and `cost_so_far`) on *every* pathfinding request is incredibly slow and fragmenting, especially when the path is much smaller than the grid.
**Action:** Replace full-grid pre-allocated tracking vectors in A* with `bevy::utils::HashMap` keyed by grid index. This strictly limits memory allocation to the explored path area instead of the entire grid, scaling pathfinding cost to the path length rather than the map size.
