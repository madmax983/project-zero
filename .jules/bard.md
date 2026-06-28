## 2026-06-22 - [Fixed Broken Intra-Doc Links]
**Confusion:** Module structure comments in `src/layer1/mod.rs` were incorrectly pointing to the old architecture paths, and integration tests for missing bridges were causing compilation errors.
**Clarification:** Updated intra-doc links to correctly resolve to `crate::layer1::architecture::building` and similar submodules. Also, implemented missing `ghost_code_chronicle_bridge` function.

## 2026-06-23 - [Added Missing Module Documentation for Communications and Crafting]
**Confusion:** The `layer2::communications` and `layer1::crafting` modules lacked `//!` module-level documentation and `///` doc tests, acting as "Black Boxes" for developers trying to integrate signal decay/latency or crafting byproducts.
**Clarification:** Added module-level summaries, structural overviews, and executable doctests to both modules. Explained the high-level concepts and how various components interact.
