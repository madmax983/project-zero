# 19. Decoupled Camera Interpolation

Date: 2024-06-05

## Status

Accepted

## Context

In a grid-based simulation, the simulation tick rate is often fixed and relatively low (e.g., 10-20 TPS) to manage CPU load for pathfinding and utility evaluation. However, the user interface (TUI) rendering loop can run much faster (60 FPS).

Directly tying the camera position (`Viewport`) to the simulation tick or input events resulted in "jittery" or "instant" camera movement. This feels unresponsive and harsh, especially when panning across large maps. We needed a way to provide smooth visual feedback without increasing the simulation tick rate.

## Decision

We decoupled the camera logic into two distinct phases: **Target Designation** and **Smooth Interpolation**.

1.  **CameraTarget (Resource):** Stores the desired destination (f32 coordinates). Input systems and "Focus on Entity" logic update this value instantly.
2.  **CameraCurrent (Resource):** Stores the current, interpolated position (f32 coordinates). This exists only for rendering smoothness.
3.  **Update Loop Separation:**
    *   **Input/Sim Systems:** Modify `CameraTarget`.
    *   **Render Loop (`update_camera_smooth`):** Runs every frame (independent of tick). It interpolates `CameraCurrent` towards `CameraTarget` using a lerp function (t=0.2), then updates the integer-based `Viewport` resource.

## Consequences

### Positive
*   **Fluid UX:** Camera movement feels smooth and responsive ("high fidelity" feel in a terminal).
*   **Performance:** Interpolation is cheap and runs only during render, while heavy simulation logic remains on the slower tick.
*   **Flexibility:** Allows for "Screen Shake" and other visual effects to be applied to `CameraCurrent` without affecting the logical `CameraTarget` or game state.

### Negative
*   **State Duplication:** We now have three resources tracking camera position (`CameraTarget`, `CameraCurrent`, `Viewport`), which can lead to desync bugs if not managed carefully.
*   **Input Lag Perception:** High interpolation values might make the camera feel "floaty" or slow to stop. (Currently tuned to 0.2).
