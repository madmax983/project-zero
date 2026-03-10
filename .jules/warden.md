2025-03-07 - [lru IterMut violating Stacked Borrows]
**Threat:** [The `lru` crate version 0.12.5 has a vulnerability where `IterMut` violates Stacked Borrows by invalidating an internal pointer, which can lead to Undefined Behavior (RUSTSEC-2026-0002).]
**Defense:** [Updated `ratatui` to 0.30.0 and `ratzilla` to 0.3.0 which brought in `lru` 0.16.3, resolving the unsoundness.]

2026-03-10 - [Integer Overflow DoS in Grids]
**Threat:** `TemperatureGrid` and `RadiationGrid` allocated unbounded memory (`width * height`) and calculated un-checked array indices which could overflow on user-provided size inputs, leading to panics (DoS) or Out of Memory (OOM).
**Defense:** Applied safe array allocation with `checked_mul` and explicit limits (`1_000_000`). Rewrote index calculations in `get`, `set`, and `add` using `checked_mul` and `checked_add` to fail safely and gracefully.
