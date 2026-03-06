## [Reduction]
**Bloat:** Generic Soup (`Input<T>` struct generic over `T` when it was only ever instantiated as `Input<KeyCode>`).
**Cut:** Removed the generic `T` parameter and made the `Input` struct concrete, specifically using `KeyCode` for its hash sets and methods.
**Saved:** Unnecessary type parameter passing (`<KeyCode>`) across `src/shared/input.rs`, `src/setup.rs`, and `src/layer1/direct_link.rs`, reducing cognitive load and simplifying initialization logic.
