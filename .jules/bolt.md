# Bolt Learnings

## System Parameter Local Optimization
**Learning:** Instantiating new HashMaps or Vecs inside a hot system causes frame-by-frame heap allocations which can degrade performance. Using `Local<T>` stores the data across frames so its capacity can be reused by simply calling `.clear()` each frame.
**Action:** Always check systems for `HashMap::new()` or `Vec::new()` and consider extracting them into a `Local<T>` system parameter.

## Documentation Comments on Parameters
**Learning:** Rust does not allow `///` (doc comments) on function parameters. This will cause a compilation error.
**Action:** When commenting inside a function signature to explain an optimization to a parameter, use standard comments (`//`) instead of doc comments (`///`).

## HashMap Clone Avoidance vs. Mutability
**Learning:** Optimizing `HashMap` clones (like `ts.techs.clone()`) by attempting to use an iterator with a filter on a borrowed reference can run afoul of the borrow checker if the system also mutates other parts of the `World` (e.g. `query.iter_mut(world)`). Sometimes deriving `Copy` on small structs and cloning them is the cleanest path without redesigning the system architecture or isolating queries.
**Action:** When removing `.clone()` on a `HashMap` extracted from a `World` resource in a system that also performs mutable queries, either execute the query and collect the results separately from the mutation pass, or accept the clone if the dataset is small and redesigning would increase complexity disproportionately. Alternatively, derive `Copy` on simple struct elements instead of cloning.

## Exclusive System Memory Allocation Avoidance
**Learning:** Exclusive Bevy systems (`&mut World`) cannot take `Local<T>` system parameters. Allocating vectors or hashmaps inside them (e.g. `Vec::new()`) causes heap allocations every frame.
**Action:** Extract a dedicated Bevy `Resource` to hold the collections. Remove it at the start of the system (`world.remove_resource::<Buffer>().unwrap_or_default()`), clear its internal collections instead of dropping them, and re-insert it at the end of the system to maintain a high-water mark capacity without reallocating.
