# 28. Seismic System Split

Date: 2024-05-25

## Status

Accepted

## Context

The simulation includes mechanics for geological instability (Spec 153) and machine-induced vibrations (Spec 171). Initially, a single `SeismicGrid` was proposed to track "ground disturbance."

However, we encountered a conflict in time scales:
1.  **Geological Instability (Earthquakes)** requires long-term stress accumulation that builds up over days of game time and decays very slowly.
2.  **Seismic Resonance (Vibration)** requires immediate feedback where flora or UI elements react to currently running machines. This needs to decay instantly when a machine stops.

Attempting to use a single grid resulted in either:
*   Machines causing earthquakes almost instantly (if accumulation was fast enough for vibration feedback).
*   Vibrations persisting for days after machines were turned off (if decay was slow enough for stress tracking).

## Decision

We have decided to **split the seismic simulation** into two distinct systems with separate data structures:

1.  **Seismic Grid (`SeismicGrid`)**:
    *   **Purpose**: Tracks long-term tectonic stress.
    *   **Behavior**: Accumulates slowly from mining/heavy industry. Decays very slowly (daily).
    *   **Outcome**: Triggers `GeologicalEvent::Earthquake` when thresholds are exceeded.
    *   **Location**: `src/layer1/geology.rs`.

2.  **Vibration Grid (`VibrationGrid`)**:
    *   **Purpose**: Tracks immediate ground shaking.
    *   **Behavior**: Calculated per-tick based on active `SeismicSource` components. Resets or decays rapidly (per tick).
    *   **Outcome**: Drives `SeismicResonance` mechanics (e.g., Flora agitation) and visual effects (`ScreenShake`).
    *   **Location**: `src/layer1/seismic.rs`.

## Consequences

### Positive
*   **Tuning Independence**: We can tune earthquake frequency without affecting the "feel" of machine vibrations.
*   **Responsiveness**: The UI and gameplay elements (like reacting flora) feel responsive to player actions (turning machines on/off) without side effects on the disaster system.

### Negative
*   **Memory Overhead**: Requires storing two `f32` grids for the map, doubling the memory footprint for this subsystem.
*   **Complexity**: Systems interacting with "the ground" must now decide which grid they care about (Stress vs. Vibration).
