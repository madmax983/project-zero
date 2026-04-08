## [Reduction]
**Bloat:** `GasType` enum with only one variant (`Smog`) and unnecessary wrapper methods `get_gas` and `set_gas` on `AtmosphereGrid`.
**Cut:** Removed `GasType` enum and the wrapper methods, calling `get` and `set` directly on the grid.
**Saved:** Unnecessary indirection and 15 lines of speculative code.

## [Reduction]
**Bloat:** Layer Lasagna: `evaluate_tame` in `src/layer1/husbandry.rs` simply wrapped `evaluate_candidates` with an extra layer.
**Cut:** Deleted `evaluate_tame` and invoked `evaluate_candidates` directly in callers (`utility_ai.rs` and `utility_types.rs`).
**Saved:** 5 lines of useless indirection.

## [Reduction]
**Bloat:** Empty Struct / Speculative Generality: `RhythmManager` in `src/layer1/tech/rhythm.rs` was an empty struct injected into the world for no reason.
**Cut:** Deleted the `RhythmManager` struct and removed its registration from tests.
**Saved:** 5 lines of boilerplate setup.

## [Reduction]
**Bloat:** Enterprise FizzBuzz Enum: `ScentType` in `src/layer1/olfactory.rs` only had two variants (`Pleasant` and `Foul`).
**Cut:** Converted `ScentType` to a simple boolean flag `is_pleasant` on `ScentEmitter`.
**Saved:** 6 lines and simplified pattern matching to basic `if/else`.

## [Reduction]
**Bloat:** OOP-style `Manager` suffixes for ECS resources (`ElectionManager`, `SanctuaryManager`, `ShadowMarketManager`).
**Cut:** Renamed to descriptive, data-oriented state names (`ElectionCycle`, `ActiveSanctuaries`, `ShadowMarketCooldown`) per Bevy/Razor Architecture Insight.
**Saved:** Eliminated implicit OOP terminology overhead, improving clarity and adherence to Bevy ECS conventions.

## [Reduction]
**Bloat:** `ShipType` enum in `src/layer1/shipbreaking.rs` which was only used for test simplicity with 2 variants.
**Cut:** Removed the `ShipType` enum, its usage from `SpawnCrashedShipEvent`, and assignments in tests.
**Saved:** 7 lines of code and speculative test-only complexity.

## [Reduction]
**Bloat:** Enterprise FizzBuzz Abstraction: `GeomeManager`, `GeomeType`, `ZLevel`, and `Rect` structs in `src/layer1/geomes.rs`.
**Cut:** Deleted the entire manager layer and speculative structs. Replaced with a single `fill_rect` function in `src/layer1/nature/terrain.rs` that directly sets the `TerrainType`.
**Saved:** 71 lines of code, speculative wrapper types, and unnecessary indirection.

## [Reduction]
**Bloat:** Enterprise FizzBuzz Abstraction: `AdministrationLevel`, `HighSpeechEnabled` resources and `WorkDelay` component in `src/layer1/bureaucracy.rs`.
**Cut:** Deleted the entire `src/layer1/bureaucracy.rs` module and removed its integration in `src/layer1/execution/general_work.rs` and `src/layer1/systems/execution.rs`.
**Saved:** 148 lines of code, speculative mechanics, and unnecessary indirection.

