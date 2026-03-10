**[evaluate_shower Warning Resolution]**
**Learning:** Functions that return options for AI evaluation (like `evaluate_shower`) can be silently ignored and never called if not integrated into the AI's execution path logic, throwing a `dead_code` warning.
**Action:** Always verify that newly implemented action evaluation functions are actively called inside `utility_ai.rs`'s `evaluate_group_*` functions, and include tests showing how they score specific target entities accurately based on needs and constraints.

**[Missing Resources in Tests]**
**Learning:** Tests that call system update functions directly via `run_system_once` may fail if the module changes to require new Res/ResMut parameters that the tests don't initialize on the World. This specifically happened with `update_temperature_system` which required `TerrainGrid` as a resource.
**Action:** Always verify `World` setup initializes all queried resources when testing `RunSystemOnce` execution on specific target functions.

**[Mock Default Baseline Expectations]**
**Learning:** Adding new modifiers or values to structs like `Needs` without setting those same new defaults inside mock instances initialized manually in older test files causes expectation drift (e.g. `hygiene` defaulted to 0.8, increasing overall `morale()` calculation bounds unexpectedly above trigger thresholds like 0.15).
**Action:** When manually writing object struct values for tests, explicitly declare all values if `..Default::default()` is not used or if calculating a specifically balanced metric across all contained inputs is expected.
