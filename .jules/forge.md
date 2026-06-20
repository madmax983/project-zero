**[Appeasing Clippy on Setup/Routing Functions]**
**Learning:** Monolithic setup functions like `register` in `src/layer1/systems/*`, `init_simulation_resources` in `src/simulation.rs`, and test `setup_world` blocks often trigger `clippy::too-many-lines`. However, splitting these blocks into helper functions can cause pervasive `cannot find type/value` scope and import errors due to internal crate macros and restricted visibility.
**Action:** For configuration/setup functions composed exclusively of long lists of `.add_systems()` or `world.init_resource()` calls (where complexity and branching logic is minimal), use `#[allow(clippy::too_many_lines)]` rather than extracting arbitrary sub-functions to satisfy the linter without introducing brittle abstractions or breaking compilation.

**[Refactoring Pyramids of Doom]**
**Learning:** When deeply nested if/else or match blocks exist alongside query extractions inside a Bevy system (like `execute_demolish`), extracting them out into named helper functions (like `try_scavenge_ruin`) allows us to use early returns/guard clauses which flattens the structure and reduces cognitive load drastically.
**Action:** Always favor guard clauses (`let Some(x) = opt else { return };`) and early returns inside extracted helper functions instead of continuing the Pyramid of Doom.

**[Extracting World Borrows]**
**Learning:** Sometimes repeated blocks of logic are necessary just to copy data out of Bevy's `World` so that subsequent queries can safely iterate over the `World` mutably without causing a borrow checker panic. Extracting this copy logic into a helper function cleans up the main function drastically without changing any behavior.
**Action:** Identify repeated `world.resource::<T>()` copies and extract them into helper functions like `get_terrain_tiles_in_radius`.

**[Extracting Match Arms]**
**Learning:** Large monolithic match statements serving as routers (like `handle_command` in headless tools) can quickly become God Functions.
**Action:** Extract large inline logic from match arms into focused helper functions. Ensure enums used in match arms are fully checked for all variants or use an explicit fallback.
**[Refactoring Pyramids of Doom in Match Statements]**
**Learning:** Monolithic `match` statements that perform near-identical logic for dozens of variants (e.g., repeatedly calling `resources.add_X(amount)` for every resource type) create massive Pyramids of Doom and reduce readability.
**Action:** When a struct (like `ColonyResources`) exposes a unified helper method (like `add_resource(&ResourceType, f32)`), use it to collapse the massive `match` arm into a single, clean iteration step. Ensure you fully verify the helper method accounts for any edge cases originally handled by the manual `match`.
**[Refactoring Pyramids of Doom in Match Statements]**
**Learning:** Monolithic `match` statements that perform near-identical logic for dozens of variants (e.g., repeatedly calling `resources.add_X(amount)` for every resource type) create massive Pyramids of Doom and reduce readability.
**Action:** When a struct (like `ColonyResources`) exposes a unified helper method (like `add_resource(&ResourceType, f32)`), use it to collapse the massive `match` arm into a single, clean iteration step. Ensure you fully verify the helper method accounts for any edge cases originally handled by the manual `match`.

**[Extracting Component Filters from God Functions]**
**Learning:** Monolithic calculation functions (like `calculate_work_amount`) that sequentially query the Bevy `World` for a dozen different marker components (e.g., `MemeticInfection`, `NeuralLinked`) to build a final modifier multiplier create deeply nested, repetitive "Pyramid of Doom" blocks.
**Action:** Extract the repeated `if world.get::<T>(entity).is_some()` checks into a dedicated `get_status_modifiers` helper function that computes and returns the combined float multiplier. This dramatically flattens the parent function.

**[Extracting Logic Blocks from Bevy Systems]**
**Learning:** Monolithic Bevy systems (like `check_spontaneous_build_system`) often contain distinct logic blocks hidden behind comment headers (e.g., `// 1. Identify potential builders`, `// 2. Process builders`). These systems can become difficult to read as their logic compounds.
**Action:** Extract these distinct blocks into named helper functions (e.g., `identify_potential_builders`, `process_builders`) that accept `&mut World` and return relevant intermediary data. This cleanly pipelines the main system and flattens deep nesting inside the helper functions.
**[Refactoring Repeated Context Building Logic]**
**Learning:** Sometimes repeated blocks of logic are necessary just to build temporary structs for scope or context using complex fallbacks and mappings (e.g. `WorldContext` via `build_context` inside `UtilityAI` buffers). Extracting this repeated mapping/fallback logic directly into a dedicated helper function (like `build_fallback_context`) dramatically simplifies the parent methods calling it, adhering to DRY without risking borrow checker panic.
**Action:** Extract repeated context creation steps with common fallback variables into their own `build_*` helper methods and add `#[allow(clippy::too_many_arguments)]` to the main build method instead of allowing the repetition.
