## [UI Status and Shell Buffer Optimizations]
**Learning:** We replaced multiple intermediate `.collect::<String>()` chains and `.collect::<Vec<String>>()` vector allocations in `src/ui/status.rs`, `src/ui/inspector.rs`, `src/ui/input.rs`, and `src/ui/shell/plugins` with loop iterations that pre-allocate a `String` with capacity. This reduces heap allocations on the hot path (which renders UI every frame). We also replaced `.collect::<Vec<_>>().join("")` logic that iterates and allocates on `ratatui`'s buffer cells with direct allocations.

**Action:** Whenever possible, avoid `collect::<String>()` and `Vec` allocations during string building inside UI loops. Instead, initialize a `String::with_capacity(N)` and append to it directly inside a loop.

## [String Concatenation Optimization in UI Components]
**Learning:** An optimization that pre-allocated capacity for a `String` inside UI rendering loops (`src/ui/status.rs`, `src/ui/inspector.rs`, etc.) to avoid the intermediate `.collect::<Vec<_>>().join("")` was highly effective and passed safety checks without causing lifetime issues. Using `buffer.area.area() as usize` for capacity effectively prevents reallocation overhead during the hot render path.
**Action:** When working on UI buffers rendering text arrays, use `String::with_capacity(N)` and append inside a loop, rather than chaining iterators that collect into intermediate vectors first.
