# 15. Centralized Integration Bridges

Date: 2024-05-23

## Status

Accepted

## Context

As the simulation grew in complexity, we began encountering situations where core domains needed to interact with each other in ways that created circular dependencies. For example:

- **Factions** need to affect **Pop Needs** (Unhappy factions reduce leisure).
- **Resources** (Waste) need to affect **Atmosphere** (Pollution).
- **Weather** (Fire) needs to affect **Health** (Burn damage).
- **Chronicle** (History) needs to create **Rumors** (Social).

If we allowed direct imports (e.g., `pop.rs` importing `factions.rs`), we would quickly create a tangled web of dependencies where every module knows about every other module. This makes the system rigid, hard to test in isolation, and prone to compilation errors due to circular references.

## Decision

We decided to adopt the **Integration Bridge Pattern**.

1.  **Centralized Logic:** All cross-domain logic is centralized in a dedicated module: `src/layer1/integration.rs`.
2.  **Bridge Systems:** This module contains "Bridge Systems" that query component states from Source Domains and apply changes to Target Domains.
3.  **One-Way Flow:** Core modules (like `pop.rs` or `resources.rs`) do not import `integration.rs`. Only `integration.rs` imports the core modules.
4.  **Event-Driven:** Where possible, bridges use Bevy Events (e.g., `PopDied`) to trigger cross-domain logic without direct coupling.

## Consequences

### Positive

-   **Decoupled Core:** Core domains remain loosely coupled. `pop.rs` can be tested without mocking `factions.rs`.
-   **No Circular Dependencies:** The dependency graph flows strictly from Core -> Integration -> Simulation Loop.
-   **Centralized Gameplay Rules:** All "emergent interactions" (how systems play with each other) are visible in one place, making it easier to tune game balance.

### Negative

-   **"Spooky Action at a Distance":** Logic affecting an entity is now split. A developer looking at `health.rs` might not realize that `integration.rs` also modifies health (via fire damage).
-   **Discipline Required:** Developers must resist the temptation to add "just one small import" between core modules and instead route it through a bridge system.
