**[Pyramid of Doom in Iterators]**
**Learning:** High-density nested `if let Some` loops block readability and increase indentation needlessly. Deep nesting makes code look like spaghetti logic instead of a simple iteration pipeline.
**Action:** Invert conditionals using `else { continue; }` and unpack `Option` directly via `and_then()` chaining instead of nesting `match`/`if let`.

**[Boolean Blindness in Context Builders]**
**Learning:** Copy-pasting inline context structs inside systems breaks DRY.
**Action:** Extract reusable context-builder helper methods that manage lifetimes clearly, ensuring the "Gather" phase of data extraction is localized to one helper instead of duplicated across standard systems.
**[God Functions in AI and Building Logic]**
**Learning:** Functions like `evaluate_group_work` in `utility_ai.rs` and `spawn_building` in `building.rs` had become "God Functions", spanning over 100 lines and handling multiple distinct logical steps (e.g. evaluating different types of work, or inserting dozens of components for buildings). This makes them hard to read and test.
**Action:** Extract large functional blocks into smaller, named private helper functions (e.g. `evaluate_production`, `evaluate_policing`, `insert_base_building_components`). This flattens the structure and clearly documents the phases of execution.
