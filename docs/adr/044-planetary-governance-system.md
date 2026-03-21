# ADR 044: Planetary Governance System

**Status:** Proposed
**Date:** 2026-03-21

## Context

The Layer 2 (System Simulation) previously lacked a mechanism to represent the political and economic leadership of planetary bodies. There was no direct way to leverage the individual traits of colony characters (Pops) at a macro scale, nor was there a systemic risk associated with managing highly prosperous worlds. This limited the narrative and emergent gameplay potential between the local colony and the broader system.

## Decision

We have implemented a Planetary Governance system within `src/layer2/governance.rs` (Spec 544).

- **Components:** Introduced `Governor` and `GovernorStats` (tracking ambition and corruption) to link a Layer 1 Pop to a Layer 2 Planet. Added `PlanetProduction` and `ProsperityRating` to quantify planetary economic health.
- **Systems:**
  - `apply_governor_effects_system`: Modifies planetary production throughput based on the governor's traits (e.g., `Trait::LogisticsExpert`).
  - `update_governor_ambition_system`: Passively increases a governor's ambition based on the planet's prosperity rating.
  - `check_governor_rebellion_system`: Triggers a `RebellionEvent` when a governor's ambition exceeds a critical threshold, resetting it afterward to prevent event spam.

## Consequences

### Positive
- **Cross-Layer Integration:** Meaningfully connects Layer 1 character traits to Layer 2 economic outputs.
- **Emergent Narrative:** Introduces long-term political risk (Rebellions) that players must manage, adding depth to the late-game experience.

### Negative
- **Increased State Management:** Adds new components that must be tracked and synchronized across ticks, increasing memory and processing overhead.
- **UI Requirements:** Necessitates new interfaces to assign governors and monitor their ambition and planetary prosperity.
