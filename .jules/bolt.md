**[Vec Capacity Allocation Optimization]**
**Learning:** Found an empty initialization of `Vec::new()` inside `update_drone_clusters` which then proceeded to push dynamically populated elements from `query.iter()`, leading to unneeded allocations. Replacing with `Vec::with_capacity(query.iter().len())` provides immediate allocation sizes, preventing the resize and reallocation cost, significantly improving memory and processing.
**Action:** Always pre-allocate vectors when the bounds or capacities are known (such as using `.len()` on a `query.iter()`) instead of initializing with `Vec::new()`.
**[Vec Capacity Pre-allocation in Iteration]**\n**Learning:** When pulling entries from a query iterator that provides an exact length (), allocating the vector using  exactly matches the bounds and prevents any intermediate reallocation resizing at no cost.\n**Action:** Use  on the iterator and  whenever iterating query results into a collection, rather than .

**[Vec Capacity Pre-allocation in Iteration]**
**Learning:** When pulling entries from a query iterator that provides an exact length (`pops.iter().len()`), allocating the vector using `Vec::with_capacity` exactly matches the bounds and prevents any intermediate reallocation resizing at no cost.
**Action:** Use `.len()` on the iterator and `Vec::with_capacity()` whenever iterating query results into a collection, rather than `Vec::new()`.
