**[Pyramid of Doom in Iterators]**
**Learning:** High-density nested `if let Some` loops block readability and increase indentation needlessly. Deep nesting makes code look like spaghetti logic instead of a simple iteration pipeline.
**Action:** Invert conditionals using `else { continue; }` and unpack `Option` directly via `and_then()` chaining instead of nesting `match`/`if let`.

**[Boolean Blindness in Context Builders]**
**Learning:** Copy-pasting inline context structs inside systems breaks DRY.
**Action:** Extract reusable context-builder helper methods that manage lifetimes clearly, ensuring the "Gather" phase of data extraction is localized to one helper instead of duplicated across standard systems.