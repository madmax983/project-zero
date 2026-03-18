# 42. Refactor Building God Module

Date: 2026-03-13

## Status

Accepted

## Context

The `src/layer1/building.rs` file had grown into a massive monolith containing a "God Function" called `spawn_building` and its associated `configure_*` helpers. This created a highly coupled system where adding a new building or updating an existing one meant modifying an already overloaded file, leading to potential circular dependencies and poor maintainability.

## Decision

We have decided to refactor the monolithic `building.rs` by extracting the massive `spawn_building` function and its 15 associated `configure_*` helpers. The original file has been converted into a facade (`src/layer1/building/mod.rs`), and the extracted logic was moved into a highly cohesive `src/layer1/building/configuration.rs` submodule.

## Consequences

### Positive
*   **Maintainability**: Breaking down the god module into a structured directory module with submodules like `configuration.rs` significantly improves readability and makes future updates to building configurations easier.
*   **Dependency Management**: Reduces the likelihood of circular dependencies.

### Negative
*   **Module Complexity**: Increases the number of files, requiring proper use of the facade (`mod.rs`) to expose necessary APIs so that external consumers are not disrupted.