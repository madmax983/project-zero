## 2025-04-12 - The Missing Module Docs
**Confusion:** Rustdoc generation was throwing 1000+ warnings primarily because modules lack `//!` level documentation and many public items lack `///` documentation.
**Clarification:** I am beginning to add module-level docs and doctests, specifically prioritizing undocumented functions in `src/layer1/entities/` and `src/layer1/architecture/`. Have fixed a vast majority of missing docs across `src/layer1/economy`, `src/layer1/law`, `src/layer1/psychology` and `src/layer1/architecture`.

## 2025-04-12 - Intra-doc Links Broken
**Confusion:** Broken `[TerrainGrid]` style links were causing rustdoc warnings because the items were not imported or linked correctly.
**Clarification:** Wrapped these struct names in backticks instead of square brackets to remove the rustdoc linking warnings when the symbols are not currently in scope.
