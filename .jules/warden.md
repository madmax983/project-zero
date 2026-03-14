2025-03-07 - [lru IterMut violating Stacked Borrows]
**Threat:** [The `lru` crate version 0.12.5 has a vulnerability where `IterMut` violates Stacked Borrows by invalidating an internal pointer, which can lead to Undefined Behavior (RUSTSEC-2026-0002).]
**Defense:** [Updated `ratatui` to 0.30.0 and `ratzilla` to 0.3.0 which brought in `lru` 0.16.3, resolving the unsoundness.]

2025-03-07 - [Unhandled Unwraps on ECS and GPU boundaries]
**Threat:** [The codebase contained unguarded `.unwrap()` calls. In `src/main.rs`, `world.run_system_once(update_screen_shake_system).unwrap()` could panic the entire simulation loop. In `src/gpu/evaluate.rs`, multiple `cache.buffer.as_ref().unwrap()` calls assumed GPU resource allocation always succeeds, which can panic during VRAM exhaustion or device loss contexts.]
**Defense:** [Replaced the `.unwrap()` in `src/main.rs` with safe error logging (`if let Err(e) = ...`). Replaced the GPU buffer unwraps in `src/gpu/evaluate.rs` with `match` blocks that return early and log errors if the buffers are missing.]
**2025-05-20 - Unbounded Grid Allocation DOS**
**Threat:** `TemperatureGrid` and `RadiationGrid` initialization allowed arbitrarily large unbounded memory allocation (via unchecked `width * height`) leading to Potential Denial of Service (OOM panic) if grid dimensions were user-controlled.
**Defense:** Enforced strict capacity bounds (max 1,000,000 tiles) and utilized checked arithmetic (`checked_mul`) for grid initialization to safely abort.
**2024-10-24 - [Grid Index Arithmetic Integer Overflow]**
**Threat:** [Integer overflow in grid index calculations (y * width + x) within `TemperatureGrid` and `RadiationGrid` allowing DoS via application panics.]
**Defense:** [Switched to safe arithmetic (`checked_mul` and `checked_add`) for bounded capacity and coordinate indexing in `get`, `set`, `add`, and diffusion calculations.]

**2024-03-14 - [Panic Avoidance in Core Modules]**
**Threat:** Use of `.unwrap()` on slices, hashmap lookups, and ECS query data (`Option` unwraps) could cause game-crashing panics (Denial of Service) if logic assumptions were broken (e.g. empty arrays or missing components).
**Defense:** Replaced `unwrap()` calls with robust `if let Some(...)` blocks and fallbacks across `src/layer1/oral_tradition.rs`, `src/layer1/dreams.rs`, `src/layer1/specialization.rs`, `src/layer1/science.rs`, `src/layer1/social/grievances.rs`, `src/layer1/void_signals.rs`, and `src/layer1/eureka.rs`.
