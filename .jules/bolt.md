## [Selection Prioritization Optimization]
**Learning:** Consolidating three iteration passes prioritizing UI selection logic into a single pass without using an intermediate vector caused a logic regression where lower priority entities masked higher priority ones if evaluated later in the loop.
**Action:** When removing intermediate allocations and combining passes for logic that has explicit priority levels (like `Pop` > `Building` > `Any`), track the priority categories independently using separate variables inside the loop and resolve the priority hierarchy *after* the loop terminates.

**[Biography Rendering Optimization]
**Learning:** Removing an intermediate `.collect::<Vec<_>>()` allocation when iterating and rendering large lists of struct data (like Biography events) directly into a `ratatui` UI component improves performance by avoiding heap allocations during the hot UI rendering path.
**Action:** Always prefer passing an iterator directly to UI components like `List::new(events)` instead of collecting the mapped spans or elements into an intermediate vector first, unless specifically required for reverse iteration logic that isn't native to the structure.
