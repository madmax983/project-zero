# 32. Rename `check_access` to `check_security_clearance`

Date: 2026-03-02

## Status

Proposed

## Context

A recent architectural change refactored security-related logic, creating an ambiguous glob re-export conflict. Specifically, the function `check_access` was defined in both `src/layer1/access_control.rs` and `src/layer1/security/mod.rs`. When these modules were re-exported in `src/layer1/mod.rs`, it caused a namespace collision and compiler warnings. Additionally, having identical function names for distinct concepts—physical door access versus biometric terminal clearance—caused semantic confusion.

## Decision

We renamed `check_access` to `check_security_clearance` within the `layer1/security` module. This clearly distinguishes it from the `check_access` function in `layer1/access_control`. We also fixed unused assignments and imports in related modules (`layer1/tech/mod.rs` and `layer1/the_visitor.rs`) and cleaned up namespace exports.

## Consequences

### Positive
*   **Stability**: Removed compiler warnings and resolved the ambiguous glob re-export conflict.
*   **Clarity**: The codebase now clearly differentiates between physical access checks (`AccessControl`) and biometric/security clearance checks (`SecurityTerminal` and `BiometricProfile`).

### Negative
*   **None**: This is a pure organizational and naming improvement with no negative side effects on runtime performance or architecture.
