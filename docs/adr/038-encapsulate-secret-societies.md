# 38. Encapsulate Secret Societies

Date: 2026-03-15

## Status

Accepted

## Context

The `SecretSocieties` logic (`src/layer1/society.rs`) was declared as a root-level module (`pub mod society`) directly under `layer1`. This caused social domain logic to leak into the top-level namespace rather than being encapsulated within its proper functional domain.

## Decision

Moved `src/layer1/society.rs` to `src/layer1/social/society.rs` and updated module declarations and imports accordingly.

## Consequences

### Positive
*   **Domain Boundaries**: Enforces a stronger domain boundary by nesting the secret society mechanics entirely within the `social` subsystem.
*   **Namespace Clarity**: Simplifies the top-level `layer1` namespace, reducing clutter.
*   **Maintainability**: Groups related social mechanics together, improving code organization and readability.

### Negative
*   **Refactoring Cost**: Required moving files and updating imports across the codebase.
