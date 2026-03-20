# 43. Layer 3 Revival (Interstellar Scale)

Date: 2026-03-20

## Status

Proposed

## Context

ADR 003 ("YAGNI - Excision of Layers 2 and 3") initially removed the placeholder system-scale (Layer 2) and interstellar-scale (Layer 3) simulations to focus entirely on the colony scale (Layer 1). ADR 031 ("Layer 2 Revival") reintroduced Layer 2.

Recently, development has organically reintroduced elements of Layer 3 to handle events that originate outside the local solar system but heavily impact the colony. Specifically, the `src/layer3` module now contains systems for Interstellar events such as `events/debt_prison.rs` (Bailout Offers from off-world cartels) and `silence.rs` (Detection Risk tracking and Hostile Spawn events).

These mechanics represent forces beyond the colony's direct control and do not fit well into the abstract gridless simulation of Layer 2 (System View). They require a distinct boundary.

## Decision

We officially revive **Layer 3 (Interstellar Scale)**.

1.  **Scope:** Layer 3 is responsible for tracking abstract, long-term, extra-system variables (e.g., `DetectionRisk`, `ColonyDebt` to off-world entities) and generating discrete events (`HostileSpawnEvent`, `BailoutOfferEvent`) that cascade down into Layers 1 and 2.
2.  **Implementation:** Layer 3 will not be a continuous physics or spatial simulation. It will act as a macro-manager and event generator.
3.  **Boundary:** Layer 3 systems evaluate the aggregate state of Layer 1 (e.g., total power output, total population) and Layer 2 to calculate interstellar risks and rewards.

## Consequences

### Positive
*   **Architectural Accuracy:** The module structure (`src/layer3`) once again reflects the documented architecture, resolving the discrepancy created by the recent organic additions.
*   **Domain Clarity:** Provides a clear home for "Game Master" style events (like sudden hostile invasions or economic bailouts) without cluttering the more deterministic Layer 1 or Layer 2.

### Negative
*   **Complexity:** Managing the interfaces and event flows from Layer 3 down to Layer 1 increases the overall surface area of the architecture.
