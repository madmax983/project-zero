# ADR 021: Backlog Reconciliation and Ghost Spec Recovery

## Context

During a routine audit of the `specs/` directory, the Architect identified a significant discrepancy between the file system and the project tracking documents (`design/BACKLOG.md` and `design/COMPLETED.md`).

Over 20 specification files existed in `specs/` but were listed in neither the backlog nor the completed list. These "Ghost Specs" represented a mix of:
1.  **Implemented Features**: Features that were built and merged but never marked as completed (e.g., `010 Chronicle`, `098 Medical Triage`).
2.  **Unimplemented Features**: Features that were designed but lost in the backlog (e.g., `120 Crop Diversity`, `067 Militia`).

This state of inconsistency creates confusion for Builders and Architects, potentially leading to duplicate work or lost features.

## Decision

The Architect has performed a comprehensive reconciliation of the project state:

1.  **Recovered Implemented Specs**: All specs corresponding to existing code files were identified and moved to `design/COMPLETED.md`.
2.  **Recovered Unimplemented Specs**: All valid specs without implementation were moved to `design/BACKLOG.md`.
3.  **Prioritized Critical Recovery**: Spec `120 Crop Diversity` was identified as a critical "Ghost Spec" and prioritized in the backlog.
4.  **Added New Feature**: Spec `121 Hydroponics` was designed to complement the recovered features.

## Consequences

-   **Positive**: The project backlog now accurately reflects the state of the codebase. "Lost" features are now visible and actionable.
-   **Positive**: `COMPLETED.md` is now a reliable source of truth for existing features.
-   **Negative**: The `COMPLETED.md` file now contains a large block of items with the same completion date (2026-03-05), losing some historical granularity on *when* exactly those features were finished. This is an acceptable trade-off for accuracy.

## Status

Accepted
