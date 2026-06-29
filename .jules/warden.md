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
**2023-10-27 - [Memory Exhaustion in Lore Loading]**
**Threat:** Unbounded file read in `NarrativeGenerator::load_from_files` allowing memory exhaustion DoS via huge lore files.
**Defense:** Replaced `fs::read_to_string` with `File::open` and `take(LIMIT)` to enforce a strict memory ceiling.

**2026-06-07 - proc-macro-error2 is unmaintained**
**Threat:** The `proc-macro-error2` crate is unmaintained (RUSTSEC-2026-0173), which could lead to unpatched vulnerabilities in the future.
**Defense:** Downgraded `env_logger` to `0.11.0` via `cargo update` and pinned it in `Cargo.toml` to remove the transitive dependency chain (`env_logger` -> `jiff` -> `defmt` -> `defmt-macros` -> `proc-macro-error2`).
**2026-06-28 - Uninitialized Events causing test panics**
**Threat:** Tests that invoke full system execution (e.g., `schedule.run(\&mut world)` or tests running the full simulation loop) panicked with Bevy errors: `could not access system parameter ResMut<'_, Events<T>>`. While these were mostly constrained to tests missing setup resources, uninitialized `Events<T>` configurations in core routines present a severe fragility risk that could lead to crashes in the running simulation if dependencies or startup routines change.
**Defense:** Explicitly initialized `Events<DebtInheritedEvent>`, `Events<DebtSocializedEvent>`, `Events<PublishDiscoveryEvent>`, and `Events<AttackColonyEvent>` in test configurations (`setup_world` blocks) as well as ensuring coverage in the core `init_simulation_resources` block to close the crash loop vectors.

**2026-06-30 - Out-of-bounds Read/Write via Huge Negative `as usize` Grid Access**
**Threat:** Several simulation and experimental features accessed grid arrays by casting potentially negative `pos.x` and `pos.y` (`i32`) to `usize`. Because these casts were performed before safe coordinate clamping functions were used or outside of them entirely, huge negative numbers wrapped to `usize::MAX`, defeating downstream bounds checking mechanisms like `.get()` which simply checked `if x < width` (where a massive value would evaluate to `false` and return `None` silently breaking mechanics) or caused panics when creating arrays.
**Defense:** Added explicit `pos.x >= 0 && pos.y >= 0` boundary checks *before* `as usize` casting or indexing operations in vulnerable locations including `structural_integrity`, `load_limits`, `orbital_drop`, `conveyor`, `resources`, `sleepwalking_hazards`, and others. This ensures all accesses drop invalid coordinates safely.
