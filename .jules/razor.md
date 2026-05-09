## [Reduction]
**Bloat:** `InspectorBuilder`
**Cut:** Just return the constraints and widgets and layout them directly.
**Saved:** TBD

## [Reduction]
**Bloat:** `InspectorBuilder` in `src/ui/inspector.rs`
**Cut:** Flattened the builder struct and impl block into simple local `Vec`s with macro helpers `push!` and `push_min!` inside `render_entity_inspector`.
**Saved:** 50+ lines of builder boilerplate, eliminated a cognitive overhead jump.
