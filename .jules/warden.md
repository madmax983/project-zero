2025-03-07 - [lru IterMut violating Stacked Borrows]
**Threat:** [The `lru` crate version 0.12.5 has a vulnerability where `IterMut` violates Stacked Borrows by invalidating an internal pointer, which can lead to Undefined Behavior (RUSTSEC-2026-0002).]
**Defense:** [Updated `ratatui` to 0.30.0 and `ratzilla` to 0.3.0 which brought in `lru` 0.16.3, resolving the unsoundness.]

2025-03-07 - [Unhandled Unwraps on ECS and GPU boundaries]
**Threat:** [The codebase contained unguarded `.unwrap()` calls. In `src/main.rs`, `world.run_system_once(update_screen_shake_system).unwrap()` could panic the entire simulation loop. In `src/gpu/evaluate.rs`, multiple `cache.buffer.as_ref().unwrap()` calls assumed GPU resource allocation always succeeds, which can panic during VRAM exhaustion or device loss contexts.]
**Defense:** [Replaced the `.unwrap()` in `src/main.rs` with safe error logging (`if let Err(e) = ...`). Replaced the GPU buffer unwraps in `src/gpu/evaluate.rs` with `match` blocks that return early and log errors if the buffers are missing.]
