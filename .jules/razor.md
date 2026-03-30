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
**Bloat:** Speculative Generality / Enterprise FizzBuzz Enums: `MemeticInfection`, `QuirkType`, `XpSource` had 0 or 1 variants and were needlessly verbose. `MimicState`, `Decision`, `MerchantType`, `OreType`, `SyzygyPhase` were binary states masquerading as enums.
**Cut:** Flattened 0/1 variant enums into unit structs (`struct MemeticInfection;`, `struct QuirkType;`, `struct XpSource;`). Converted 2-variant enums into simple boolean flags (`is_revealed`, `is_accepted`, `is_smuggler`, `is_whispering`, `is_active`) on their respective components/events. Removed `Decision` entirely.
**Saved:** Unnecessary indirection, matching boilerplate, and type definitions across 10+ files. Simplified logic to direct boolean evaluations (`if is_active {}`).
