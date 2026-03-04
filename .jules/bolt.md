
**[Deriving Copy for Light Bevy Resources]**
**Learning:** Found multiple small, plain-old-data structs (`ColonyResources`, `DayNightCycle`, `UtilityConfig`) being explicitly `.clone()`'d in the hot path of the `evaluate_actions_system` during parallel context construction. Explicit cloning on non-Copy types, even small ones, carries conceptual bloat and potential optimization hurdles compared to standard bitwise stack-copies for `Copy` types. By deriving `Copy`, we communicate to the compiler and the developer that this data is just simple bytes.
**Action:** When creating new lightweight `Resource` structs containing only numbers or small enums, always add `Copy` to the derive macros alongside `Clone` to allow safe, fast dereferencing (`*world.resource::<T>()`) instead of `.clone()`.
