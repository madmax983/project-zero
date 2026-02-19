# Bolt's Journal

**[RenderCache Allocation]**
**Learning:** Initializing new HashMaps every frame for rendering creates significant memory churn and allocation overhead (3 allocations * frame rate).
**Action:** Use a `RenderCache` resource with `clear()` and reuse the same memory buffers. Use the "Take-Update-Insert" pattern to manage borrow checker rules when updating a resource while iterating the world.

**[Optimized Fallback Utility AI]**
**Learning:** Vec::collect in a loop over entities is a hidden performance killer. Reusing a Resource buffer (even if it requires temporarily removing it from the World to satisfy the borrow checker) is a powerful pattern to eliminate per-tick allocations.
**Action:** Look for collect::<Vec<_>>() in hot paths and replace with a Resource-cached buffer. Also, aggressively derive Copy for small components (< 64 bytes) to avoid clone() overhead.

**[Borrow Checker in Test Helpers]**
**Learning:** When writing helper functions for tests that take `&mut World`, be careful not to hold a mutable borrow of a Resource (like `world.resource_mut::<T>()`) while iterating a Query using `world`. This causes a double borrow error.
**Action:** Extract data from the Query into a collection (e.g., `Vec`) first, drop the query borrow, then mutate the Resource using the collected data.

**[Zero-Copy Resource Access]**
**Learning:** Large resources like `ZoneGrid` (40KB+) cloned every frame for read-only access in a mutable system context create significant overhead (10%+).
**Action:** Use `world.remove_resource::<T>()` to temporarily extract the resource, use it by reference, and re-insert it at the end. This avoids allocation/copying entirely while satisfying the borrow checker for concurrent world access.
