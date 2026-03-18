# 41. Encapsulate Tech Submodules

Date: 2026-03-15

## Status

Accepted

## Context

Internal mechanisms of the `tech` module (such as `ghost_code`, `machine_awakening`, and `infinite_archive`) were exposing internal logic and structures publicly. This led to leaky abstractions where implementation details were not properly hidden from the rest of the application.

## Decision

We have decided to encapsulate the sub-modules across `src/layer1/tech/*` by restricting their visibility to `pub(crate)`. Internal structs, enums, and functions related to specific tech features are now kept internal to the module, ensuring structural safety and enforcing strict boundaries.

## Consequences

### Positive
*   **Structural Safety**: Prevents "Leaky Abstractions" by hiding internal implementation details.
*   **Stability**: Reduces coupling between the `tech` module and other parts of the system, leading to potentially faster compile times and fewer ripple effects when modifying internal logic.

### Negative
*   **Refactoring Overhead**: Any external code that previously relied on internal `tech` structures must now use proper public APIs, which may require updating existing tests or cross-module logic.