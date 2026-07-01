## 2026-06-22 - [Fixed Broken Intra-Doc Links]
**Confusion:** Module structure comments in `src/layer1/mod.rs` were incorrectly pointing to the old architecture paths, and integration tests for missing bridges were causing compilation errors.
**Clarification:** Updated intra-doc links to correctly resolve to `crate::layer1::architecture::building` and similar submodules. Also, implemented missing `ghost_code_chronicle_bridge` function.

## 2026-06-23 - [Added Missing Module Documentation for Communications and Crafting]
**Confusion:** The `layer2::communications` and `layer1::crafting` modules lacked `//!` module-level documentation and `///` doc tests, acting as "Black Boxes" for developers trying to integrate signal decay/latency or crafting byproducts.
**Clarification:** Added module-level summaries, structural overviews, and executable doctests to both modules. Explained the high-level concepts and how various components interact.

## 2026-06-29 - [Clarified Feature Flags for Nova]
**Confusion:** The README incorrectly stated `NarrativeGenerator` required the `nova` feature. Additionally, `oral_tradition.rs` falsely claimed its structs were "always defined" and would emit warnings, rather than properly failing with `E0422`.
**Clarification:** Removed the erroneous `REQUIRES FEATURE NOVA` banner from the base narrative section in `README.md`, made the actual warning a huge banner in the README, and deleted the inaccurate "always defined" claims from the documentation in `src/layer1/oral_tradition.rs`, embracing the compiler error for missing features and adhering to architectural guidelines avoiding bloated dummy fallback code.
## 2026-07-01 - [Added Missing Module Documentation for Silence (Layer 3)]
**Confusion:** The `src/layer3/silence.rs` module lacked module-level `//!` documentation, detailed struct/event `///` comments, and executable examples, leaving the DetectionRisk mechanic as a "Black Box" for developers integrating stealth or power mechanics.
**Clarification:** Added a comprehensive module-level summary explaining the thematic purpose (void entities detecting power/population). Detailed `DetectionRisk` and `HostileSpawnEvent` with examples and formula explanations to remove the mystery.
