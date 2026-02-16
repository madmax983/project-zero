# Forge's Journal

## Utility AI God Function Refactor
**Learning:** `evaluate_single_pop` in `src/layer1/utility_ai.rs` was a classic God Function (150+ lines) with a "Pyramid of Doom" (nested `if`s). It manually checked every single action type, making it hard to read and maintain.
**Action:** Refactored into a `ActionEvaluator` struct with helper methods (`evaluate_needs`, `evaluate_work`, `evaluate_specialized_jobs`). This flattened the logic using Guard Clauses and grouped related actions.

## Data Gathering Extraction
**Learning:** `evaluate_actions_system` mixed data gathering (queries) with decision logic, creating a large, cluttered function.
**Action:** Extracted the data gathering logic into `populate_ai_buffer`. This separated concerns and made the system loop cleaner.
