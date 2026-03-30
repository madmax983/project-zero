**[Optimized PopEvalData]**
**Learning:** `Traits`, `ChemicalState`, and `FactionMember` components were frequently cloned inside `PopEvalData` causing a performance bottleneck on the hot path for `evaluate_actions_system`. `HashSet<Trait>` inside `Traits` specifically added high allocation overhead.
**Action:** Changed `Traits` to utilize a `u64` bitmask, removing `HashSet`, making it 8 bytes and `Copy`. Added `Copy` to `FactionMember`. This eliminated heap allocations on the hottest path in the codebase.

**[Optimized Hauling Stockpile Lookup]**
**Learning:** `src/layer1/hauling.rs`'s `find_and_target_generic_item` was collecting `Stockpile` `GridPosition`s into a `Vec` inside an O(N) loop run by idle haulers, turning a fast `query.iter()` check into O(N) vec allocation per idle hauler + `vec.contains(p)` O(M) inside the loop.
**Action:** Changed the `Vec` collection to a `HashSet` to turn the inner loop `contains` check from O(M) to O(1), mathematically reducing the time complexity of the stockpile inclusion check from `O(N * M)` to `O(N + M)`.
