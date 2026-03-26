# 047. Encapsulate Economy Domain

## Status

Proposed

## Context

The `layer1::mod.rs` file was suffering from the "Blob" anti-pattern, acting as a massive catch-all for various unrelated submodules. Specifically, 9 submodules heavily related to the economy (`trade`, `resources`, `refining`, `hauling`, `items`, `stockpile`, `inventory`, `black_market`, and `shadow_market`) were cluttering the root namespace of `layer1`. This lack of cohesion made it difficult to manage and comprehend the economic subsystem's boundaries.

## Decision

We extracted the 9 economy-related submodules out of the root `src/layer1/mod.rs` and moved them into a new, dedicated `src/layer1/economy/` domain module. We also updated the corresponding import paths across the codebase.

## Consequences

- **High Cohesion:** The economic subsystems are now logically grouped together, making the code easier to navigate and maintain.
- **Clear Boundaries:** Enforces strict domain boundaries for the economy, preventing external systems from tightly coupling to internal implementation details.
- **Reduced Clutter:** The root `layer1::mod` is significantly cleaner and more focused on orchestrating the high-level sub-domains.
- **Import Changes:** Required modifying `use` statements throughout the codebase to reference the new `layer1::economy::*` namespace.
