**[Exclusive System Vectors]**
**Learning:** Exclusive systems (`fn(world: &mut World)`) often force you to iterate over queries, collect entity IDs into intermediate vectors (e.g. `Vec::new()`), and then iterate *again* to mutate the world to satisfy borrow checker rules.
**Action:** Always convert exclusive systems to standard Bevy systems (`Query`, `ResMut`) when possible. This allows you to apply mutations directly within the `iter_mut()` loop, completely eliminating the heap allocations of the intermediate vectors and allowing parallel system execution.

**[HashMap Hashing]**
**Learning:** `std::collections::HashMap` uses a cryptographically strong but slow default hasher, which causes significant overhead in hot loops (like cellular automata physics simulations mapping `(i32, i32)` to `f32`).
**Action:** Always use `bevy::utils::HashMap` (which defaults to `AHasher`) inside Bevy queries and physics loops instead of `std::collections::HashMap`.

**[Optimize String Collect and Join]**
**Learning:** `buffer.content().iter().map(|cell| cell.symbol()).collect::<Vec<_>>().join("")` creates unnecessary intermediate `Vec` allocations just to pass to `.join()`.
**Action:** Use `.collect::<String>()` directly instead to allocate only once for the target `String` and avoid creating and tearing down an intermediate heap-allocated `Vec`.

**[Bevy Resource Scope]**
**Learning:** `world.resource_scope` can be used to safely borrow a resource from the `World` without holding a conflicting immutable borrow on the whole `World`. This avoids needing to create intermediate `Vec` allocations to store query results before processing them against a resource.
**Action:** When working with exclusive systems (`&mut World`) and encountering conflicting borrows between `World::query` and `World::get_resource`, prefer wrapping the query iteration inside `world.resource_scope` rather than allocating an intermediate `Vec`.
