## [Reduction]
**Bloat:** `UtilityAIBuffer` used 12 different `*Proxy` structs that were structurally identical (either `{entity, pos}` or `{entity, pos, capacity, usage}`).
**Cut:** Consolidated into 2 generic structs: `PositionProxy` and `CapacityProxy`. Also removed unused `Plan` struct (YAGNI).
**Saved:** Removed 12 struct definitions (~100 lines of boilerplate) and simplified `UtilityAIBuffer`. Reduced cognitive load by unifying concepts.
