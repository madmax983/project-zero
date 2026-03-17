# Specification 491: Orbital Tether Snaps

## 1. Overview
This feature introduces catastrophic failure for Layer 2 Space Elevators (Tethers). Tethers provide cheap transport of goods and Pops between Layer 1 and Layer 2. However, if a Tether is destroyed (via war, debris, or neglect), it snaps and collapses. The resulting impact acts like a continent-sized whip, destroying tiles and buildings in a massive line across the Layer 1 map and causing severe global seismic events, adding significant risk to the immense logistical benefits.

## 2. Dependencies
- `152` Orbital Stations / `042` Layer 2 Tethers (The structure itself)
- `389` Geological Instability / Seismic Events (For planetary consequences)
- `112` Maintenance Debt / Infrastructure Health (To trigger collapse from neglect)

## 3. RED Phase: Tests First

```rust
// tests/orbital_tether_snaps_tests.rs

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use scale::layer1::map::{GridPosition, TileType};
    use scale::layer1::building::{Building, Health};
    use scale::layer2::tether::{Tether, TetherHealth, TetherDestroyedEvent};

    fn setup_world() -> World {
        let mut world = World::new();
        world.init_resource::<Events<TetherDestroyedEvent>>();
        // Minimal setup for map and systems
        world
    }

    #[test]
    fn test_tether_snap_destroys_buildings_in_line() {
        let mut world = setup_world();

        let anchor_pos = GridPosition { x: 50, y: 50 };
        world.spawn((Tether, TetherHealth { current: 0.0, max: 1000.0 }, anchor_pos));

        // Spawn a building directly in the snap path (e.g., along the X axis)
        let building_entity = world.spawn((
            Building,
            Health { current: 500.0, max: 500.0 },
            GridPosition { x: 60, y: 50 },
        )).id();

        // Trigger the snap event
        world.send_event(TetherDestroyedEvent { anchor_position: anchor_pos, snap_direction_x: 1, snap_direction_y: 0, length: 50 });

        let mut schedule = Schedule::default();
        schedule.add_systems(process_tether_snap_impact_system);
        schedule.run(&mut world);

        // Building should be obliterated
        let health = world.get::<Health>(building_entity).unwrap();
        assert_eq!(health.current, 0.0);
    }

    #[test]
    fn test_tether_snap_ignores_buildings_outside_line() {
        let mut world = setup_world();

        let anchor_pos = GridPosition { x: 50, y: 50 };

        // Spawn a building outside the snap path
        let safe_building = world.spawn((
            Building,
            Health { current: 500.0, max: 500.0 },
            GridPosition { x: 50, y: 80 },
        )).id();

        world.send_event(TetherDestroyedEvent { anchor_position: anchor_pos, snap_direction_x: 1, snap_direction_y: 0, length: 50 });

        let mut schedule = Schedule::default();
        schedule.add_systems(process_tether_snap_impact_system);
        schedule.run(&mut world);

        let health = world.get::<Health>(safe_building).unwrap();
        assert_eq!(health.current, 500.0); // Unharmed
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/tech/orbital_tether_snaps.rs

use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::building::Health;

#[derive(Event, Debug, Clone)]
pub struct TetherDestroyedEvent {
    pub anchor_position: GridPosition,
    pub snap_direction_x: i32,
    pub snap_direction_y: i32,
    pub length: u32,
}

pub fn process_tether_snap_impact_system(
    mut events: EventReader<TetherDestroyedEvent>,
    mut buildings: Query<(&mut Health, &GridPosition)>,
) {
    for event in events.read() {
        // Calculate the line of impact based on direction and length
        let mut impact_zone = Vec::new();
        for i in 1..=event.length {
            impact_zone.push(GridPosition {
                x: event.anchor_position.x + (event.snap_direction_x * i as i32),
                y: event.anchor_position.y + (event.snap_direction_y * i as i32),
            });
        }

        // Damage buildings in the zone
        for (mut health, pos) in buildings.iter_mut() {
            if impact_zone.contains(pos) {
                health.current = 0.0; // Instant destruction for MVP
            }
        }
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Seismic Shockwaves**: In addition to the direct line of destruction, trigger a planetary seismic event that lightly damages buildings globally or reduces morale.
- **Visual & Audio**: The snap should feature massive screen shake, loud audio, and persistent dust/debris visual effects along the impact line.
- **Debris Tiles**: Instead of just destroying buildings, change the underlying `TileType` to an impassable `TetherDebris` type that must be cleared by Pops over a long time.
- **Variable Damage**: Buildings made of high-tier materials (e.g., Plasteel) might survive with 1 HP rather than instant destruction.

## 6. Acceptance Criteria
- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures for new code.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `src/layer1/tech/orbital_tether_snaps.rs`.
- [ ] Buildings along the tether's collapse line are severely damaged or destroyed.
- [ ] Event handles direction and length dynamically.

## 7. Technical Guidance
- Calculating the exact line of impact can be tricky on a grid; for the MVP, pure horizontal/vertical/diagonal snaps are sufficient.
- Ensure `TetherDestroyedEvent` is properly initialized in `setup_world()` and scheduled for cleanup to avoid memory leaks.
- When generating the snap direction, it should ideally be somewhat random but perhaps influenced by the planetary rotation or the location of the destroying force.

## 8. Questions
*Builder: add questions here if spec is unclear. Architect will address.*
