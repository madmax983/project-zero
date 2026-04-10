**[HashSet Preallocation]
**Learning:** Avoid dynamic hash map resizing for large query results. Even without complex logic, simple `HashSet::new()` allocations inside frequently run systems (`apply_door_movement_penalties_system`) incur unnecessary O(N) allocation penalties. Using `door_query.iter().len()` to pre-allocate capacity is an O(1) operation in Bevy and a safe, zero-cost abstraction.
**Action:** Always check the exact size of the incoming iterator/query and use `HashSet::with_capacity` or `Vec::with_capacity` instead of `new()`.
