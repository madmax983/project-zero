# 074: Visitor System

## Overview

Introduces **Visitor Entities**—non-player characters who arrive at the colony, interact with facilities (Tavern, Trade Depot), and then leave.
- **Visitor Component**: Marks an entity as a temporary guest.
- **VisitorState**: Tracks lifecycle (`Arriving`, `Loitering`, `Departing`).
- **VisitorAI**: Simplified Utility AI that prioritizes `Leisure` and `Social` needs over work.
- **Spawning**: Visitors spawn at map edges (or specific entry points) and pathfind to points of interest.

This system lays the foundation for "The Inspector", "Galactic Tourism", and physical "Merchants".

## Dependencies

- `004` — Pop Entity (to reuse `Name`, `Position`, `Renderable`)
- `016` — Utility AI (for `ActionType::Leisure`, `Social`)
- `028` — Social Tavern (destination for visitors)
- `039` — Trade System (context for Merchants)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/visitor_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::visitor::{Visitor, VisitorState, VisitorSource, spawn_visitor_system, visitor_lifecycle_system};
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Name;
    use crate::shared::time::SimulationTime;

    #[test]
    fn test_visitor_component_defaults() {
        let visitor = Visitor::default();
        assert_eq!(visitor.state, VisitorState::Arriving);
        assert!(visitor.arrival_tick == 0);
        assert!(visitor.departure_tick > 0);
    }

    #[test]
    fn test_visitor_source_resource() {
        let mut world = World::new();
        world.insert_resource(VisitorSource {
            spawn_points: vec![GridPosition { x: 0, y: 0 }],
            ..Default::default()
        });

        let source = world.resource::<VisitorSource>();
        assert_eq!(source.spawn_points.len(), 1);
    }

    #[test]
    fn test_spawn_visitor_system() {
        let mut world = World::new();
        world.insert_resource(SimulationTime { tick: 100, ..Default::default() });
        world.insert_resource(VisitorSource {
            spawn_points: vec![GridPosition { x: 0, y: 0 }],
            next_spawn_tick: 100,
            ..Default::default()
        });

        // Run system
        spawn_visitor_system(&mut world);

        // Verify spawn
        let count = world.query::<&Visitor>().iter(&world).count();
        assert_eq!(count, 1);

        let (visitor, pos, name) = world.query::<(&Visitor, &GridPosition, &Name)>().single(&world);
        assert_eq!(visitor.arrival_tick, 100);
        assert_eq!(pos.x, 0);
        assert!(!name.first.is_empty());

        // Verify cooldown updated
        let source = world.resource::<VisitorSource>();
        assert!(source.next_spawn_tick > 100);
    }

    #[test]
    fn test_visitor_departure_lifecycle() {
        let mut world = World::new();
        world.insert_resource(SimulationTime { tick: 200, ..Default::default() });

        // Spawn visitor scheduled to leave at 200
        let entity = world.spawn((
            Visitor {
                state: VisitorState::Loitering,
                arrival_tick: 100,
                departure_tick: 200,
            },
            GridPosition { x: 10, y: 10 },
        )).id();

        // Run lifecycle system
        visitor_lifecycle_system(&mut world);

        // Check state change to Departing
        let visitor = world.get::<Visitor>(entity).unwrap();
        assert_eq!(visitor.state, VisitorState::Departing);
    }

    #[test]
    fn test_visitor_despawn_on_exit() {
        let mut world = World::new();
        world.insert_resource(SimulationTime { tick: 300, ..Default::default() });
        world.insert_resource(VisitorSource {
            spawn_points: vec![GridPosition { x: 0, y: 0 }], // Exit point
            ..Default::default()
        });

        // Spawn departing visitor at exit point
        let entity = world.spawn((
            Visitor {
                state: VisitorState::Departing,
                arrival_tick: 100,
                departure_tick: 200,
            },
            GridPosition { x: 0, y: 0 },
        )).id();

        // Run lifecycle system
        visitor_lifecycle_system(&mut world);

        // Verify entity despawned
        assert!(world.get::<Visitor>(entity).is_none());
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Components and Resources

```rust
// src/layer1/visitor.rs

use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VisitorState {
    #[default]
    Arriving,   // Moving from edge to colony center
    Loitering,  // Hanging out (Tavern, etc.)
    Departing,  // Moving from colony center to edge
}

#[derive(Component, Debug, Clone)]
pub struct Visitor {
    pub state: VisitorState,
    pub arrival_tick: u64,
    pub departure_tick: u64,
}

impl Default for Visitor {
    fn default() -> Self {
        Self {
            state: VisitorState::Arriving,
            arrival_tick: 0,
            departure_tick: 1000, // Default duration
        }
    }
}

#[derive(Resource, Default)]
pub struct VisitorSource {
    pub spawn_points: Vec<GridPosition>, // Map edges
    pub next_spawn_tick: u64,
}
```

### 2. Implement Spawn System

```rust
use crate::shared::time::SimulationTime;
use crate::layer1::pop::{Name, Pop}; // Reuse Pop visual/name logic
use rand::Rng;

pub fn spawn_visitor_system(world: &mut World) {
    let current_tick = world.resource::<SimulationTime>().tick;
    let mut source = world.resource_mut::<VisitorSource>();

    if current_tick >= source.next_spawn_tick && !source.spawn_points.is_empty() {
        let mut rng = rand::thread_rng();
        let spawn_idx = rng.gen_range(0..source.spawn_points.len());
        let spawn_pos = source.spawn_points[spawn_idx];

        // Calculate departure (e.g., stay for 1 day = 100 ticks)
        let departure = current_tick + 500;

        // Spawn entity
        world.spawn((
            Visitor {
                state: VisitorState::Arriving,
                arrival_tick: current_tick,
                departure_tick: departure,
            },
            GridPosition { x: spawn_pos.x, y: spawn_pos.y },
            Name::random(&mut rng), // Helper from Pop
            // Add visual component (char 'V' or similar color override)
        ));

        // Update cooldown
        source.next_spawn_tick = current_tick + rng.gen_range(2000..5000);
    }
}
```

### 3. Implement Lifecycle System

```rust
pub fn visitor_lifecycle_system(world: &mut World) {
    let current_tick = world.resource::<SimulationTime>().tick;
    let exit_points = world.resource::<VisitorSource>().spawn_points.clone();

    // We need command buffer or collect entities to modify
    let mut to_despawn = Vec::new();
    let mut to_depart = Vec::new();

    let mut query = world.query::<(Entity, &mut Visitor, &GridPosition)>();

    for (entity, mut visitor, pos) in query.iter_mut(world) {
        // State transitions
        match visitor.state {
            VisitorState::Arriving => {
                // Logic handled by AI (move to center).
                // Once near center/tavern, switch to Loitering.
                // For MVP, just switch after some time or distance?
                // Better: AI sets state.
                // Fallback: If time > arrival + 50, switch to Loitering.
                if current_tick > visitor.arrival_tick + 50 {
                    visitor.state = VisitorState::Loitering;
                }
            },
            VisitorState::Loitering => {
                if current_tick >= visitor.departure_tick {
                    visitor.state = VisitorState::Departing;
                }
            },
            VisitorState::Departing => {
                // Check if at exit point
                if exit_points.contains(pos) {
                    to_despawn.push(entity);
                }
            }
        }
    }

    // Apply changes
    for entity in to_despawn {
        world.despawn(entity);
    }
}
```

### 4. Utility AI Integration (Guidance)

In `src/layer1/utility_ai.rs` or `visitor_ai.rs`:
- Create `ActionType::VisitTavern` (high weight for `VisitorState::Loitering`).
- Create `ActionType::LeaveColony` (high weight for `VisitorState::Departing`).
- `Arriving` state should use standard movement to a random `BuildingType::Tavern` or `TradeDepot`.

## REFACTOR Phase: Quality & Design

- **Map Generation**: `VisitorSource` needs to be populated during map gen (find edge tiles).
- **Merchant Integration**: Eventually, `Merchant` logic should attach to a `Visitor` entity. When `Visitor` arrives at `TradeDepot`, the `MerchantState` becomes active.
- **Needs**: Visitors should have `Hunger` and `Leisure`. They buy food/drinks (consuming resources) and pay in `Credits` (if implemented) or just improve `Diplomacy` / `Chronicle` rating.
- **Visuals**: Give Visitors a distinct color (e.g., Purple) to distinguish from Colonists.

## Acceptance Criteria

- [ ] `Visitor` component and `VisitorState` enum exist.
- [ ] `VisitorSource` resource tracks spawn points.
- [ ] Visitors spawn periodically at map edges.
- [ ] Visitors transition from Arriving -> Loitering -> Departing based on time.
- [ ] Visitors despawn when they reach the edge in Departing state.
- [ ] Tests pass.

## Technical Guidance

- Use `crate::layer1::pop::Name::random()` to generate names.
- Ensure `VisitorSource` is initialized in `main.rs` or `simulation.rs`.
- For MVP AI, you can just make them wander if `Loitering`, and move to `VisitorSource` points if `Departing`.
