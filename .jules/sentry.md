# Sentry's Journal

**[Testing Module Redefinition]**
**Learning:** When using Python scripts to append new tests to a file, verify whether a `#[cfg(test)] mod tests { ... }` block already exists in the file. Blindly appending a new `mod tests` block to the bottom of the file causes rustc `E0428` ("the name `tests` is defined multiple times").
**Action:** Use regex replacements that target the end of the existing `mod tests` block (e.g., replacing the final closing brace `}`) or carefully place the new test functions inside the existing scope, rather than creating a duplicate module namespace.

**[Clippy Anti-Patterns in Tests]**
**Learning:** `cargo clippy` with `-D warnings` is strict even inside test modules. Anti-patterns like `assert!(events.len() > 0)` or `assert!(true)` inside match arms trigger warnings (`clippy::len_zero` and `clippy::assertions_on_constants`). Furthermore, clippy enforces `clippy::items_after_test_module`, meaning `mod tests` must always be the absolute last item in the file.
**Action:** When updating tests or formatting files as Sentry, prefer `!is_empty()` over `.len() > 0`, use empty blocks `{}` instead of `assert!(true)` for intentional no-ops, and ensure the `mod tests` declaration remains at the very bottom of the source file.

**[Utility AI Evaluation Context]**
**Learning:** `evaluate_actions_system` relies on `UtilityAIBuffer` which is populated by `collect_pop_data` and component-specific populate functions (like `populate_hospitals`). These populate functions often require specific components (like `PowerConsumer { active: true }` and `Hospital` for hospitals). If you manually test action evaluations by inserting a pre-populated `UtilityAIBuffer` in your test, remember that the system clears and repopulates the buffer from the World state during evaluation, so your manual inserts will be overwritten! You must spawn the required components correctly so the populate functions find them, OR explicitly bypass the full system and test the sub-functions if needed.
**Action:** When testing `UtilityAI` logic, ensure all entities spawned to satisfy goals (e.g. Hospitals, Farms) have the exact components required by the `populate_<entity_type>` functions in `utility_ai_population.rs` so they are correctly detected and added to the evaluation buffer.
