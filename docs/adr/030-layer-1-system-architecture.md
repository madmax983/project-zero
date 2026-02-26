# 30. Layer 1 System Architecture Refactor

Date: 2024-05-22
Status: Accepted

## Context

The `src/layer1/systems.rs` file was becoming a monolithic "God File," containing registration logic for dozens of systems. As the simulation grew, we encountered two main issues:
1.  **Code Organization**: Navigation was difficult due to the sheer volume of imports and system definitions in a single file.
2.  **Bevy Tuple Limit**: Bevy's `add_systems` API accepts a tuple of systems. This tuple has a maximum size (historically 12, now often higher but still limited, around 21 in some configurations). We were hitting this limit, preventing new systems from being added without awkward workarounds.

## Decision

We have refactored the system registration logic into a dedicated directory structure `src/layer1/systems/`, splitting systems into semantic submodules based on their execution phase:

*   `cleanup`: Event cleanup and input handling.
*   `execution`: Movement, work, and direct interactions.
*   `economy`: Production, resources, and passive ticking.
*   `environment`: Fire, weather, and decay.
*   `consumption`: Needs decay, spoilage, and death.
*   `observation`: History, social, and dreams.

We introduced `Layer1SystemSet` to enforce execution order across these modules:

```rust
pub enum Layer1SystemSet {
    EventCleanup,
    Execution,   // .after(EventCleanup)
    Economy,     // .after(Execution)
    Environment, // .after(Economy)
    Consumption, // .after(Economy)
    Observation, // .after(Consumption)
}
```

Each submodule (`cleanup.rs`, `execution.rs`, etc.) exposes a `register(schedule: &mut Schedule)` function that adds its specific systems to the appropriate `SystemSet`.

## Consequences

### Positive
*   **Modularity**: Systems are grouped by domain, making it easier to find related logic.
*   **Scalability**: We are no longer bound by a single tuple limit for all Layer 1 systems. Each submodule manages its own registration, and even within submodules, systems can be added in multiple `add_systems` calls if needed.
*   **Clarity**: The `Layer1SystemSet` enum provides a clear, high-level view of the frame's execution flow.

### Negative
*   **Boilerplate**: Adding a new system requires identifying the correct module and ensuring it is registered in the corresponding `register` function.
*   **Indirection**: The registration logic is now spread across multiple files, requiring developers to traverse the module tree to see the full system list.
