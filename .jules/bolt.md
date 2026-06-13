**Preserving Inner Collection Capacities**
**Learning:** When using a buffer struct with nested collections (like `HashMap<Entity, Vec<T>>`) to avoid frame-by-frame memory allocations, calling `.clear()` on the outer `HashMap` drops all of its values (the `Vec`s), completely defeating the purpose of the optimization since their heap capacities are destroyed.
**Action:** When clearing nested collections to reuse their capacity, only call `.clear()` on the inner elements (e.g., iterating over `.values_mut()` and clearing each `Vec`). Do NOT clear the outer map unless you specifically intend to drop the inner allocations.

**Removing Intermediate `.collect::<Vec<_>>()` Chains in Tick Systems**
**Learning:** When a system processes events (like `process_chronicles` handling `Chronicle` events each tick), chaining `.iter().filter(...).collect::<Vec<_>>()` forces a new heap allocation every frame/tick just to iterate over the filtered results.
**Action:** Instead of collecting into an intermediate `Vec`, iterate directly over the filtered iterator. Use simple local variables to track iteration state (like `has_new` and `last_tick`) to apply updates after the loop finishes without fighting the borrow checker.

**Extracting Component Data Before Despawning (Safe Method)**
**Learning:** When a system reads data from a component and then immediately despawns that entity, calling `.clone()` on the data is an unnecessary heap allocation. However, trying to use `world.entity_mut(entity)` to get a mutable reference can cause a runtime panic if the entity is missing. Furthermore, injecting dummy data with `std::mem::replace` is an anti-pattern.
**Action:** Use `world.get_entity_mut(entity)?.take::<T>()?` to safely consume and extract the entire component without panicking or requiring dummy values.

**Optimizing HashMap for Integer Keys**
**Learning:** Using `std::collections::HashMap` with integer keys (like `(i32, i32)`) introduces significant overhead due to its default SipHash algorithm.
**Action:** Always prefer `bevy::utils::HashMap` (which uses the faster AHash algorithm) for non-cryptographic use cases, such as spatial maps or caches using integer coordinates, to improve performance. Ensure to include a comment with `/// ⚡ Bolt Optimization:` to document the change.
