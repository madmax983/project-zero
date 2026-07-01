## 2026-06-18 - Deferring allocations
**Learning:** `Vec::with_capacity` and iterators with `.collect()` are sometimes not as useful or idiomatic to "optimize" as one might think because Rust's `Iterator` traits already implement memory capacity hinting (`TrustedLen` / `SpecExtend`). Attempting to manually `Vec::with_capacity` with `size_hint` on `.filter` operations can cause severe memory regressions (allocating array spaces for elements that get filtered out).
**Action:** The safest and most effective way to eliminate allocations is to defer them (e.g. deferring `.clone()` and struct creation to only occur *after* early-exit checks, such as when an iteration list is empty), or remove dummy `.collect()` implementations in dead code.

**Eliminating intermediate HashSets in Bevy queries**
**Learning:** `std::collections::HashSet` allocations inside tight loops or systems called frequently (like scanning logic) cause unnecessary heap pressure. Instead of collecting query target values into an intermediate HashSet, we can query resources dynamically from inside the iter.
**Action:** Extract `world.get_resource::<T>()` prior to `world.query_filtered()`, and reference the exact values inside the `.filter()` closures directly.
**AHash over SipHash for Entity maps**
**Learning:** Bevy provides `bevy::utils::HashMap` which uses `ahash`, a much faster hasher for integer keys like `Entity` than Rust's default `std::collections::HashMap` (which uses SipHash). Using SipHash for mapping entities or coordinates incurs unnecessary overhead.
**Action:** Replace `std::collections::HashMap` with `bevy::utils::HashMap` when the keys are `Entity` or integer coordinate tuples to eliminate SipHash overhead.
**Deferring allocations by avoiding intermediate vectors using `retain_mut`**
**Learning:** `drain` followed by an assignment to `self.field = survivors` forces heap allocation of an intermediate `survivors` vector during operations like `take_damage`.
**Action:** Replace this pattern with `retain_mut(|elem| { /* mutate inline and return true to keep, false to drop */ })`, filtering in-place and eliminating the intermediate heap allocation.

**Eliminating intermediate HashSets in Bevy queries**
**Learning:** `std::collections::HashSet` and `HashMap` allocations inside tight loops or systems called frequently (like scanning logic) cause unnecessary heap pressure and use the slow `SipHash` which is inefficient for integer keys.
**Action:** Replace `std::collections::HashSet` and `HashMap` with `bevy::utils::HashSet` and `HashMap` when the keys are `Entity` or integer coordinate tuples to eliminate `SipHash` overhead. `bevy::utils` provides an optimized `ahash` algorithm.

**Using default initialization for fields to avoid field-reassign-with-default**
**Learning:** Initializing variables with `Default::default()` and then manually overriding some fields triggers the `clippy::field-reassign-with-default` warning and adds a minor unnecessary performance cost.
**Action:** Always prefer the `..Default::default()` unpacking syntax during structure initialization (e.g., `let action = PopAction { current: ActionType::Research, ..Default::default() };`).

**AHash over SipHash for GridPosition sets**
**Learning:** `std::collections::HashSet` uses `SipHash` which is slow for integer keys. In `src/layer1/economy/hauling.rs`, replacing it with `bevy::utils::HashSet` (which uses `AHash`) avoids unnecessary overhead when checking `GridPosition` structures.
**Action:** Always prefer `bevy::utils::HashSet` for small integer keys like coordinates or entities.

**[Acoustic Noise Propagation Allocations]**
**Learning:** The noise propagation system was allocating a VecDeque and a std::collections::HashSet per noise source per frame, causing unnecessary heap allocations. Using std::collections::HashSet on integer tuples also incurs SipHash overhead.
**Action:** Hoist the allocations out of the loop and reuse them with `.clear()`. Switch to `bevy::utils::HashSet` for AHash to avoid SipHash overhead on coordinate keys.
**Avoid cloning simple Entity wrappers**
**Learning:** `MoveToOrder` only contains a single `Entity`. Deriving `Copy` removes the need for explicit `.clone()` calls during event passing and queue processing, saving minimal allocations and making the code more idiomatic.
**Action:** Always derive `Copy` for structs that wrap primitives or `Entity` to avoid `.clone()` calls.

**[Acoustic Physics Allocation Optimization]**
**Learning:** Instantiating `Vec`, `VecDeque`, and `HashSet` inside systems running every frame causes excessive heap allocations. Bevy's `Local<T>` allows systems to retain persistent data structures between frames without needing global Resources.
**Action:** Use `Local<T>` alongside `.clear()` and `.extend()` to reuse capacities for per-system collections, avoiding per-frame allocations.
