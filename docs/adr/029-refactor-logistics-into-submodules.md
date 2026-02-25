# 29. Refactor Logistics into Submodules

Date: 2024-05-27

## Status

Accepted

## Context

The `logistics` module originally handled only conveyor belt mechanics. With the introduction of Pneumatic Tubes (Spec 228), the complexity of the file grew significantly. Conveyor belts operate on a continuous flow model (moving items between adjacent tiles), while Pneumatic Tubes use a packet-based system (moving `TubeCarrier` entities along a path).

Keeping both systems in a single `logistics.rs` file violated the Single Responsibility Principle, making the code harder to read, maintain, and extend. It also increased the risk of merge conflicts when multiple developers worked on different transport systems simultaneously.

## Decision

We will refactor the `logistics` module from a single file into a directory structure: `src/layer1/logistics/`.

The new structure will be:
- `mod.rs`: The entry point for the module. It will re-export public types and functions to maintain backward compatibility with existing code that imports from `crate::layer1::logistics`.
- `conveyor.rs`: Contains all logic related to conveyor belts and continuous item movement.
- `pneumatic.rs`: Contains all logic related to pneumatic tubes, carriers, and terminals.

Future logistics systems (e.g., Drones, Rail Networks) will follow this pattern and be added as separate files within the `logistics` directory.

## Consequences

### Positive
*   **Modularity**: Each transport system is isolated in its own file, making the code easier to understand and reason about.
*   **Extensibility**: Adding new transport methods is straightforward and does not require modifying existing, stable code (Open/Closed Principle).
*   **Maintainability**: Bug fixes and features for one system are less likely to inadvertently affect others.
*   **Testing**: Unit tests can be more focused on specific transport mechanics.

### Negative
*   **File Count**: Increases the number of files in the project.
*   **Visibility Management**: Requires careful use of `pub(crate)` and `pub use` in `mod.rs` to ensure the API remains clean and accessible.
