# 15. Experimental Feature Flags

Date: 2024-05-22

## Status

Accepted

## Context

As the simulation grows, we need to prototype complex systems (like Acoustics, Advanced Weather, or Layer 2 mechanics) without destabilizing the core gameplay loop or bloating the main build with unfinished code.

Early attempts to integrate these features directly into `layer1` led to regression risks and harder debugging of core systems.

## Decision

We will use a combination of directory isolation and Rust's `feature` flags to manage experimental code:

1.  **Directory Isolation**: Experimental systems reside in `src/experimental/`.
2.  **Feature Flags**: These modules are guarded by `#[cfg(feature = "feature_name")]`.
3.  **Cargo Features**: The features are defined in `Cargo.toml` (e.g., `nova` for acoustics).

Example:
```rust
#[cfg(feature = "nova")]
pub mod acoustics;
```

## Consequences

### Positive
*   **Stability**: The default build remains stable and clean.
*   **Focus**: Developers can work on "vertical slices" of new mechanics without worrying about breaking the main game loop immediately.
*   **Performance**: Experimental systems (which might be unoptimized) do not run in the default release build.

### Negative
*   **Complexity**: Requires careful management of `cfg` attributes.
*   **Bitrot**: Experimental features might break silently if not tested regularly with the feature flag enabled.
*   **Integration Friction**: Moving a feature from `experimental` to `layer1` requires moving files and removing guards.
