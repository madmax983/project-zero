# 099: Fleet Movement

## Overview

Introduces mobile **Fleets** to Layer 2 (System View). Fleets are entities that can travel between orbital bodies (Stars, Planets, Moons).

This spec defines:
1.  **Fleet Component**: Identifies an entity as a fleet.
2.  **InOrbit Component**: State when a fleet is stationary at an orbital body.
3.  **InTransit Component**: State when a fleet is moving between bodies.
4.  **FleetOrder Component**: A command to move to a destination.
5.  **Movement System**: Updates fleet position during transit and handles arrival.

This feature enables the dynamic layer of the system map, serving as the foundation for trade, exploration, and combat.

## Dependencies

- `094` — System View Architecture (for `OrbitalBody`, `Orbit` components)
- `095` — System Generation (for test data context)

## RED Phase: Tests First

Write these tests in `src/layer2/fleet_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer2::system::{OrbitalBody, Orbit};
    use crate::layer2::fleet::{Fleet, InOrbit, InTransit, FleetOrder, fleet_movement_system, fleet_order_system};
    use ratatui::style::Color;

    fn setup_world() -> World {
        let mut world = World::new();
        // Register components if needed
        world
    }

    #[test]
    fn test_fleet_spawn_in_orbit() {
        let mut world = setup_world();
        let planet = world.spawn_empty().id();

        let fleet = world.spawn((
            Fleet,
            InOrbit { parent: planet },
            OrbitalBody {
                name: "Scout 1".to_string(),
                radius: 0.0, // Fleets are points usually, or very small
                color: Color::White,
                char: '▲',
            }
        )).id();

        let in_orbit = world.get::<InOrbit>(fleet).expect("Fleet should be in orbit");
        assert_eq!(in_orbit.parent, planet);
    }

    #[test]
    fn test_order_fleet_movement() {
        let mut world = setup_world();
        let planet_a = world.spawn_empty().id();
        let planet_b = world.spawn_empty().id();

        let fleet = world.spawn((
            Fleet,
            InOrbit { parent: planet_a },
        )).id();

        // Issue Move Order
        world.entity_mut(fleet).insert(FleetOrder::MoveTo(planet_b));

        // Run Order System
        let mut schedule = Schedule::default();
        schedule.add_systems(fleet_order_system);
        schedule.run(&mut world);

        // Verify Fleet is now InTransit
        assert!(world.get::<InOrbit>(fleet).is_none(), "Fleet should leave orbit");
        assert!(world.get::<FleetOrder>(fleet).is_none(), "Order should be consumed");

        let transit = world.get::<InTransit>(fleet).expect("Fleet should be in transit");
        assert_eq!(transit.origin, planet_a);
        assert_eq!(transit.destination, planet_b);
        assert_eq!(transit.progress, 0.0);
        assert!(transit.duration > 0.0);
    }

    #[test]
    fn test_fleet_transit_progress() {
        let mut world = setup_world();
        let planet_a = world.spawn_empty().id();
        let planet_b = world.spawn_empty().id();

        let fleet = world.spawn((
            Fleet,
            InTransit {
                origin: planet_a,
                destination: planet_b,
                progress: 0.5,
                duration: 10.0, // 10 ticks
            }
        )).id();

        // Run Movement System
        let mut schedule = Schedule::default();
        schedule.add_systems(fleet_movement_system);
        schedule.run(&mut world);

        let transit = world.get::<InTransit>(fleet).unwrap();
        // Progress should increase by 1.0 / duration
        // 0.5 + (1.0/10.0) = 0.6
        assert!((transit.progress - 0.6).abs() < f32::EPSILON);
    }

    #[test]
    fn test_fleet_arrival() {
        let mut world = setup_world();
        let planet_a = world.spawn_empty().id();
        let planet_b = world.spawn_empty().id();

        let fleet = world.spawn((
            Fleet,
            InTransit {
                origin: planet_a,
                destination: planet_b,
                progress: 0.95,
                duration: 10.0,
            }
        )).id();

        // Run Movement System (should complete transit)
        let mut schedule = Schedule::default();
        schedule.add_systems(fleet_movement_system);
        schedule.run(&mut world);

        // Verify Fleet is InOrbit at destination
        assert!(world.get::<InTransit>(fleet).is_none(), "Fleet should arrive");
        let in_orbit = world.get::<InOrbit>(fleet).expect("Fleet should be in orbit");
        assert_eq!(in_orbit.parent, planet_b);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Components (`src/layer2/fleet.rs`)

```rust
use bevy_ecs::prelude::*;

