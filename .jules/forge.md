**[Pyramid of Doom in Iterators]**
**Learning:** High-density nested `if let Some` loops block readability and increase indentation needlessly. Deep nesting makes code look like spaghetti logic instead of a simple iteration pipeline.
**Action:** Invert conditionals using `else { continue; }` and unpack `Option` directly via `and_then()` chaining instead of nesting `match`/`if let`.

**[Boolean Blindness in Context Builders]**
**Learning:** Copy-pasting inline context structs inside systems breaks DRY.
**Action:** Extract reusable context-builder helper methods that manage lifetimes clearly, ensuring the "Gather" phase of data extraction is localized to one helper instead of duplicated across standard systems.

**[God Functions in AI and Building Logic]**
**Learning:** Functions like `evaluate_group_work` in `utility_ai.rs` and `spawn_building` in `building.rs` had become "God Functions", spanning over 100 lines and handling multiple distinct logical steps (e.g. evaluating different types of work, or inserting dozens of components for buildings). This makes them hard to read and test.
**Action:** Extract large functional blocks into smaller, named private helper functions (e.g. `evaluate_production`, `evaluate_policing`, `insert_base_building_components`). This flattens the structure and clearly documents the phases of execution.

**[God Functions in Building Placement and Hauling]**
**Learning:** `try_place_building` in `building.rs` and `handle_drop_off_item` in `hauling.rs` had grown into monolithic "God Functions", managing everything from tech prerequisite checking and resource deduction to entity despawning, item creation, and logging. This excessive responsibility makes testing difficult and masks core flow.
**Action:** Extract specific phases (e.g. `check_tech_requirements`, `deduct_building_cost`, `try_drop_off_gene_bank`) into helper functions using early returns. This flattens conditionals and transforms large functions into clear, declarative pipelines.

**[Refactoring evaluate_haul and handle_pickup]**
**Learning:** `cargo clippy --fix` on test code might create missing fields when replacing struct initialization with defaults.
**Action:** Be careful to limit refactoring scopes or fix lints manually when involving tests that rely heavily on `Default::default()`.

**[Workspace Test Failure Handling]**
**Learning:** Some integration tests may fail unrelated to hauling refactors due to missing resources (e.g. `ColonyDebt` or `SpawnGhostFleetEvent`) or timing changes.
**Action:** Since these are unrelated pre-existing or timing failures, document them and proceed with the refactor PR.
