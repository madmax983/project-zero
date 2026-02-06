## 2024-05-23 - The Case of the Missing Narratives
**Confusion:** The codebase has functional documentation (what things do) but lacks narrative documentation (why things exist and how they fit together). Specifically, `src/layer1/pop.rs` spawns entities but hides the "why" of their placement logic inside private functions.
**Clarification:** Documenting the *intent* of algorithms (like retry logic for spawning) is just as important as the function signature. I am adding module-level docs and "Hero's Journey" examples to key structs to fix this.

## 2024-05-23 - The Invisible Map
**Confusion:** `src/layer1/map.rs` defined `GridPosition` but didn't explain the coordinate system origin or direction. This requires users to deduce it from rendering code.
**Clarification:** Added explicit "The Coordinate System" section to `map.rs` module docs: (0,0) is Top-Left, Y increases Down (South).
