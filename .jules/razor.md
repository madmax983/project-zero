## [Reduction]
**Bloat:** Dummy fallback structs (`#[cfg(not(feature = "nova"))]`) and confusing feature flags scattered randomly inside `oral_tradition.rs` just to emit "helpful warnings" for missing compile-time features. This caused namespace collision expectations and was an anti-pattern.
**Cut:** Removed all `#[cfg(feature = "nova")]` scattered inside the file and all `#[cfg(not(feature = "nova"))]` dummy systems. Pushed the feature flag up to the module declaration level in `src/layer1/mod.rs` and `src/prelude.rs` so the entire feature is gated.
**Saved:** 20 lines of dummy code, removed 15 unnecessary feature attributes, entirely flattened the mental model for this feature.
