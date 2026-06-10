**[Bevy Events `get_reader` vs `get_cursor`]**
**Learning:** Bevy `Events::get_reader()` is deprecated. Use `Events::get_cursor()` instead. The cursor's `read(&events)` method returns an iterator, which in unit tests may need to be collected into a vector (`let events_iter: Vec<_> = reader.read(events).collect();`) to assert its length (`events_iter.len()`).
**Action:** When writing Bevy unit tests that assert against event queues, always use `get_cursor()` and explicitly collect the iterator if length assertions are required.
