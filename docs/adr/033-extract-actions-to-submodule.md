# 33. Extract Actions to Submodule

Date: 2026-03-03

## Status

Accepted

## Context

The `src/layer1/actions.rs` module had grown into a bloated catch-all file containing over 10 unrelated `evaluate_*` functions. This violated the single responsibility principle and led to a disorganized codebase, making it difficult to maintain and understand individual action evaluation logic. The file size and complexity hindered readability and increased the risk of merge conflicts when multiple developers worked on different action types simultaneously.

## Decision

We created a new directory `src/layer1/actions/` and moved the contents of `actions.rs` into `mod.rs`. Then, we extracted each evaluator function into its own dedicated, properly scoped file (e.g., `haul.rs`, `mental_break.rs`, `clean.rs`, etc.) within the new submodule.

## Consequences

### Positive
*   **Maintainability**: Reduced file bloat and improved module organization by giving each piece of logic its own isolated file context.
*   **Separation of Concerns**: Adheres better to the single responsibility principle.
*   **Reduced Conflicts**: Smaller, focused files reduce the likelihood of merge conflicts.

### Negative
*   **Increased File Count**: The number of files in the project has increased, requiring slightly more navigation.
*   **Visibility Management**: Requires careful management of `pub(crate)` visibility across multiple files within the submodule.
