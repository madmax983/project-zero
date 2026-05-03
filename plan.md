1. **Explore the current implementation:**
   - I have found that `INT-649` is already in `IN_PROGRESS.md`, claiming the task.
   - The integration uses `trigger_shift_end_system` to map `DayNightCycle` to `ShiftEndEvent`.
   - The `ShiftEndEvent` drives `update_workplace_relationships_system` which then updates relationships.
   - A test `tests/integration/pop_relationships_bridge.rs` was already added, but the event wasn't cleared.
   - I have successfully updated `src/layer1/systems/cleanup.rs` to include `update_event_buffer::<crate::layer1::social::pop_relationships::ShiftEndEvent>` inside `register`.

2. **Run all tests:**
   - `cargo test --all-targets --all-features` has passed without any failures.

3. **Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done:**
   - Run `pre_commit_instructions` tool and obey instructions.

4. **Submit changes:**
   - Commit and submit.
