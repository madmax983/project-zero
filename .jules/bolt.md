# Bolt Journal

**Performance via pre-allocation**
**Learning:** Re-allocating `Vec` instances inside a `for` loop that runs every tick is slow and causes excessive heap allocations. It's especially noticeable when doing graph traversal or grid analysis.
**Action:** Lift `Vec` declarations outside the loop and use `.clear()` inside the loop instead. If feasible, also use references to passed-in buffers to reuse allocation capacity across frames.