#[derive(Component, Debug, Clone, Copy)]
pub struct Fleet;

#[derive(Component, Debug, Clone, Copy)]
pub struct InOrbit {
    pub parent: Entity,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct InTransit {
    pub origin: Entity,
    pub destination: Entity,
    pub progress: f32, // 0.0 to 1.0
    pub duration: f32, // ticks
}

#[derive(Component, Debug, Clone, Copy)]
pub enum FleetOrder {
    MoveTo(Entity),
}

pub fn fleet_order_system(
    mut commands: Commands,
    query: Query<(Entity, &FleetOrder, Option<&InOrbit>), With<Fleet>>,
) {
    for (entity, order, maybe_orbit) in query.iter() {
        match order {
            FleetOrder::MoveTo(target) => {
                // Determine origin
                let origin = if let Some(orbit) = maybe_orbit {
                    orbit.parent
                } else {
                    // If already in transit, origin is tricky.
                    // For MVP, allow retargeting from "current location"?
                    // Or block retargeting.
                    // Implementation choice: Panic or Ignore?
                    // Let's assume start from orbit for now.
                    continue;
                };

                commands.entity(entity)
                    .remove::<FleetOrder>()
                    .remove::<InOrbit>()
                    .insert(InTransit {
                        origin,
                        destination: *target,
                        progress: 0.0,
                        duration: 100.0, // Fixed duration for MVP
                    });
            }
        }
    }
}

pub fn fleet_movement_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut InTransit)>,
) {
    for (entity, mut transit) in query.iter_mut() {
        // Increment progress
        let delta = 1.0 / transit.duration;
        transit.progress += delta;

        if transit.progress >= 1.0 {
            // Arrive
            let destination = transit.destination;
            commands.entity(entity)
                .remove::<InTransit>()
                .insert(InOrbit { parent: destination });
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Variable Speed**: `Fleet` should have a `Speed` stat. Duration = Distance / Speed.
- **Distance Calculation**: Need to query transforms/orbits of Origin and Destination to calculate Euclidean distance in system map.
- **Retargeting**: Handle `FleetOrder` when already `InTransit`. (E.g., create a dummy point in space or reverse logic).
- **Fuel**: Add `Fuel` component. Transit consumes fuel.
- **Visuals**: Update `src/layer2/render.rs` (from Spec 094) to handle `InTransit` rendering.
  - Position = `lerp(pos(origin), pos(dest), progress)`.
  - NOTE: `094` render system likely iterates `OrbitalBody`. If `Fleet` has `OrbitalBody` but no `Orbit` component (because it has `InTransit`), the renderer needs to handle that case explicitly.

## Acceptance Criteria

- [ ] `Fleet`, `InOrbit`, `InTransit`, `FleetOrder` components defined.
- [ ] `fleet_order_system` correctly transitions state from Orbit to Transit.
- [ ] `fleet_movement_system` correctly advances progress.
- [ ] Fleet arrives and transitions back to Orbit when progress >= 1.0.
- [ ] Tests pass.

## Technical Guidance

- In `src/layer2/mod.rs`, ensure `fleet` module is public.
- When updating the renderer (Layer 2 Render System), remember that `InTransit` fleets don't have polar coordinates relative to a single parent. You must resolve the (x,y) of `origin` and `destination` first, then lerp.
- `OrbitalBody` positions (from `094` logic):
  - `x = parent_x + radius * cos(angle)`
  - `y = parent_y + radius * sin(angle)`
  - Recursive resolution might be needed if moons orbit planets orbiting stars. For MVP, assume 1 level of depth or just use `Orbit` component.