## [Reduction]
**Bloat:** Enterprise FizzBuzz Speculative Generality: `DebrisConfig` struct in `src/layer2/debris.rs` marked as `// Placeholder for now`.
**Cut:** Deleted the `DebrisConfig` empty placeholder struct.
**Saved:** 2 lines of unused setup and speculative overhead.
## [Reduction]
**Bloat:** Speculative Generality: `MemeticInfection` enum in `src/layer1/memetics/parasitic_broadcast.rs` and `BanishmentState` enum in `src/layer1/social/exile.rs` which had only 1 variant.
**Cut:** Converted both enums into simple marker structs and updated Bevy queries to use `With<T>` rather than retrieving data and matching on it.
**Saved:** 5 lines of code, speculative pattern matching overhead, and improved idiomatic Bevy usage by using ECS filters instead of iterating and checking values.
## [Reduction]
**Bloat:** Layer Lasagna (Deep folder hierarchy for `tectonic.rs`)
**Cut:** Flattened `src/layer1/geology/tectonic/tectonic_tests.rs` into `src/layer1/geology/tectonic.rs` and deleted the `tectonic` directory.
**Saved:** Removed unnecessary nested folder structure, making the module flat and easier to navigate without "Russian doll" files.
## [Reduction]
**Bloat:** Speculative Optimization / Clever Code: `Traits` component in `src/layer1/traits.rs` using a `u64` bitmask to store a rapidly growing `Trait` enum (over 50 variants), requiring manual bitwise logic (`Traits(1 << (Trait::... as u8))`) across the codebase.
**Cut:** Replaced the `u64` bitmask with a standard `bevy::utils::HashSet<Trait>`. Updated methods to use standard set operations (`insert`, `remove`, `contains`) and replaced all bitwise initializations with explicit builder patterns (`Traits::default().add(...)`). Retained deterministic iteration by yielding over the enum and filtering by the set.
**Saved:** Eliminated cognitive load of manual bitwise operations, prevented an imminent integer overflow (capped at 64 traits), and standardized component initialization.
## [Reduction]
**Bloat:** `Resolution` enum in `src/layer3/council.rs` used to denote variants in a Vec but not meaningfully matched anywhere.
**Cut:** Removed the `Resolution` enum entirely and replaced `active_resolutions: Vec<Resolution>` in `GalacticCouncil` with a simple boolean `has_active_resolutions`.
**Saved:** 7 lines of code and speculative wrapper overhead.

## [Reduction]
**Bloat:** `Clause` enum in `src/layer3/diplomacy.rs` had only two variants where only `SporePropagation` was ever used or checked.
**Cut:** Deleted the `Clause` enum entirely and replaced `clauses: Vec<Clause>` in `Treaty` with a simple boolean `has_spore_propagation`. Updated test and system logic.
**Saved:** 8 lines of code and speculative wrapper/vec overhead.

## [Reduction]
**Bloat:** `DiplomaticTrait` enum in `src/layer3/diplomacy_reflection.rs` stored as an array of traits, matched frequently with `.contains(&DiplomaticTrait::X)`.
**Cut:** Replaced the enum and vector entirely with flat boolean fields (`is_warlike`, `is_barbarian`, `is_ecological`, `is_pacifist`) directly on `DiplomaticTraits`.
**Saved:** Removed speculative enumeration complexity and vector overhead, resulting in 24 fewer lines across tests and component logic.

## [Reduction]
**Bloat:** `LeaderTrait` enum in `src/layer3/diplomacy/succession.rs` containing variants (`Pacifist`, `Militant`, `Cruel`), which served no mechanical purpose other than optional flavor text in a single Chronicle event.
**Cut:** Excised `LeaderTrait` enum entirely. Removed `leader_trait` from `SuccessionEvent` and integration bridges.
**Saved:** 15 lines of code, one less `#[derive]`, and removed an unnecessary text-formatting path.

## [Reduction]
**Bloat:** `GeologicalEvent` enum in `src/layer1/geology/mod.rs` containing variants (`Tremor`, `Earthquake`), where `Tremor` was never constructed or utilized.
**Cut:** Converted `GeologicalEvent` from an enum into a standard struct (representing just an `Earthquake`). Removed pattern matching wrappers at instantiation sites across `src/layer1/geology/mod.rs`, `src/layer1/seismic.rs`, and `src/layer1/geology/tectonic.rs`.
**Saved:** 7 lines of code, speculative pattern matching overhead, and improved explicit ECS communication by eliminating unused event variants.
