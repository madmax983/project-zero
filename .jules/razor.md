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

## [Reduction]
**Bloat:** Reassigning field after default initialization for `ColonyResources` in `tests/integration/temporal_chamber_bridge.rs`
**Cut:** Inline field assignment during default struct instantiation `ColonyResources { fuel: X, ..Default::default() }`
**Saved:** Reduced mutability and fixed clippy warning.

## [Reduction]
**Bloat:** `BeanstalkEvent` enum in `src/layer1/logistics/beanstalk.rs` with 1 variant `Severed`.
**Cut:** Converted the enum into a concrete struct `pub struct BeanstalkEvent { ... }`.
**Saved:** Unnecessary matching and enum namespacing.

## [Reduction]
**Bloat:** `process_ransom_decisions_system` triggering `clippy::too_many_arguments` warning.
**Cut:** Silenced warning with `#[allow(clippy::too_many_arguments)]` instead of abstracting ECS system parameters into a complex tuple/struct, preserving simple dependency injection.
**Saved:** Unnecessary parameter grouping structs and abstraction overhead.

## [Reduction]
**Bloat:** `EspionageOperation` and `CassusBelliReason` single-variant enums in `src/layer3/intellectual_property_wars.rs`.
**Cut:** Converted them to concrete structs `EspionageOperation { invalidated_tech: TechId }` and `CassusBelliReason`.
**Saved:** Boilerplate pattern-matching and enum namespacing.

## [Reduction]
**Bloat:** `TraceKind` enum in `src/layer1/anomalies/cryptid.rs` with unused variant `Fur`.
**Cut:** Converted the enum into a concrete boolean `is_slime` flag inside `TraceItem`.
**Saved:** Unnecessary matching and enum namespacing.
