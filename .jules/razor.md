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
**Bloat:** `ScentMap` struct in `src/layer1/olfactory.rs` wrapping a `HashMap` with no other fields.
**Cut:** Replaced the struct with a tuple struct (`pub struct ScentMap(pub HashMap<GridPosition, TileScent>);`), flattening the wrapper and updating callers to access the inner map with `.0`.
**Saved:** Unnecessary wrapper field and improved code simplicity.

## [Reduction]
**Bloat:** `MiasmaRecordedSecret` in `src/layer1/bio_acoustic_miasma.rs` which just wrapped `secrets: Vec<String>`.
**Cut:** Flattened the wrapper to a tuple struct `MiasmaRecordedSecret(pub Vec<String>)`.
**Saved:** Unnecessary indirection.

## [Reduction]
**Bloat:** `BulletinBoard` in `src/layer1/social/grievances.rs` which was a component wrapping a `notes: Vec<BulletinNote>`.
**Cut:** Converted it to a tuple struct `BulletinBoard(pub Vec<BulletinNote>)`.
**Saved:** Redundant named fields.

## [Reduction]
**Bloat:** `GalacticCouncil` in `src/layer3/council.rs` wrapped an `active_resolutions: Vec<Resolution>`.
**Cut:** Flattened it to `GalacticCouncil(pub Vec<Resolution>)`.
**Saved:** Unnecessary indirection.
