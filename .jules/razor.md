## [Reduction]
**Bloat:** `GasType` enum with only one variant (`Smog`) and unnecessary wrapper methods `get_gas` and `set_gas` on `AtmosphereGrid`.
**Cut:** Removed `GasType` enum and the wrapper methods, calling `get` and `set` directly on the grid.
**Saved:** Unnecessary indirection and 15 lines of speculative code.
