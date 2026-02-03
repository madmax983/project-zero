# 3. YAGNI - Excision of Layers 2 and 3

Date: 2024-10-25
Status: Accepted

## Context

The initial architectural vision (ADR 001) established a three-layer structure:
1. Colony Scale (Layer 1)
2. System Scale (Layer 2)
3. Interstellar Scale (Layer 3)

However, development is currently focused exclusively on the Colony Simulation (Layer 1). The directories `src/layer2` and `src/layer3` exist as empty placeholders containing only an empty `mod.rs`.

The "Razor Protocol" in our development guidelines mandates strict adherence to YAGNI (You Aren't Gonna Need It), explicitly requiring the deletion of empty architectural placeholders to avoid confusion, cognitive overhead, and implied scope that is not yet implemented.

## Decision

We will remove the placeholder directories `src/layer2` and `src/layer3` and their references in the codebase.

The architecture will focus explicitly on `layer1` (Colony Simulation) and `shared` (Utilities).

The concepts of System and Interstellar simulation are deferred until they are actively specified and scheduled for implementation. When that time comes, new ADRs will be written to re-introduce them, potentially with better insights gained from the maturation of Layer 1.

## Consequences

**Positive:**
- **Reduced Cognitive Load:** New developers (and agents) will not be confused by empty folders or wonder where the "System Sim" logic lives.
- **Architectural Honesty:** The codebase structure reflects the actual state of the application.
- **Compliance:** Adheres to the Razor Protocol and YAGNI.

**Negative:**
- **Future Rework:** When System Scale features are prioritized, we will need to re-create the module structure and re-export them. This is a trivial cost compared to the benefit of clarity now.
