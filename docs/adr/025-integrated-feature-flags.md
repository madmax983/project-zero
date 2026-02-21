# 25. Integrated Feature Flags

Date: 2024-06-03

## Status

Accepted

## Supersedes

[ADR 015: Experimental Feature Flags](./015-experimental-feature-flags.md)

## Context

ADR 015 established a pattern of **Directory Isolation**, forcing all experimental code into a `src/experimental/` directory. While this kept the main source tree clean, it introduced significant friction:
1.  **Refactoring Hell**: Promoting a feature from `experimental` to `layer1` required moving files, rewriting imports across the entire codebase, and often resolving complex merge conflicts.
2.  **Code Duplication**: Experimental features often needed to modify core systems (like `Pop` or `Building`), leading to duplicated or hacked-in hooks.
3.  **Visibility**: Features in `experimental/` were often forgotten or bit-rotted because they weren't compiled by default during development.

We needed a way to develop complex systems (like Nova Features: Constellations, Oral Tradition) alongside the core code without destabilizing the main branch, but with an easier path to integration.

## Decision

We have shifted to an **Integrated Feature Flag** strategy.

1.  **Co-location**: Experimental features live directly in their intended domain directory (e.g., `src/layer1/constellations.rs`).
2.  **Module-Level Guards**: The entire module declaration in `mod.rs` is guarded by the feature flag.
    ```rust
    #[cfg(feature = "nova")]
    pub mod constellations;
    ```
3.  **Inline Guards**: Specific hooks within core systems are guarded inline.
    ```rust
    #[cfg(feature = "nova")]
    if let Some(observer) = world.get_resource::<Observer>() { ... }
    ```

## Consequences

### Positive
*   **Zero-Cost Promotion**: Moving a feature to "Stable" simply requires removing the `#[cfg(feature = "x")]` line. No file moves, no import rewrites.
*   **Contextual Cohesion**: Related code lives together. `constellations.rs` is next to `astronomy.rs` (if it existed), making the system easier to understand.
*   **Compiler Support**: IDEs and tools can better analyze the code structure even if the feature is disabled (depending on configuration).

### Negative
*   **Polluted Core**: The `layer1` directory now contains files that might not be compiled in the default build.
*   **Conditional Compilation Complexity**: Developers must be careful to guard all usages of the feature, or the build will break when the feature is disabled.
