# Bolt's Journal

**[RenderCache Allocation]**
**Learning:** Initializing new HashMaps every frame for rendering creates significant memory churn and allocation overhead (3 allocations * frame rate).
**Action:** Use a `RenderCache` resource with `clear()` and reuse the same memory buffers. Use the "Take-Update-Insert" pattern to manage borrow checker rules when updating a resource while iterating the world.
