## 2025-04-12 - The Missing Module Docs
**Confusion:** Rustdoc generation was throwing 1000+ warnings primarily because modules lack `//!` level documentation and many public items lack `///` documentation.
**Clarification:** I am beginning to add module-level docs and doctests, specifically prioritizing undocumented functions in `src/layer1/entities/` and `src/layer1/architecture/`.
