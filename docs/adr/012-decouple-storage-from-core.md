# 12. Decouple Storage from Core

Date: 2024-05-20

## Status

Withdrawn

## Context

Circular dependencies were causing build failures when the core simulation logic attempted to persist state. The persistence layer was tightly coupled with the core simulation, leading to a tangled dependency graph where `Core` depended on `Storage` (for saving) and `Storage` depended on `Core` (for types).

**Update (Current):** This decision was not implemented as described. The `storage` module does not exist in the codebase, and the architectural separation was likely achieved through other means or reverted to simplify the build process. References to `Storage` have been removed from the main architecture documentation.

## Decision

Move persistence logic to a dedicated crate or module (`storage`). The `Core` module will define trait bounds for storage, and the `Storage` module will implement them.

## Consequences

### Positive
*   **Build Times**: Incremental compilation improves as `storage` changes don't force `core` rebuilds unless the trait changes.
*   **Dependency Graph**: Eliminates the circular dependency, making the architecture cleaner.

### Negative
*   **Complexity**: Increases FFI complexity or requires more careful trait design to ensure `Core` doesn't leak implementation details.
