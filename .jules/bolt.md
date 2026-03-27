**[Optimized PopEvalData]**
**Learning:** `Traits`, `ChemicalState`, and `FactionMember` components were frequently cloned inside `PopEvalData` causing a performance bottleneck on the hot path for `evaluate_actions_system`. `HashSet<Trait>` inside `Traits` specifically added high allocation overhead.
**Action:** Changed `Traits` to utilize a `u64` bitmask, removing `HashSet`, making it 8 bytes and `Copy`. Added `Copy` to `FactionMember`. This eliminated heap allocations on the hottest path in the codebase.

## Avoid HashMap initialization inside tight loops
**Learning:** `HashSet::new()` and iter -> collect causes memory allocation.
**Action:** Lift `HashMap` / `HashSet` creation out of loops by storing them in a persistent state/buffer that can be cleared each iteration.

**[Zero-Allocation Apply Collapse]**
**Learning:** Found an unnecessary `Vec<Entity>` allocation in `src/layer1/structural_integrity.rs` where the system collected entities via a read-only query and then performed a second iteration with `world.get_mut::<Health>()` to apply damage.
**Action:** Replaced the two loops with a single `query_mut` pass on `(&GridPosition, &mut Health)` to eliminate the heap allocation and O(N) entity lookups. Always prefer single-pass mutable queries over intermediate collections when updating components.

**[Unused Doc Comments on Expressions]**
**Learning:** Placing `///` doc comments directly above a `for` loop or other expressions triggers an `unused_doc_comments` warning. Under `-D warnings`, this causes a compilation error.
**Action:** Use standard `//` comments for inline code explanations and reserve `///` strictly for documenting items like structs, enums, and functions.
