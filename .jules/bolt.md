**[AHash vs SipHash for Coordinates]
**Learning:** `std::collections::HashMap` uses SipHash, which is secure but slow for integer tuples like `(i32, i32)`. The codebase frequently accesses spatial structures using grid coordinates.
**Action:** Always prefer `bevy::utils::HashMap` and `bevy::utils::HashSet` (which use AHash) over the `std::collections` equivalents when indexing spatial data, especially on hot paths like pathfinding and AI lookups.
**[Optimizing HashMaps for Primitive Keys]**
**Learning:** For game engines and pathfinding systems using integer or tuple coordinates, `std::collections::HashMap` with default SipHash is excessively slow. Replacing it with `bevy::utils::HashMap` uses AHash which is significantly faster for primitive keys. Combining this with `.with_capacity(size)` entirely eliminates intermediate heap reallocations.
**Action:** Always favor `bevy::utils::HashMap::with_capacity` when accumulating grid coordinates during a system tick to minimize per-frame GC pressure and hashing overhead.
