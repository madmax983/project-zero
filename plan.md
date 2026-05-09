1. **Change imports in `src/layer1/architecture/building.rs`**
   - Update `use std::collections::{HashMap, HashSet};` to `use bevy::utils::{HashMap, HashSet};` to take advantage of `AHash` for better hashing performance, especially since `BuildingMap` uses `(i32, i32)` integer tuples as keys, where SipHash overhead is high.
2. **Update doc comments in `src/layer1/architecture/building.rs`**
   - Add a `/// ⚡ Bolt Optimization: Switched to \`bevy::utils::HashMap\` and \`bevy::utils::HashSet\` (AHash) to eliminate SipHash overhead for integer coordinate keys, avoiding bottlenecks during building lookups and placement validation.` to both `OccupiedTiles` and `BuildingMap`.
3. **Verify other usages in the file**
   - Confirm if `HashMap` and `HashSet` are used elsewhere in `building.rs` and verify they compile with the `bevy::utils` variants (they should, as the APIs are largely identical).
4. **Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.**
5. **Submit the change.**
   - Commit with title `⚡ Bolt: Use AHash for building spatial structures` and describe the optimization.
