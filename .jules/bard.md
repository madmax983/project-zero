## 2025-04-12 - The Missing Module Docs
**Confusion:** Rustdoc generation was throwing 1000+ warnings primarily because modules lack `//!` level documentation and many public items lack `///` documentation.
**Clarification:** I am beginning to add module-level docs and doctests, specifically prioritizing undocumented functions in `src/layer1/entities/` and `src/layer1/architecture/`. Have fixed a vast majority of missing docs across `src/layer1/economy`, `src/layer1/law`, `src/layer1/psychology` and `src/layer1/architecture`.

## 2025-04-12 - Intra-doc Links Broken
**Confusion:** Broken `[TerrainGrid]` style links were causing rustdoc warnings because the items were not imported or linked correctly.
**Clarification:** Wrapped these struct names in backticks instead of square brackets to remove the rustdoc linking warnings when the symbols are not currently in scope.

## 2025-04-17 - README Copy-Paste Compilation Error
**Confusion:** The code example for the Oral Tradition feature inside the `README.md` was wrapped in `#[cfg(feature = "nova")]` and `#[cfg(not(feature = "nova"))]` conditional blocks. When users copy-pasted this example into their own project's `main.rs`, the compiler evaluated the conditionals against the user's crate features (which lacked the `nova` feature, or evaluated differently), causing the example to fail to compile or silently print a missing-feature message.
**Clarification:** I have removed the `cfg` feature condition blocks directly from the `README.md`'s Rust example. Examples in documentation should be written as if the required feature is already enabled, as per DX documentation guidelines.
