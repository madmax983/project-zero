# 7. Parallelism Roadmap: CPU par_iter, Bevy Schedule, GPU Compute

Date: 2025-02-06

## Status

Accepted (Phase 1+2 implemented; Phase 3 planned)

## Context

SCALE's simulation runs 22 systems each tick, all originally implemented as exclusive systems taking `&mut World`. This locks the entire ECS World per system call, preventing Bevy's multi-threaded executor from running any systems in parallel. At small scale (~5 pops, ~20 buildings) this is fine, but the architecture has two scaling problems:

1. **Inter-system**: Exclusive systems run strictly sequentially even when they touch disjoint data (e.g., `advance_season_system` and `restore_leisure_system` have zero overlap).
2. **Intra-system**: The hot path `evaluate_actions_system` is O(N pops x M buildings) — each pop queries every farm, house, tavern, library, designation, item, and stockpile. This is embarrassingly parallel but was running single-threaded.

Since SCALE is a TUI application, the GPU is entirely free for compute work, making it a natural target for the heaviest simulation workloads.

## Decision

We adopted a **three-phase approach** to parallelism:

### Phase 1+2: Bevy Schedule + Typed System Params (Implemented)

**Replace manual `run_simulation_tick()` calls with a Bevy `Schedule`** and convert exclusive systems to typed params.

- **Why Bevy Schedule, not raw rayon?** Bevy's `Schedule` with `multi_threaded` feature provides automatic parallel execution based on declared data access. Systems with non-overlapping `Query`/`Res`/`ResMut` params run concurrently without manual thread management. This is safer (compile-time borrow checking via ECS), requires less code, and integrates naturally with Bevy's `par_iter_mut()` for intra-system parallelism.

- **Alternatives rejected:**
  - **Raw rayon `par_iter`**: Would require manual synchronization, no integration with ECS access tracking, and risks data races.
  - **Tokio async tasks**: Wrong paradigm for frame-synchronous game simulation — async is for I/O concurrency, not compute parallelism.

**Conversion results**: 19 of 22 systems converted to typed params. Three remain as exclusive `fn(&mut World)`:
- `evaluate_actions_system` — reads many queries across the entire World (Phase 3 GPU target)
- `work_execution_system` — calls `mine_rock`/`chop_tree` which take `&mut World` for entity spawning
- `haul_system` — complex multi-query state machine

**Schedule ordering** organizes systems into five groups:

```
1. AI Decision:   evaluate_actions → update_action_timer
2. Execution:     cleanup_previous → start_plan → movement → arrival → work/haul
3. Economy:       resource_caps | season | produce_food | refining | research | rest | leisure
4. Consumption:   consume_food → decay_needs → kill_starving → clean_dead_*
5. Observation:   track_outcomes | biography | dreams | milestones
```

Within each group, systems without data dependencies (joined by `|`) run in parallel automatically. The Economy and Observation groups each have 4-7 systems that can execute concurrently.

### Phase 3: GPU Compute with wgpu (Planned)

**Offload `evaluate_actions_system` to the GPU** using wgpu compute shaders written in WGSL.

- **Why wgpu, not CUDA?** Portability. wgpu compiles to Vulkan (Windows/Linux), Metal (macOS), DX12 (Windows), and WebGPU (browsers). CUDA is NVIDIA-only and incompatible with WASM. Since SCALE already has a WASM build (ADR 006), a portable GPU API preserves that capability.

- **Why WGSL over CubeCL?** WGSL is the native shader language for wgpu and WebGPU. Writing it directly provides:
  - Transparent, debuggable GPU code (no macro magic)
  - Educational value (learning GPU programming is an explicit goal)
  - No additional dependencies or build complexity
  - CubeCL can be adopted later if WGSL becomes unwieldy for more complex kernels.

- **Why GPU for utility AI?** The evaluation is embarrassingly parallel: each pop's score against each building is independent. At 100 pops x 100 buildings = 10,000 evaluations per tick; at 1,000 x 1,000 = 1,000,000. This is classic GPU workload territory — heavy reads, light writes, simple arithmetic.

**Data flow:**
```
CPU (bevy_ecs)                              GPU (wgpu/WGSL)
──────────────                              ────────────────
Pop positions, needs, weights  ──────►  ┌──────────────────┐
Building positions, types      ──────►  │ For each pop:     │
                                        │   For each bldg:  │
                                        │     score(pop,bldg)│
                                        │   best = argmax   │
Best action + target per pop  ◄──────   └──────────────────┘
```

Feature-gated under `gpu = ["dep:wgpu"]`, with the CPU path remaining as the default fallback.

## Consequences

**Positive:**
- Economy and Observation groups (~11 systems) can now run in parallel automatically
- `decay_needs_system` and `update_action_timer_system` use `par_iter_mut` for intra-system parallelism
- Typed params provide compile-time verification of data access patterns
- Schedule ordering is declarative and self-documenting
- GPU compute (Phase 3) will unlock scaling to thousands of entities
- WASM build is unaffected — `multi_threaded` uses rayon internally, which degrades gracefully to single-threaded in WASM

**Negative:**
- `ComputeTaskPool` must be initialized before any `par_iter_mut` call (test helper `init_task_pools()` required)
- Three systems remain exclusive, limiting full parallelism until they're refactored
- Phase 3 adds wgpu as an optional dependency (~significant compile time increase)
- GPU path requires CPU-GPU data marshaling overhead, only worthwhile above ~100 entities

**Trade-offs:**
- Converting to typed params changes test patterns from `system_fn(&mut world)` to `world.run_system_once(system_fn).unwrap()`, requiring `RunSystemOnce` imports
- Bevy's `Commands` use deferred execution — component insertions/removals apply after system completes, not immediately, which changes the mental model for systems that previously used `world.entity_mut()` directly
- Schedule ordering constraints must be maintained manually when adding new systems
