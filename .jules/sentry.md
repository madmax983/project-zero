**[Bevy Events `get_reader` vs `get_cursor`]**
**Learning:** Bevy `Events::get_reader()` is deprecated. Use `Events::get_cursor()` instead. The cursor's `read(&events)` method returns an iterator, which in unit tests may need to be collected into a vector (`let events_iter: Vec<_> = reader.read(events).collect();`) to assert its length (`events_iter.len()`).
**Action:** When writing Bevy unit tests that assert against event queues, always use `get_cursor()` and explicitly collect the iterator if length assertions are required.
**[Suction Out of Bounds Panic]**
**Learning:** `GridPosition` coordinates (`i32`) must be handled carefully when computing neighboring grid cells to prevent overflow panics. Using `pos.x + dx` inside system loops, especially when checking dynamically generated or unconstrained coordinates, can trigger an `attempt to add with overflow` panic if the initial position is already at `i32::MAX` or `i32::MIN`.
**Action:** Use `.saturating_add()` or explicit boundary/overflow checks before performing arithmetic on `GridPosition` values to safely discard out-of-bounds queries without panicking.
**[Misleading Coverage]**
**Learning:** Some files show 0% logic coverage, but tests might reside in other modules. Use TDD strictly to catch these if adding tests.
**Action:** Continue Red-Green-Refactor to confidently test 0% coverage code.
