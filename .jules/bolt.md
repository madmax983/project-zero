**[Optimized PopEvalData]**
**Learning:** `Traits`, `ChemicalState`, and `FactionMember` components were frequently cloned inside `PopEvalData` causing a performance bottleneck on the hot path for `evaluate_actions_system`. `HashSet<Trait>` inside `Traits` specifically added high allocation overhead.
**Action:** Changed `Traits` to utilize a `u64` bitmask, removing `HashSet`, making it 8 bytes and `Copy`. Added `Copy` to `FactionMember`. This eliminated heap allocations on the hottest path in the codebase.

## Avoid HashMap initialization inside tight loops
**Learning:** `HashSet::new()` and iter -> collect causes memory allocation.
**Action:** Lift `HashMap` / `HashSet` creation out of loops by storing them in a persistent state/buffer that can be cleared each iteration.
