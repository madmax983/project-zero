**[Refactoring Execution Layer and Pre-emptive Cleanup]**
**Learning:** Simplify loops by making the general case handle edge cases. In `movement_system`, handling `current == target` inside the `calculate_next_position` (returning `current`) allowed removing an explicit "already there" check, reducing duplication and potential bugs where logic diverges.
**Action:** When extracting helpers, consider if the helper can handle the "identity" or "no-op" case naturally to avoid surrounding `if` checks.
