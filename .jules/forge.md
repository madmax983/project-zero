**[Appeasing Clippy on Setup/Routing Functions]**
**Learning:** Monolithic setup functions like `register` in `src/layer1/systems/*`, `init_simulation_resources` in `src/simulation.rs`, and test `setup_world` blocks often trigger `clippy::too-many-lines`. However, splitting these blocks into helper functions can cause pervasive `cannot find type/value` scope and import errors due to internal crate macros and restricted visibility.
**Action:** For configuration/setup functions composed exclusively of long lists of `.add_systems()` or `world.init_resource()` calls (where complexity and branching logic is minimal), use `#[allow(clippy::too_many_lines)]` rather than extracting arbitrary sub-functions to satisfy the linter without introducing brittle abstractions or breaking compilation.

**[Refactoring Pyramids of Doom]**
**Learning:** When deeply nested if/else or match blocks exist alongside query extractions inside a Bevy system (like `execute_demolish`), extracting them out into named helper functions (like `try_scavenge_ruin`) allows us to use early returns/guard clauses which flattens the structure and reduces cognitive load drastically.
**Action:** Always favor guard clauses (`let Some(x) = opt else { return };`) and early returns inside extracted helper functions instead of continuing the Pyramid of Doom.
