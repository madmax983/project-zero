## 2026-07-15 - [NarrativeError Documentation]
**Confusion:** The variants of `NarrativeError` were completely undocumented, leaving users guessing how to resolve errors like `MissingContext`. Furthermore, the `to_table` method lacked an example showing its helpful formatting.
**Clarification:** Added comprehensive `///` comments to each variant of `NarrativeError` explaining the cause and providing actionable fixes (e.g. `Fix: context.insert()`). Also added an executable doctest to `to_table` to demonstrate how the error table renders.
## 2026-07-28 - [Headless Sim and Docs confusion]
**Confusion:** The README examples pointed users to modules they couldn't import or explicitly warned about compiler errors (E0422) that were not the ones emitted by the compiler (E0433) when missing the `nova` feature, leading to silent test executions or confused new users doing headless simulation. Also users assumed `NarrativeError::to_table()` existed because the error printed beautifully but had no `.to_table()` method.
**Clarification:** Corrected `README.md` to reference `E0433`, documented the automatic `Display` formatting of `NarrativeError` using an executable doctest so users stop looking for `.to_table()`, and exported `crate::layer1::buildings` directly in `scale::prelude` to ease headless developer friction.
## 2026-08-11 - [Narrative Generator Documentation Update]
**Confusion:** Missing documentation for the `NarrativeGenerator`, `NarrativeContext`, `Template`, and `FragmentType` structs. Also lacking examples. Unused variables in examples caused clippy to fail. Duplication of struct doc strings due to placing them before and after derives.
**Clarification:** Rewrote documentation to include module level, struct, and function level `///` doc strings. Removed duplicate doc strings on `Template` and `FragmentType` and resolved unused variables in doc test examples. Fixed `load_from_files` doc test.
## 2026-08-14 - [Headless Simulation Prelude Exports]
**Confusion:** Users attempting to abstract headless simulation logic into functions taking `&mut World` encountered `E0425` (cannot find type `World` in this scope) because standard Bevy ECS types were not exported in the prelude.
**Clarification:** Re-exported core Bevy ECS types (`World`, `Query`, `Commands`, `Res`, `ResMut`, `Entity`, `Component`) in `scale::prelude` and added module-level `## Examples` documentation to clarify their usage.
## 2026-08-15 - [README CFG Feature Fix]
**Confusion:** The `# #![cfg(feature = "nova")]` line in the `README.md` example caused a compilation error (`error: expected one of \`!\` or \`[\`, found \`#\``) when users copy-pasted it.
**Clarification:** Removed the offending line from `README.md` since the warning banner above it is sufficient.
