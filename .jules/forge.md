## 2024-05-24 - Resource Deduction Readability Smell
**Learning:** Checking massive `ColonyResources` capacity in-place by instantiating a nearly-empty struct creates immense visual clutter (e.g., in `shadow_market.rs`). Similarly, manual bounds checking `if resources.ration >= cost { resources.consume(...) }` is repetitive and prone to error.
**Action:** Implemented `get_amount` and `try_consume` helper methods directly on `ColonyResources`. Replaced inline dummy-struct instantiation with clean, single-line method calls that check capacity and consume atomically, eliminating ~80 lines of boilerplate logic.

## 2024-05-25 - Building Cost Initialization Clutter Smell
**Learning:** Massive struct instantiation using `ColonyResources { metal: 50.0, ..ColonyResources::zeroed() }` within `match` arms inside `src/layer1/architecture/building.rs` creates extreme pyramid-of-doom style visual clutter when the struct has dozens of fields.
**Action:** Implemented the builder pattern on `ColonyResources` (`with_metal()`, `with_wood()`, etc). Replaced all block initializations with clean `ColonyResources::zeroed().with_X(...).with_Y(...)` method chains, compressing logic down to a single line per match arm without changing behavior.
