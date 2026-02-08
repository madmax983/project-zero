# Forge's Journal

## Refactoring `src/layer1/execution.rs`

**Learning:** Large systems often mix decision logic, execution logic, and side effects. This makes them hard to read and test.
**Action:** Extracting logic into small, named private helper functions (e.g., `calculate_work_amount`, `execute_work_on_designation`) dramatically improves readability of the main system function. It allows the reader to understand *what* is happening without getting bogged down in *how*.

**Learning:** `match` statements can often be simplified by extracting arms into helpers, especially if some arms share cleanup logic.
**Action:** Consolidated cleanup logic (removing `MovementTarget` and `AtTarget`) by using a flag `keep_target` instead of repeating the removal in every arm.
