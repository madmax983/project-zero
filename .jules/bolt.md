**[AHash vs SipHash for Coordinates]
**Learning:** `std::collections::HashMap` uses SipHash, which is secure but slow for integer tuples like `(i32, i32)`. The codebase frequently accesses spatial structures using grid coordinates.
**Action:** Always prefer `bevy::utils::HashMap` and `bevy::utils::HashSet` (which use AHash) over the `std::collections` equivalents when indexing spatial data, especially on hot paths like pathfinding and AI lookups.
