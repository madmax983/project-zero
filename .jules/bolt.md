**[Optimizing ECS Iteration Overheads]**
**Learning:** Collecting intermediate structures (like `Vec<Vec<Entity>>`) from Bevy queries just to bypass borrow checker limitations introduces significant heap allocation overhead per frame.
**Action:** When a system needs to pair entities from the same query (like exchanging rumors) and then mutate the `World` via a helper function, flatten the pairs into a single `Vec<(Entity, Entity)>` during query iteration rather than cloning or allocating collections per component group. This reduces allocations to `1` per frame while satisfying borrow rules safely.

**[HashSet Lazy Initialization]
**Learning:** `///` doc comments inside a function body attached to a `let` statement will trigger a `clippy::unused_doc_comments` error because rustdoc does not generate documentation for statements.
**Action:** Always place `///` performance documentation on the function declaration itself or use `//` for inline code blocks.
