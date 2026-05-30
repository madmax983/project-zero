## [UI Status and Shell Buffer Optimizations]
**Learning:** We replaced multiple intermediate `.collect::<String>()` chains and `.collect::<Vec<String>>()` vector allocations in `src/ui/status.rs`, `src/ui/inspector.rs`, `src/ui/input.rs`, and `src/ui/shell/plugins` with loop iterations that pre-allocate a `String` with capacity. This reduces heap allocations on the hot path (which renders UI every frame). We also replaced `.collect::<Vec<_>>().join("")` logic that iterates and allocates on `ratatui`'s buffer cells with direct allocations.

**Action:** Whenever possible, avoid `collect::<String>()` and `Vec` allocations during string building inside UI loops. Instead, initialize a `String::with_capacity(N)` and append to it directly inside a loop.

## [String Concatenation Optimization in UI Components]
**Learning:** An optimization that pre-allocated capacity for a `String` inside UI rendering loops (`src/ui/status.rs`, `src/ui/inspector.rs`, etc.) to avoid the intermediate `.collect::<Vec<_>>().join("")` was highly effective and passed safety checks without causing lifetime issues. Using `buffer.area.area() as usize` for capacity effectively prevents reallocation overhead during the hot render path.
**Action:** When working on UI buffers rendering text arrays, use `String::with_capacity(N)` and append inside a loop, rather than chaining iterators that collect into intermediate vectors first.

**Pre-allocate Vector Capacities in Hot Loops**
**Learning:** By tracing the maximum number of elements appended to vectors during UI rendering, I found that `Vec::new()` caused multiple intermediate heap allocations per frame. Counting the maximum items added and using `Vec::with_capacity(n)` instead removes this overhead.
**Action:** Always estimate the maximum length of vectors instantiated in tight loops and use `Vec::with_capacity(n)` to avoid reallocation overhead.

## [Iterator Chaining for Performance]
**Learning:** Replaced manual `Vec::new()` and iterative `.push()` inside `collect_scanners` in `src/layer1/anomalies/mod.rs` with a single `.collect()` pipeline using `.filter()` and `.map()`. While `filter()` obscures the exact capacity bound from `collect()`, the chained iterator is a zero-cost abstraction that completely eliminates the intermediate variable initialization and manual state management.
**Action:** When finding imperative patterns pushing to manually constructed vectors in a loop, refactor them into functional iterator chains (`.iter().filter().map().collect()`) to improve conciseness and potentially leverage compiler optimizations.

**Remove Upfront Allocations of Large Resources**
**Learning:** `OccupiedTiles` is a `HashSet` that can grow to thousands of items. Calling `world.resource::<OccupiedTiles>().0.clone()` just to check if a single tile is occupied causes a massive O(N) heap allocation inside a loop.
**Action:** Use a local scope block `let is_occupied = { let r = world.resource::<...>(); r.contains(...) };` to immutably borrow the resource, perform the check, and drop the borrow before the loop mutates the `World`. This entirely eliminates the allocation and memory copy.
