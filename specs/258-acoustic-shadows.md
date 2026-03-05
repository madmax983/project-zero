# 258: Acoustic Shadows

## Overview

"In space, no one can hear you scream."

Updates the **Acoustic Simulation** (Spec 060) so that **Vacuum** tiles block sound transmission completely. This allows players to build "Vacuum Gaps" (moats of empty space) around noisy machinery or bedrooms to create perfect soundproofing.

However, this creates a safety risk: alarms, shouts for help, and gunfire cannot be heard across a vacuum gap. If a fire starts in a vacuum-isolated room, no one outside will hear it.

## Dependencies

- `060` — Acoustic Simulation (Noise propagation)
- `119` — Airlock & Pressure (Vacuum definition)

## RED Phase: Tests First

Write these tests in `src/layer1/acoustic_shadow_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::acoustic::{NoiseMap, update_noise_system};
    use crate::layer1::atmosphere::{AtmosphereGrid, AtmosphereCell};
    use crate::layer1::map::GridPosition;

    #[test]
    fn test_vacuum_blocks_noise() {
        let mut world = World::new();
        let size = 10;
        world.insert_resource(NoiseMap::new(size, size));

        let mut atmos = AtmosphereGrid::new(size, size);
        // Set (5,5) as noise source (Air)
        atmos.set_pressure(5, 5, 100.0);
        // Set (6,5) as Vacuum (0 pressure)
        atmos.set_pressure(6, 5, 0.0);
        // Set (7,5) as Listener (Air)
        atmos.set_pressure(7, 5, 100.0);

        world.insert_resource(atmos);

        // Spawn noise source
        world.spawn((
            GridPosition { x: 5, y: 5 },
            crate::layer1::acoustic::NoiseSource { volume: 100.0 },
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(update_noise_system);
        schedule.run(&mut world);

        let noise = world.resource::<NoiseMap>();

        // Source should be loud
        assert!(noise.get(5, 5) > 90.0);
        // Vacuum should have 0 noise (physically impossible to carry sound)
        assert_eq!(noise.get(6, 5), 0.0);
        // Listener across the gap should hear NOTHING
        assert_eq!(noise.get(7, 5), 0.0);
    }

    #[test]
    fn test_sound_flanks_vacuum() {
        // Sound should travel around the vacuum gap if there is air
        let mut world = World::new();
        let size = 10;
        world.insert_resource(NoiseMap::new(size, size));

        let mut atmos = AtmosphereGrid::new(size, size);
        // 5,5 Source
        atmos.set_pressure(5, 5, 100.0);
        // 6,5 Vacuum wall
        atmos.set_pressure(6, 5, 0.0);
        // 7,5 Listener
        atmos.set_pressure(7, 5, 100.0);
        // 6,6 Air Bridge
        atmos.set_pressure(6, 6, 100.0);

        world.insert_resource(atmos);

        world.spawn((
            GridPosition { x: 5, y: 5 },
            crate::layer1::acoustic::NoiseSource { volume: 100.0 },
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(update_noise_system);
        schedule.run(&mut world);

        let noise = world.resource::<NoiseMap>();
        // Should hear some noise via (5,5)->(5,6)->(6,6)->(7,6)->(7,5) or similar path
        assert!(noise.get(7, 5) > 0.0);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update `update_noise_system`

Modify `src/layer1/acoustic.rs`. Ensure it reads `AtmosphereGrid`.

```rust
// src/layer1/acoustic.rs

use crate::layer1::atmosphere::AtmosphereGrid;

// Update the propagation logic (assuming Dijkstra or Flood Fill)
pub fn update_noise_system(
    mut noise_map: ResMut<NoiseMap>,
    sources: Query<(&GridPosition, &NoiseSource)>,
    atmos: Res<AtmosphereGrid>,
) {
    noise_map.clear();

    for (pos, source) in sources.iter() {
        // Simple BFS/FloodFill
        let mut queue = std::collections::VecDeque::new();
        queue.push_back((*pos, source.volume));

        // ... (visited set logic) ...

        while let Some((p, vol)) = queue.pop_front() {
            if vol <= 0.0 { continue; }

            // Check vacuum
            if atmos.get_pressure(p.x, p.y) < 1.0 {
                // Sound dies instantly in vacuum
                continue;
            }

            // Update map
            let current = noise_map.get(p.x, p.y);
            noise_map.set(p.x, p.y, current.max(vol));

            // Spread to neighbors
            // ... decay vol by distance ...
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Vibration**: Some sound travels through *solids* (floor) even in vacuum. Implement `StructureBorneNoise` vs `AirBorneNoise`. Vacuum blocks AirBorne, but floor connects StructureBorne.
- **Spacesuits**: Pops in suits can hear via radio (ignoring vacuum) but local environment sound is muffled.

## Acceptance Criteria

- [ ] Vacuum tiles (pressure ~0) act as perfect sound barriers.
- [ ] Sound can flank around vacuum gaps if air exists.
- [ ] Tests pass.

## Technical Guidance

- Ensure `AtmosphereGrid` is initialized before Acoustic system runs in tests.
- This is a breaking change for `060` if it didn't account for atmosphere.

## Questions

*Builder: Does glass block sound?*
*Architect:* Yes, any fully enclosed structural tile (including glass walls) blocks sound propagation for the simulation.
