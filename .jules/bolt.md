**[Exclusive System Vectors]**
**Learning:** Exclusive systems (`fn(world: &mut World)`) often force you to iterate over queries, collect entity IDs into intermediate vectors (e.g. `Vec::new()`), and then iterate *again* to mutate the world to satisfy borrow checker rules.
**Action:** Always convert exclusive systems to standard Bevy systems (`Query`, `ResMut`) when possible. This allows you to apply mutations directly within the `iter_mut()` loop, completely eliminating the heap allocations of the intermediate vectors and allowing parallel system execution.

**[HashMap Hashing]**
**Learning:** `std::collections::HashMap` uses a cryptographically strong but slow default hasher, which causes significant overhead in hot loops (like cellular automata physics simulations mapping `(i32, i32)` to `f32`).
**Action:** Always use `bevy::utils::HashMap` (which defaults to `AHasher`) inside Bevy queries and physics loops instead of `std::collections::HashMap`.
