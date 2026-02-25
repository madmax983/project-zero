**[Refactoring Execution Layer and Pre-emptive Cleanup]**
**Learning:** Simplify loops by making the general case handle edge cases. In `movement_system`, handling `current == target` inside the `calculate_next_position` (returning `current`) allowed removing an explicit "already there" check, reducing duplication and potential bugs where logic diverges.
**Action:** When extracting helpers, consider if the helper can handle the "identity" or "no-op" case naturally to avoid surrounding `if` checks.

**[QueryData Refactor in Utility AI]**
**Learning:** Bevy 0.15's `#[derive(QueryData)]` is a powerful tool to eliminate massive tuple unpacking in query iterations. It not only improves readability but also makes the component access strictly typed and named, preventing "index blindness" in long tuples.
**Action:** Identify other "God Queries" with 5+ components and refactor them into named structs using `QueryData`.

**[Generic Buffer Population]**
**Learning:** Repetitive query-and-push patterns for simple components (e.g. `query::<(Entity, &Pos, &T)>`) are prevalent in data-gathering phases. These can be entirely replaced by a single generic function using `query_filtered::<(Entity, &Pos), With<T>>`.
**Action:** Look for other systems that "collect" entities into lists (like rendering or UI data gathering) and apply similar generic collectors.
