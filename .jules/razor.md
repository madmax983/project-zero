## [Reduction]
**Bloat:** `InspectorBuilder`
**Cut:** Just return the constraints and widgets and layout them directly.
**Saved:** TBD

## [Reduction]
**Bloat:** `InspectorBuilder` in `src/ui/inspector.rs`
**Cut:** Flattened the builder struct and impl block into simple local `Vec`s with macro helpers `push!` and `push_min!` inside `render_entity_inspector`.
**Saved:** 50+ lines of builder boilerplate, eliminated a cognitive overhead jump.

## [Reduction]
**Bloat:** `Rumor` enum in `src/layer1/culture/nostalgia.rs` with 1 variant `PastGlory`.
**Cut:** Removed the enum and the `rumor` field from `RumorSpreadEvent`.
**Saved:** Unnecessary matching and plumbing for a single concept.

## [Reduction]
**Bloat:** `Edict` enum in `src/layer1/law/rogue_ai_arbitration.rs` with 1 variant `MaximizeFoodProduction`.
**Cut:** Converted the enum into a concrete unit struct `pub struct Edict;`.
**Saved:** Boilerplate pattern-matching and enum namespacing.
