# 43. Encapsulate Social Mechanics

Date: 2026-03-19

## Status

Accepted

## Context

The `layer1` root directory had become cluttered with numerous social mechanics such as `debt`, `grievances`, `old_guard`, `empty_room`, `cultural_vandalism`, `pen_pals`, `cadet`, `placebo`, `sentient_standard`, and `zero_g_sports`. These mechanics were declared directly under `layer1`, causing domain logic to leak into the top-level namespace rather than being properly encapsulated in a dedicated social subsystem. While `society` (Secret Societies) had an earlier ADR (038) for this, the rest of the social features still remained at the top level.

## Decision

We have decided to encapsulate all social mechanics into a distinct `src/layer1/social/` module. The scattered files (`debt.rs`, `grievances.rs`, `old_guard.rs`, etc.) have been moved into this cohesive domain directory, and the parent `layer1::social` module now controls their visibility.

## Consequences

### Positive
*   **Domain Boundaries**: Enforces a stronger domain boundary by nesting all social-related simulation mechanics entirely within the `social` subsystem.
*   **Maintainability**: Significantly cleans up the `layer1` root namespace and groups related social mechanics together, improving code organization and readability.

### Negative
*   **Refactoring Overhead**: Existing imports pointing to the root `layer1` for these mechanics had to be updated across the codebase.
