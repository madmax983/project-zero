**[evaluate_shower Warning Resolution]**
**Learning:** Functions that return options for AI evaluation (like `evaluate_shower`) can be silently ignored and never called if not integrated into the AI's execution path logic, throwing a `dead_code` warning.
**Action:** Always verify that newly implemented action evaluation functions are actively called inside `utility_ai.rs`'s `evaluate_group_*` functions, and include tests showing how they score specific target entities accurately based on needs and constraints.

**[Spawn Confetti Panic Mitigation]**
**Learning:** Selecting random elements from a fixed array using `.choose(&mut rng).unwrap()` is a ticking time bomb. Even if the array is currently non-empty, future refactoring could accidentally make it empty and cause unexpected panics during purely visual effects (like spawning confetti).
**Action:** Always replace `.unwrap()` with a safe fallback like `.unwrap_or(&default_value)` when picking random visual configurations.
**[Combat Damage Assertions Drift]**
**Learning:** Tests checking hard-coded max/min bounds for combat damage (e.g., `(dmg - 20.0)`) can silently start failing if core multipliers (`CRIT_MULTIPLIER`) are changed elsewhere in the codebase.
**Action:** Always derive expected test assertions from the defined constants (`CRIT_MULTIPLIER`, `CRIT_CHANCE`) rather than hardcoding resulting values.
**[Mental Break Coverage Improvements]**
**Learning:** Testing ECS utility functions (like `evaluate_mental_break`) that accept complex context types (`PopEvalData`, `UtilityAIBuffer`) requires robust default factories to prevent boilerplate sprawl and ensure isolation across test variants.
**Action:** Always create `default_data()` and `default_buffer()` factories when testing ECS evaluation logic to keep test cases concise and focused on the mutated parameters.
