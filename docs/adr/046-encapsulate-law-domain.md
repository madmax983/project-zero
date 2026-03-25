# 046. Encapsulate Law Domain

## Status

Proposed

## Context

The Law and Order subsystem (`justice`, `penal`, `predictive_policing`, `contraband`) was previously scattered across the root `layer1` module namespace. This created a sprawl anti-pattern, polluting the top-level directory and making it difficult to understand the boundaries and dependencies of the legal systems within the simulation.

## Decision

We extracted all law-related files into a dedicated `src/layer1/law/` module and updated the corresponding import paths across the codebase.

## Consequences

- **Domain Boundaries:** Strict domain boundaries are enforced, encapsulating all law and order features together.
- **Organization:** Reduces file clutter in the root `layer1` module, improving overall readability and discoverability of subsystems.
- **Maintainability:** Makes it easier to understand the relationships between different legal mechanics (such as predictive policing feeding into justice).
- **Import Changes:** Requires downstream code (e.g., utility AI, event systems) to update their import paths to the new `layer1::law::*` namespace.
