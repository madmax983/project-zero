**2026-03-04 - [Dependency Vulnerability in lru]**
**Threat:** The `lru` crate version 0.12.5 had a vulnerability where `IterMut` violates Stacked Borrows, potentially leading to Undefined Behavior.
**Defense:** `ratatui` version 0.29.0 strictly limits `lru` to `< 0.13`. Bumping `ratatui` to `0.30.0` breaks the WASM platform layer since `ratzilla` relies on older event models. We are unable to bump `lru` independently or without a major codebase rewrite on WASM. Ignoring via audit since `lru` is used for widget cache internally in `ratatui` and does not accept unvalidated user strings/types directly.

**2026-03-04 - [Unmaintained Dependency in paste]**
**Threat:** The `paste` crate is unmaintained (RUSTSEC-2024-0436).
**Defense:** No immediate action taken. It is pulled in as a macro dependency deep in the `wgpu` tree. It poses no runtime or memory safety risk in its current macro-expansion capacity. Ignored in CI via configuration.
