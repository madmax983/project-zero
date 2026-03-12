# 36. Split Overloaded System Tuples

Date: 2026-03-12

## Status

Accepted

## Context

Bevy's `add_systems()` macro has a maximum tuple size limit of 21 elements. As `src/layer1/systems/observation.rs` grew, it exceeded this limit (reaching 22 items), which caused a compilation failure: "Tuple Limit Tangle".

## Decision

Instead of trying to force all systems into a single tuple, split the overloaded tuple in `add_systems()` into two separate tuples, safely maintaining the `.in_set(Layer1SystemSet::Observation)` schedule association for all included systems by repeating `.in_set()` or chaining `.add_systems()`.

## Consequences

### Positive
*   **Compilation**: Fixes the build break caused by exceeding the macro tuple limit.
*   **Scalability**: Allows adding an unbounded number of systems per phase without having to refactor the entire system registration architecture again.

### Negative
*   **Verbose Syntax**: Requires slightly more boilerplate and repeated `.in_set()` calls or separate `add_systems()` invocations.
