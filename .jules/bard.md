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
## 2026-07-02 - [README Formatting]
**Confusion:** I attempted to make the `Cargo.toml` example in the README a doctest. But it's actually just meant to show the `Cargo.toml` config, and making it a doctest means it is visible as rust code.
**Clarification:** Use `toml` instead of `rust,ignore` for Cargo.toml config snippets to prevent rustdoc from processing them while maintaining readability.
## 2026-07-02 - [Oral Tradition Doctest Feature Gate]
**Confusion:** I attempted to make the `Cargo.toml` example in the README a doctest by removing `ignore`. But it's actually just meant to show the `Cargo.toml` config, and making it a doctest means it is visible as rust code. Also, attempting to bypass feature gates by inserting dummy `#[deprecated]` structs into the main codebase was a bad idea and pollutes the API.
**Clarification:** Use `toml` instead of `rust,ignore` for `Cargo.toml` config snippets to prevent rustdoc from processing them while maintaining readability. Leave the feature-dependent example as `rust,ignore`.
## 2026-07-03 - [Nova Feature Missing Types Confusion]
**Confusion:** Users attempting to run the Oral Tradition 'nova' feature snippet from the README without enabling the `nova` feature in their Cargo.toml expected helpful deprecation warnings, but instead encountered raw `E0422` compiler panics because the fallback struct stubs were missing from `src/prelude.rs`.
**Clarification:** Dummy `#[deprecated]` stubs must not be added to the main codebase just to force compilation for feature-gated code examples. Instead, we explicitly document the requirement by placing a large `⚠️ REQUIRES FEATURE NOVA` banner in the README right above the snippet and hiding the feature gate in the Rust snippet using `# #![cfg(feature = "name")]`.

## 2024-07-10 - [NarrativeError::to_table returns String, not Table]
**Confusion:** Users saw that the `NarrativeError` `.to_table()` method prints a beautiful table, and assumed it returns `comfy_table::Table` directly (exposing an internal dependency). They tried to handle it using `if let Some(table) = e.to_table()` which resulted in mismatched types, causing frustration.
**Clarification:** `to_table` returns a `String`. Documented the return type explicitly in `src/shared/narrative.rs` with an executable doctest to demonstrate that it returns a `String`, preventing users from assuming they need to add `comfy-table` to their dependencies.
