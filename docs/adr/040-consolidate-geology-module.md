# 40. Consolidate Geology Module

Date: 2026-03-17

## Status

Accepted

## Context

The `src/layer1/geology.rs` file was growing in complexity, and tests were creating a tangle within the single file. It needed to be split into a dedicated directory module to better encapsulate tectonic stress, seismic grids, and associated tests.

## Decision

We have decided to consolidate the geology module from a single file (`src/layer1/geology.rs`) into a directory module (`src/layer1/geology/`).
The `SeismicGrid` and long-term stress tracking now live in `src/layer1/geology/mod.rs`, while tectonic logic is extracted into `src/layer1/geology/tectonic.rs` and its tests into `src/layer1/geology/tectonic/tectonic_tests.rs`.

## Consequences

### Positive
*   **Maintainability**: Improved separation of concerns and test encapsulation. The `geology` module is now cleanly structured.
*   **Readability**: Smaller, more focused files.

### Negative
*   **Refactoring Overhead**: Existing imports pointing to `crate::layer1::geology` may need to be updated depending on what was moved to submodules, though re-exports in `mod.rs` can mitigate this.