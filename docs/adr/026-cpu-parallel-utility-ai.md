# 26. CPU Parallel Utility AI

Date: 2025-05-20

## Status

Accepted

## Context

The Utility AI system (`evaluate_actions_system`) is the most computationally intensive part of the simulation, iterating over every Pop and evaluating every possible action (Farm, Work, Socialize, etc.) against every relevant entity. This creates an O(N * M) complexity that significantly impacts performance as the colony scales.

Originally, this system ran sequentially on the main thread because it required mutable access to the `World` to query components and update `PopAction` states. The Rust borrow checker prevents trivial parallelization (e.g., `par_iter_mut`) when multiple systems or threads need disjoint access to the same components, or when the system needs to read from the world while writing to it.

While [ADR 013](./013-gpu-accelerated-utility-ai.md) proposes a GPU-based solution for massive scale, a robust and performant CPU implementation is critical for:
1.  **Portability**: Running on systems without dedicated GPUs or with limited WebGPU support.
2.  **Fallback**: Ensuring the simulation remains playable if the GPU pipeline fails.
3.  **Determinism**: CPU floating-point operations are often more strictly deterministic than cross-vendor GPU shaders.

We needed a way to parallelize the CPU evaluation without violating ECS borrow rules or incurring excessive allocation overhead.

## Decision

We have implemented a **Split-Phase Utility AI Architecture** (also known as "Buffer-Oriented Utility AI") that divides the decision process into three distinct phases:

1.  **Phase 1: Collection (Main Thread)**
    *   **Pop Data**: The system iterates over all relevant Pops (filtered by `ticks_committed`) and extracts their read-only state (Needs, Skills, Traits, Position) into a linear `Vec<PopEvalData>`.
    *   **World Context**: The system queries all candidate entities (Farms, Stockpiles, Items) and populates a reusable `UtilityAIBuffer` with lightweight proxy structs (`ScorableCandidate`). This converts an N-query problem into a 1-query bulk operation.

2.  **Phase 2: Parallel Evaluation (Worker Threads)**
    *   Using Bevy's `ComputeTaskPool`, the `Vec<PopEvalData>` is chunked and processed in parallel.
    *   Each thread runs `evaluate_single_pop` against the read-only `UtilityAIBuffer` and `WorldContext`.
    *   Since all input data is read-only and pre-fetched, there are no ECS access conflicts.
    *   Results (Best Action, Score, Target) are written to a pre-allocated `Vec<Option<Decision>>`.

3.  **Phase 3: Application (Main Thread)**
    *   The system iterates over the results vector.
    *   For Pops with a valid new decision (score > current + threshold), the system applies the change to the mutable `PopAction` component in the ECS.

## Consequences

### Positive
*   **Thread Safety**: The split-phase approach eliminates the need for complex internal locking or unsafe code. The borrow checker is satisfied because the mutable write phase is strictly separated from the parallel read phase.
*   **Performance**: Evaluation scales linearly with CPU core count. The "proxy buffering" optimization also improves cache locality by packing candidate data into contiguous memory arrays.
*   **Reduced Allocations**: The `UtilityAIBuffer` resource is reused across frames, preventing the allocation of thousands of temporary vectors (one per pop) that would otherwise occur.
*   **Testability**: The `evaluate_single_pop` function is pure (inputs -> output) and does not require a `World` instance, making it easy to unit test.

### Negative
*   **Memory Overhead**: Storing `PopEvalData` and `ScorableCandidate` proxies duplicates data that already exists in the ECS. For extremely large worlds, this buffer could become significant (though still much smaller than the entity overhead).
*   **Stale Data Risk**: The evaluation phase runs on a snapshot of the world taken at the start of the tick. If another system modifies an entity (e.g., a building is destroyed) *during* the evaluation phase (which is unlikely in the current schedule but possible in future async designs), the AI might make a decision based on outdated info. This is mitigated by validity checks in the Execution phase.
