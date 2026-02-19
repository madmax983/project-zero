## [Reduction]
**Bloat:** `UtilityAIBuffer` used 12 different `*Proxy` structs that were structurally identical (either `{entity, pos}` or `{entity, pos, capacity, usage}`).
**Cut:** Consolidated into 2 generic structs: `PositionProxy` and `CapacityProxy`. Also removed unused `Plan` struct (YAGNI).
**Saved:** Removed 12 struct definitions (~100 lines of boilerplate) and simplified `UtilityAIBuffer`. Reduced cognitive load by unifying concepts.

## [Reduction]
**Bloat:** "Reinforcement Learning" in Utility AI. The system defined `PlanOutcome` and tracked `success_count` and `attempt_count` for every action, sending massive arrays (28x2 u32s) to the GPU per pop every frame. However, `PlanOutcome` was never inserted by any system, making the entire feedback loop dead code.
**Cut:** Removed `PlanOutcome`, `track_plan_outcomes_system`, `calculate_success_modifier`, and all associated fields in `UtilityWeights`, `GpuPopInput`, and WGSL shaders.
**Saved:** ~200 lines of dead logic, 224 bytes of GPU bandwidth per pop per frame, and significant cognitive load in understanding AI evaluation.
