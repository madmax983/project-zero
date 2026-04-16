## [Reduction]
**Bloat:** [The over-engineered pattern: PlacementError enum for single-site internal validation]
**Cut:** [The simplified solution: Returning `Result<(), &'static str>` directly from `validate_building_placement`]
**Saved:** [Lines of code / Cognitive load: Removed an enum and a match statement indirection.]
