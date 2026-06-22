## [Integration Bug]
**Bug:** Exclusive systems blocking parallel execution and improperly draining events.
**Fix:** Instead of consuming all events via an exclusive `&mut World` system or `SystemState`, define `PlayerDemandResponse` with `#[derive(Event, Clone)]` and use `EventReader<PlayerDemandResponse>` alongside standard resource queries in a normal system.
**Saved:** Fixes compile errors when using `.chain()` and prevents logic black holes by keeping events in Bevy's normal buffering lifecycle.

**2025-06-02 - Deserialization DoS in Layout Config**
**Threat:** Unbounded JSON deserialization in `decoded_persisted_layout` could allow a malformed, massive string to crash the game (DoS) via memory exhaustion or stack overflow.
**Defense:** Added a 1MB length limit to the layout JSON before passing it to `serde_json::from_str`.

**2025-02-09 - O(N) Inventory Remove Performance Vector**
**Threat:** Heavy usage of `Vec::remove()` on `Inventory::items` under simulation load leads to O(N) shifting of elements. This causes severe simulation lag and frame drops which functions as a DoS vector.
**Defense:** Replaced 10 usages of `Vec::remove()` with `Vec::swap_remove()`, turning an O(N) operation into an O(1) operation.
2026-06-20 - Out-of-bounds Read/Write via Negative Coordinates in Grids
**Threat:** Several simulation and experimental features accessed grid arrays by casting negative `pos.x` and `pos.y` (i32) to `usize` without bounds checking, which could lead to buffer overflows/panics (or logic bugs when the massive wrapped `usize` fails safe-grid bounds checks and silently drops processing).
**Defense:** Explicit `pos.x >= 0 && pos.y >= 0` boundary checks were added before any `as usize` casts in nature systems, logging, and experimental components.
**2025-06-25 - Grid Coordinate Bounds Checks and Integer Overflows**
**Threat:** Missing `x >= 0 && y >= 0` bound checking in `harpoon.rs` prior to `as usize` cast, resulting in `usize::MAX` values wrapping and indexing out of bounds into arrays. In `aeolian_clutter.rs`, `width * y + x` calculations were made without `checked_mul` or `checked_add`, allowing potential integer overflow exploits.
**Defense:** Added robust boundary checks for `x` and `y` before performing coordinate conversions. Rewrote manual array indexing calculations to use safe `checked_mul` and `checked_add` and check `new_idx < new_clutter.len()` to prevent memory violations.
