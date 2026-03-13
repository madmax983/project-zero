## [Reduction]
**Bloat:** Generic Soup (`Input<T>` struct generic over `T` when it was only ever instantiated as `Input<KeyCode>`).
**Cut:** Removed the generic `T` parameter and made the `Input` struct concrete, specifically using `KeyCode` for its hash sets and methods.
**Saved:** Unnecessary type parameter passing (`<KeyCode>`) across `src/shared/input.rs`, `src/setup.rs`, and `src/layer1/direct_link.rs`, reducing cognitive load and simplifying initialization logic.

## [Reduction]
**Bloat:** Bevy system tuple size exceeding 20 elements.
**Cut:** Split the large tuple into two separate `schedule.add_systems(...)` calls.
**Saved:** 1 compilation error.

## [Reduction]
**Bloat:** `clippy::complexity` warnings related to large tuples and unused generic structure in tests and systems.
**Cut:** Split the large 20+ elements tuple inside `src/layer1/systems/observation.rs` and ran `cargo clippy --fix` on `src/layer1/` tests and source code to eliminate redundant `.default()` assignments and unused variables.
**Saved:** Multiple compilation errors related to `IntoSystemConfigs` macro boundaries limit and numerous warnings cluttering output.

## [Reduction]
**Bloat:** The "One-Time" Enum (`GasType` enum in `src/layer1/nature/atmosphere.rs` with exactly one variant `Smog`).
**Cut:** Deleted the `GasType` enum entirely. Renamed generic `get_gas` and `set_gas` methods on `AtmosphereGrid` to explicitly named `get_smog` and `set_smog` methods, removing the redundant `_gas: GasType` argument.
**Saved:** Unnecessary parameter passing, a dead enum, and improved clarity of the atmospheric pollution API.
