## 2026-03-10 - Resolving private intra-doc links

**Confusion:** Many public enums and modules (like `ActionType`) linked to private evaluation functions (`evaluate_*`) using intra-doc links (`[`foo`]`), which caused `cargo doc` warnings and broken links for consumers. Additionally, an implicit code block warning in `mod.rs` was tricky to isolate.
**Clarification:** To fix this without losing context, I converted `[`foo`]` intra-doc links for private items to standard inline code blocks (`` `foo` ``). This prevents `cargo doc` from attempting to resolve them, avoiding warnings while retaining the text reference for code maintainers.
