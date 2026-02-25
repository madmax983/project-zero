# 223: Volatile Resources

## Overview

Introduces **Volatile Resources**—items that degrade over time and explode if not used or stabilized. This adds a logistical challenge: "Just-In-Time" delivery is required for dangerous materials (e.g., `Plasma Gel`, `Antimatter Cells`).

- **Volatility**: Items have a `Volatile` component with `stability` and `decay_rate`.
- **Explosion**: When `stability` reaches 0, the item despawns and triggers an explosion, damaging nearby buildings and pops.
- **Stabilization**: Certain storage or conditions (e.g., "Cryo-Freezer") can pause decay (future scope, but component should support it).

## Dependencies

- `018` Mining Resources (for Item basics)
- `206` Orbital Crossfire (reusing or mirroring explosion logic)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/volatile_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::items::{Item, ItemType};
    use crate::layer1::map::GridPosition;
    use crate::layer1::volatile::{Volatile, volatile_decay_system, ExplosionEvent};
    use crate::shared::time::SimulationTime;

    #[test]
    fn test_volatile_component_initialization() {
        let volatile = Volatile {
            stability: 100.0,
            decay_rate: 10.0,
            explosion_power: 50.0,
            explosion_radius: 2,
            paused: false,
        };
        assert_eq!(volatile.stability, 100.0);
        assert!(!volatile.paused);
    }

    #[test]
    fn test_decay_system_reduces_stability() {
        let mut world = World::new();
        world.insert_resource(SimulationTime::default()); // Time resource

        let entity = world.spawn(Volatile {
            stability: 100.0,
            decay_rate: 10.0,
            explosion_power: 10.0,
            explosion_radius: 1,
            paused: false,
        }).id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(volatile_decay_system);
        schedule.run(&mut world);

        let v = world.get::<Volatile>(entity).unwrap();
        assert!(v.stability < 100.0);
    }

    #[test]
    fn test_decay_paused_does_not_reduce() {
        let mut world = World::new();
        world.insert_resource(SimulationTime::default());

        let entity = world.spawn(Volatile {
            stability: 100.0,
            decay_rate: 10.0,
            explosion_power: 10.0,
            explosion_radius: 1,
            paused: true,
        }).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(volatile_decay_system);
        schedule.run(&mut world);

        let v = world.get::<Volatile>(entity).unwrap();
        assert_eq!(v.stability, 100.0);
    }

    #[test]
    fn test_explosion_trigger_at_zero_stability() {
        let mut world = World::new();
        world.insert_resource(SimulationTime::default());
        // Need Events resource
        world.init_resource::<Events<ExplosionEvent>>();

        let pos = GridPosition { x: 5, y: 5 };
        let entity = world.spawn((
            Volatile {
                stability: 5.0, // Low stability
                decay_rate: 10.0, // Should reach 0 in one tick
                explosion_power: 50.0,
                explosion_radius: 2,
                paused: false,
            },
            pos,
        )).id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(volatile_decay_system);
        schedule.run(&mut world);

        // Entity should be despawned
        assert!(world.get_entity(entity).is_err());

        // Event should be fired
        let events = world.resource::<Events<ExplosionEvent>>();
        let mut reader = events.get_reader();
        let event = reader.read(events).next();
        assert!(event.is_some());
        let e = event.unwrap();
        assert_eq!(e.center, pos);
        assert_eq!(e.damage, 50.0);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. `Volatile` Component and `ExplosionEvent`

```rust
// src/layer1/volatile.rs

use bevy_ecs::prelude::*;
use crate::layer1::GridPosition;

#[derive(Component, Debug, Clone)]
pub struct Volatile {
    pub stability: f32,
    pub decay_rate: f32, // Stability loss per tick (or second)
    pub explosion_power: f32,
    pub explosion_radius: u32,
    pub paused: bool,
}

#[derive(Event, Debug, Clone)]
pub struct ExplosionEvent {
    pub center: GridPosition,
    pub damage: f32,
    pub radius: u32,
}
```

### 2. `volatile_decay_system`

```rust
// src/layer1/volatile.rs

pub fn volatile_decay_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Volatile, &GridPosition)>,
    mut events: EventWriter<ExplosionEvent>,
    // time: Res<SimulationTime>, // If using delta time
) {
    for (entity, mut volatile, pos) in query.iter_mut() {
        if volatile.paused {
            continue;
        }

        volatile.stability -= volatile.decay_rate;

        if volatile.stability <= 0.0 {
            // Explode
            events.send(ExplosionEvent {
                center: *pos,
                damage: volatile.explosion_power,
                radius: volatile.explosion_radius,
            });

            // Despawn item
            commands.entity(entity).despawn_recursive();
        }
    }
}
```

### 3. `handle_explosion_system`

This system processes `ExplosionEvent` to damage nearby entities. It can reuse or mimic `impact_system` from `orbital_crossfire.rs`.

```rust
// src/layer1/volatile.rs

use crate::layer1::structure::Structure;
use crate::layer1::pop::Pop; // Assuming Pop has Health or handle damage
use crate::layer1::health::Health; // If Health component exists

pub fn handle_explosion_system(
    mut events: EventReader<ExplosionEvent>,
    mut structures: Query<(&GridPosition, &mut Structure)>,
    // mut pops: Query<(&GridPosition, &mut Health), With<Pop>>,
) {
    for event in events.read() {
        // Damage structures
        for (pos, mut structure) in structures.iter_mut() {
            if is_in_radius(event.center, *pos, event.radius) {
                structure.current_hp -= event.damage;
                // De-spawning destroyed structures is handled by another system usually?
                // Or handle it here if needed.
            }
        }

        // Damage Pops (similarly)
    }
}

fn is_in_radius(center: GridPosition, target: GridPosition, radius: u32) -> bool {
    let dx = (center.x - target.x).abs();
    let dy = (center.y - target.y).abs();
    // Chebyshev distance (square) or Euclidean?
    // Usually Manhattan or Chebyshev for grid. Let's use Chebyshev for "Radius".
    dx.max(dy) as u32 <= radius
}
```

## REFACTOR Phase: Quality & Design

- **Integration with `OrbitalEvent`**: Consider making `ExplosionEvent` the generic event for both Orbital Impacts and Volatile explosions to avoid code duplication.
- **UI Feedback**: Volatile items should show a progress bar for stability in the Inspector.
- **Logistics**: Haulers should prioritize Volatile items to get them to destination (or stabilization) quickly.
- **Storage**: "Freezer" or "Containment" buildings should set `paused: true` on Volatile items stored within them.

## Acceptance Criteria

- [ ] `Volatile` component exists.
- [ ] Items decay over time unless paused.
- [ ] Items explode (despawn + event) when stability hits 0.
- [ ] Explosion damages structures within radius.
- [ ] Tests pass.

## Technical Guidance

- Register `ExplosionEvent` in `src/simulation.rs`.
- Register systems in `src/simulation.rs`.
- Ensure `is_in_radius` logic matches game grid distance standards (usually Manhattan or Chebyshev).
