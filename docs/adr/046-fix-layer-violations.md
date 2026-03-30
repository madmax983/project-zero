# ADR 046: Fix Layer Violations

## Status
Proposed

## Context
Several Layer 1 systems were directly accessing and mutating Layer 2 types (`OrbitalBody`, `PsychicBackground`, `OrbitalShield`) and Layer 3 types (`DetectionRisk`). This created circular dependencies and broke the acyclic architectural directive, tangling the core mechanics.

## Decision
We implemented Event bridges and direct Integration Systems to enforce strict layer boundaries.
- `layer1::politics` now broadcasts `ElectionFinishedEvent`, which is captured by `layer2::integration`.
- `apply_psychic_radiation_system` was moved to `layer2::integration`.
- Tracking of `MartyrsEngine` duration was separated in Layer 1 from `OrbitalShield` effects in `layer2::integration`.
- `parasitic_broadcast_risk_system` was moved entirely out of Layer 1 into a new `layer3::integration` module.

## Consequences
- **Strict Boundaries:** Layer 1 no longer directly mutates Layer 2 or 3 states, preserving the acyclic architecture.
- **Event-Driven Integration:** Communication between layers is now safely handled via Bevy Events and dedicated `integration` modules.
- **Reduced Coupling:** Systems are more isolated and easier to test.
