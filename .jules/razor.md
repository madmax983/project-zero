## [Reduction]
**Bloat:** [Echo's reported DX issues regarding Story missing Debug, missing imports in Prelude, and scary warnings in README]
**Cut:** [No action needed. All three Confusions reported by Echo were already addressed in previous commits. The components natively derive Debug, exports are present in `src/prelude.rs`, and the README is up-to-date]
**Saved:** [0 Lines of code / Reduced cognitive load of verifying stale issues]

## [Reduction]
**Bloat:** Re-used `TechLevel` struct and enum in different layers.
**Cut:** Renamed `TechLevel` to `BuildingTechLevel` in `grafting.rs` and `PrimitiveTechLevel` in `primitives/mod.rs` to flatten unnecessary shadowing of `layer1::tech::tech_level::TechLevel` and make types globally distinct.
**Saved:** Reduced cognitive load and namespace collisions across the simulation layers.
