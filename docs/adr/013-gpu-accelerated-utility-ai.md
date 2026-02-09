# 13. GPU Accelerated Utility AI

Date: 2024-10-25
Status: Accepted

## Context

As the simulation scales, the number of Pops (N) and Buildings (M) increases. The Utility AI system evaluates every possible action for every pop against every relevant building. This results in a computational complexity of O(N * M) per decision cycle.

With hundreds of pops and thousands of buildings, this evaluation becomes a significant CPU bottleneck, causing frame rate drops and simulation lag. The CPU-based `evaluate_actions_system` iterates sequentially (or with limited parallelism via `par_iter`) over these combinations, consuming valuable main thread time.

## Decision

We have implemented a GPU compute pipeline using `wgpu` to parallelize action evaluation.

The system `gpu_evaluate_actions` replaces the CPU-based `evaluate_actions_system` when a GPU is available. The process is as follows:

1.  **Extraction:** The system queries the ECS for all Pops that are ready to update (based on their `ticks_committed` timer). It extracts their state (Needs, Memories) into a linear `GpuPopInput` buffer.
2.  **Context Assembly:** The system extracts all relevant Buildings (Farms, Stockpiles, etc.) into a `GpuBuildingInput` buffer.
3.  **Upload:** These buffers are uploaded to the GPU via `wgpu` Storage Buffers.
4.  **Dispatch:** A compute shader (`evaluate.wgsl`) is dispatched with one thread per Pop. Each thread iterates over the building buffer to score all possible actions in parallel.
5.  **Readback:** The system reads back the `GpuPopDecision` buffer, containing the best action and target index for each Pop.
6.  **Application:** The results are applied back to the ECS `PopAction` components in a batch.

A CPU fallback (`evaluate_actions_system`) is retained for platforms without GPU support (e.g., some WASM environments or headless servers) or if GPU initialization fails.

## Consequences

### Positive
*   **Performance:** Massive performance improvement for large colonies. The GPU can process thousands of scoring operations in parallel, freeing up the CPU for other systems (pathfinding, logistics).
*   **Scalability:** The simulation can support significantly more entities before hitting performance limits.

### Negative
*   **Complexity:** Requires maintaining a duplicate implementation of the scoring logic in WGSL (WebGPU Shading Language). Any change to the utility formulas must be applied to both Rust and WGSL code.
*   **Data Marshalling:** Overhead of copying data between CPU and GPU memory. This is mitigated by using persistent `GpuBuffers` to avoid reallocation.
*   **Debugging:** Debugging GPU shaders is significantly harder than debugging Rust code.
