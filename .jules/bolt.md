**[Vec Capacity Allocation Optimization]**
**Learning:** Found an empty initialization of `Vec::new()` inside `update_drone_clusters` which then proceeded to push dynamically populated elements from `query.iter()`, leading to unneeded allocations. Replacing with `Vec::with_capacity(query.iter().len())` provides immediate allocation sizes, preventing the resize and reallocation cost, significantly improving memory and processing.
**Action:** Always pre-allocate vectors when the bounds or capacities are known (such as using `.len()` on a `query.iter()`) instead of initializing with `Vec::new()`.
## [Performance]
**Learning:** In Bevy, `query.iter().len()` is an O(1) operation (via ExactSizeIterator) because Archetypes maintain their counts.
**Action:** When collecting items from a query into a new vector, use `Vec::with_capacity(query.iter().len())` rather than an un-sized `Vec::new()` to safely avoid dynamic reallocation overhead. This is safe and circumvents borrow-checker issues if `world` ownership flows cleanly.
**[Loop Allocation Optimization]
**Learning:** Found an imperative nested loop allocating vectors inside `exchange_rumors_system`. Replacing with iterators and `flat_map` along with `collect` automatically computes capacity and removes intermediate manual `Vec` allocation management overhead.
**Action:** Use `flat_map` on nested iterations combined with `collect` to avoid manual Vec capacity tracking and let Rust optimize memory allocation natively.
**[Avoiding Temporary Allocation in Inner Loop]
**Learning:** Found an unneeded `Vec::new()` temporary allocation being used inside `resonance_social_spread_system` to store entities for secondary update. Iterating mutably (`iter_mut`) using the same component removes the need to collect elements into an intermediate `Vec`, reducing heap allocations per frame by 1.
**Action:** Iterate mutably when collecting updates for elements in a query to avoid intermediate `Vec` allocations.
