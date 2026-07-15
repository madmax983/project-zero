## 2026-07-15 - [NarrativeError Documentation]
**Confusion:** The variants of `NarrativeError` were completely undocumented, leaving users guessing how to resolve errors like `MissingContext`. Furthermore, the `to_table` method lacked an example showing its helpful formatting.
**Clarification:** Added comprehensive `///` comments to each variant of `NarrativeError` explaining the cause and providing actionable fixes (e.g. `Fix: context.insert()`). Also added an executable doctest to `to_table` to demonstrate how the error table renders.
