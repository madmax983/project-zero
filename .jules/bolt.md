# Bolt's Journal

**[RenderCache Allocation]**
**Learning:** Initializing new HashMaps every frame for rendering creates significant memory churn and allocation overhead (3 allocations * frame rate).
**Action:** Use a `RenderCache` resource with `clear()` and reuse the same memory buffers. Use the "Take-Update-Insert" pattern to manage borrow checker rules when updating a resource while iterating the world.

**[Optimized Fallback Utility AI]**
**Learning:** Vec::collect in a loop over entities is a hidden performance killer. Reusing a Resource buffer (even if it requires temporarily removing it from the World to satisfy the borrow checker) is a powerful pattern to eliminate per-tick allocations.
**Action:** Look for collect::<Vec<_>>() in hot paths and replace with a Resource-cached buffer. Also, aggressively derive Copy for small components (< 64 bytes) to avoid clone() overhead.
