
**[Deriving Copy for Light Bevy Resources]**
**Learning:** Found multiple small, plain-old-data structs (`ColonyResources`, `DayNightCycle`, `UtilityConfig`) being explicitly `.clone()`'d in the hot path of the `evaluate_actions_system` during parallel context construction. Explicit cloning on non-Copy types, even small ones, carries conceptual bloat and potential optimization hurdles compared to standard bitwise stack-copies for `Copy` types. By deriving `Copy`, we communicate to the compiler and the developer that this data is just simple bytes.
**Action:** When creating new lightweight `Resource` structs containing only numbers or small enums, always add `Copy` to the derive macros alongside `Clone` to allow safe, fast dereferencing (`*world.resource::<T>()`) instead of `.clone()`.

**Avoid deep copying Bevy Resource HashMaps in per-tick queries**
**Learning:** Calling `.map(|f| f.map.clone())` on a `world.get_resource::<Factions>()` inside a per-tick system creates a significant performance overhead by heap-allocating and deep copying the entire internal `HashMap` every frame. The borrow checker error (E0502) that led to this pattern can be avoided by tightly scoping the mutable and immutable accesses.
**Action:** Keep the `Option<&Resource>` reference and drop it or separate the query iteration by structuring the code so the immutable borrow of the resource and the mutable borrow of the `world.query()` do not overlap directly, or by pre-collecting into a `Vec` inside an isolated block.
