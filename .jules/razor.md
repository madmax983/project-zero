## [Reduction]
**Bloat:** [The over-engineered pattern: PlacementError enum for single-site internal validation]
**Cut:** [The simplified solution: Returning `Result<(), &'static str>` directly from `validate_building_placement`]
**Saved:** [Lines of code / Cognitive load: Removed an enum and a match statement indirection.]

## [Reduction]
**Bloat:** `EventType` enum in `src/layer1/geography.rs` used solely to map to Strings.
**Cut:** Removed `EventType` entirely. Replaced it with a direct `event_name: String` inside `HistoricalEvent`.
**Saved:** 7 lines of code and an unnecessary enum indirection.
