# 249: Holographic Facades

## Overview

"The view is beautiful, until the batteries run dry."

Holographic Facades allow players to cover ugly, efficient industrial environments with illusions of luxury (Forests, Marble, Windows). A `HoloProjector` building emits a high `Beauty` value while powered. However, this beauty is fragile. If power is lost, the projection fails immediately, and any Pops witnessing the failure suffer a massive "Disillusionment" mood penalty—worse than if they had just lived in the ugly room to begin with.

This mechanic creates a tension between **Real Luxury** (Static, Expensive, Permanent) and **Fake Luxury** (Dynamic, Cheap, Fragile).

## Dependencies

- `044` — Horticulture & Beauty (BeautyGrid)
- `042` — Energy System (Power consumption)
- `031` — Pop Morale (Mood impact)

## RED Phase: Tests First

Write these tests in `src/layer1/hologram_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::beauty::BeautyGrid;
    use crate::layer1::power::{PowerConsumer, PowerGrid};
    use crate::layer1::hologram::{HoloProjector, update_holograms_system};
    use crate::layer1::pop::{Pop, Mood};
    use crate::layer1::map::GridPosition;

    #[test]
    fn test_hologram_adds_beauty_when_powered() {
        let mut world = World::new();
        world.insert_resource(BeautyGrid::new(10, 10));

        // Spawn powered HoloProjector
        world.spawn((
            HoloProjector {
                active_beauty: 50.0,
                radius: 5.0,
                is_active: true,
            },
            PowerConsumer {
                demand: 10.0,
                received: 10.0, // Fully powered
                ..Default::default()
            },
            GridPosition { x: 5, y: 5 },
        ));

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(update_holograms_system);
        schedule.run(&mut world);

        let beauty = world.resource::<BeautyGrid>();
        assert!(beauty.get(5, 5) >= 50.0);
    }

    #[test]
    fn test_hologram_removes_beauty_when_unpowered() {
        let mut world = World::new();
        let mut beauty_grid = BeautyGrid::new(10, 10);
        // Pre-fill with beauty to verify removal
        beauty_grid.set(5, 5, 50.0);
        world.insert_resource(beauty_grid);

        // Spawn unpowered HoloProjector
        world.spawn((
            HoloProjector {
                active_beauty: 50.0,
                radius: 5.0,
                is_active: true, // Was active
            },
            PowerConsumer {
                demand: 10.0,
                received: 0.0, // Power cut
                ..Default::default()
            },
            GridPosition { x: 5, y: 5 },
        ));

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(update_holograms_system);
        schedule.run(&mut world);

        let beauty = world.resource::<BeautyGrid>();
        // Should drop to ambient (0.0 assumed for test)
        assert!(beauty.get(5, 5) < 1.0);
    }

    #[test]
    fn test_disillusionment_shock() {
        let mut world = World::new();
        world.insert_resource(BeautyGrid::new(10, 10));
        world.insert_resource(Events::<crate::layer1::hologram::HologramFailureEvent>::default());

        // Spawn HoloProjector losing power
        world.spawn((
            HoloProjector { active_beauty: 50.0, radius: 5.0, is_active: true },
            PowerConsumer { demand: 10.0, received: 0.0, ..Default::default() },
            GridPosition { x: 5, y: 5 },
        ));

        // Spawn Pop nearby
        let pop_id = world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            Mood { value: 80.0, ..Default::default() },
        )).id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(update_holograms_system);
        // Add shock system
        schedule.add_systems(crate::layer1::hologram::apply_disillusionment_system);
        schedule.run(&mut world);

        let mood = world.get::<Mood>(pop_id).unwrap();
        // Should drop significantly (e.g. -20)
        assert!(mood.value < 80.0);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Components

```rust
// src/layer1/hologram.rs

use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::power::PowerConsumer;
use crate::layer1::beauty::BeautyGrid;

#[derive(Component)]
pub struct HoloProjector {
    pub active_beauty: f32,
    pub radius: f32,
    pub is_active: bool,
}

#[derive(Event)]
pub struct HologramFailureEvent {
    pub position: GridPosition,
    pub radius: f32,
}

pub fn update_holograms_system(
    mut query: Query<(&mut HoloProjector, &PowerConsumer, &GridPosition)>,
    mut beauty_grid: ResMut<BeautyGrid>,
    mut events: EventWriter<HologramFailureEvent>,
) {
    for (mut holo, power, pos) in query.iter_mut() {
        let powered = power.received >= power.demand;

        if powered && !holo.is_active {
            // Turning ON
            holo.is_active = true;
            // Add beauty (simplified: just center tile for GREEN)
            beauty_grid.add_modifier(pos.x, pos.y, holo.active_beauty);
        } else if !powered && holo.is_active {
            // Turning OFF (Failure)
            holo.is_active = false;
            // Remove beauty
            beauty_grid.add_modifier(pos.x, pos.y, -holo.active_beauty);

            // Trigger shock
            events.send(HologramFailureEvent {
                position: *pos,
                radius: holo.radius,
            });
        }
    }
}

pub fn apply_disillusionment_system(
    mut events: EventReader<HologramFailureEvent>,
    mut pops: Query<(&GridPosition, &mut crate::layer1::pop::Mood)>,
) {
    for event in events.read() {
        for (pop_pos, mut mood) in pops.iter_mut() {
            // Simple distance check
            if pop_pos.distance_chebyshev(&event.position) <= event.radius as u32 {
                mood.value -= 20.0; // Major penalty
                // Optional: Add "Disillusioned" thought/memory
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Beauty Radius**: The `HoloProjector` should apply beauty over a radius, not just a single tile. This requires iterating grid points in `update_holograms_system` (expensive if naive). Use `BeautyGrid`'s existing propagation if available, or a `BeautySource` component handled by the beauty system.
- **Grace Period**: To prevent flickering (power oscillating 99%<->100%), add a small buffer or delay before failure triggers.
- **Visuals**: Trigger a "glitch" particle effect when failing.

## Acceptance Criteria

- [ ] `HoloProjector` component defined.
- [ ] Active holograms add Beauty to the grid.
- [ ] Power loss removes Beauty immediately.
- [ ] `HologramFailureEvent` is emitted on power loss.
- [ ] Pops near failure suffer Mood penalty.
- [ ] Tests pass.

## Technical Guidance

- Integrate with `Spec 044` (Horticulture). If `BeautyGrid` recalculates every frame (as per memory), `HoloProjector` logic simplifies to "If powered, apply modifier". The "Failure" detection just needs to track `was_active` state change.
- Use `PowerConsumer` from `Spec 042`.

## Questions

*Builder: add questions here if spec is unclear.*
*Architect:* See related specifications for design details. MVP implementation should follow standard conventions.
