## 2026-06-18 - Deferring allocations
**Learning:** `Vec::with_capacity` and iterators with `.collect()` are sometimes not as useful or idiomatic to "optimize" as one might think because Rust's `Iterator` traits already implement memory capacity hinting (`TrustedLen` / `SpecExtend`). Attempting to manually `Vec::with_capacity` with `size_hint` on `.filter` operations can cause severe memory regressions (allocating array spaces for elements that get filtered out).
**Action:** The safest and most effective way to eliminate allocations is to defer them (e.g. deferring `.clone()` and struct creation to only occur *after* early-exit checks, such as when an iteration list is empty), or remove dummy `.collect()` implementations in dead code.

**Eliminating intermediate HashSets in Bevy queries**
**Learning:** `std::collections::HashSet` allocations inside tight loops or systems called frequently (like scanning logic) cause unnecessary heap pressure. Instead of collecting query target values into an intermediate HashSet, we can query resources dynamically from inside the iter.
**Action:** Extract `world.get_resource::<T>()` prior to `world.query_filtered()`, and reference the exact values inside the `.filter()` closures directly.
