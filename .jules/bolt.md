## [Selection Prioritization Optimization]
**Learning:** Consolidating three iteration passes prioritizing UI selection logic into a single pass without using an intermediate vector caused a logic regression where lower priority entities masked higher priority ones if evaluated later in the loop.
**Action:** When removing intermediate allocations and combining passes for logic that has explicit priority levels (like `Pop` > `Building` > `Any`), track the priority categories independently using separate variables inside the loop and resolve the priority hierarchy *after* the loop terminates.
