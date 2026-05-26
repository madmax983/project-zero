## [Integration Bug]
**Bug:** Exclusive systems blocking parallel execution and improperly draining events.
**Fix:** Instead of consuming all events via an exclusive `&mut World` system or `SystemState`, define `PlayerDemandResponse` with `#[derive(Event, Clone)]` and use `EventReader<PlayerDemandResponse>` alongside standard resource queries in a normal system.
**Saved:** Fixes compile errors when using `.chain()` and prevents logic black holes by keeping events in Bevy's normal buffering lifecycle.
## [Memory Safety / Overflow]
**Threat:** Integer overflow via unbounded arithmetic inside resource mechanics (`mine_rock` and `chop_tree`) calculating index out-of-bounds due to use of `unwrap_or(usize::MAX)` which resulted in unreachable tiles defaulting to panic values under certain conditions.
**Defense:** Explicitly used `if let Some(idx) = ...` alongside bounds checks instead of relying on `unwrap_or(usize::MAX)` fallback, effectively hardening index boundary parsing. Also introduced `#![deny(unsafe_code)]` inside `src/lib.rs` and `src/main.rs`.
