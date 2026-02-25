# 234: The Visitor

## Overview

Introduces "The Visitor", a massive, neutral, and indestructible entity that occasionally wanders onto the map.
Unlike hostile fauna (Spec 048), The Visitor does not hunt Pops. It hunts **Resources**.
It targets Stockpiles containing specific high-value items (Food, Energy, Rare Minerals).
It moves relentlessly towards its target, destroying walls and obstacles in its path ("Trample" mechanic).
After consuming its fill, it leaves the map.

This creates an emergent "Traffic Control" challenge: players must either let it eat (resource loss) or try to lure/divert it (risky). Attacking it is futile and may cause it to retaliate or simply ignore the damage.

## Dependencies

- `004` — Pop Entity (Movement/Rendering)
- `022` — Resource Stockpiles (Targeting)
- `002` — Basic Map (Terrain/Structure modification)
- `048` — Hostile Fauna (Base AI patterns)

## RED Phase: Tests First

Write these tests in `src/layer1/visitor_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::visitor::{Visitor, VisitorState, visitor_behavior_system};
    use crate::layer1::stockpile::Stockpile;
    use crate::layer1::map::{GridPosition, Map, TileType};
    use crate::layer1::structure::Structure; // Assuming Wall is a Structure
    use crate::layer1::items::ItemType;

    fn setup_world() -> World {
        let mut world = World::new();
        world.init_resource::<Map>();
        world
    }

    #[test]
    fn test_visitor_initialization() {
        let visitor = Visitor::default();
        assert_eq!(visitor.state, VisitorState::Wander);
        assert!(visitor.hunger > 0.0);
        assert!(visitor.health > 1000.0); // Indestructible-ish
    }

    #[test]
    fn test_visitor_detects_food_stockpile() {
        let mut world = setup_world();

        // Spawn Visitor
        let visitor = world.spawn((
            Visitor::default(),
            GridPosition { x: 0, y: 0 },
        )).id();

        // Spawn Stockpile with Food
        let stockpile = world.spawn((
            Stockpile {
                items: vec![(ItemType::Rations, 10)],
                ..Default::default()
            },
            GridPosition { x: 10, y: 0 },
        )).id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(visitor_behavior_system);
        schedule.run(&mut world);

        let v = world.get::<Visitor>(visitor).unwrap();
        assert_eq!(v.state, VisitorState::MoveToTarget);
        assert_eq!(v.target_stockpile, Some(stockpile));
    }

    #[test]
    fn test_visitor_tramples_walls() {
        let mut world = setup_world();
        let mut map = world.resource_mut::<Map>();

        // Place a wall at (1,0)
        map.set_tile(1, 0, TileType::Wall);
        // Note: Real implementation might use Structure entity for walls.
        // Assuming Structure component for destructible walls here:
        let wall_ent = world.spawn((
            Structure { durability: 100.0, ..Default::default() },
            GridPosition { x: 1, y: 0 },
        )).id();

        // Spawn Visitor at (0,0) moving to (2,0)
        let visitor = world.spawn((
            Visitor {
                state: VisitorState::MoveToTarget,
                target_position: Some(GridPosition { x: 2, y: 0 }),
                ..Default::default()
            },
            GridPosition { x: 0, y: 0 },
        )).id();

        // Run system (Movement step)
        let mut schedule = Schedule::default();
        schedule.add_systems(visitor_behavior_system);
        schedule.run(&mut world);

        // Visitor should be at (1,0)
        let pos = world.get::<GridPosition>(visitor).unwrap();
        assert_eq!(*pos, GridPosition { x: 1, y: 0 });

        // Wall should be destroyed (Entity despawned or durability 0)
        assert!(world.get::<Structure>(wall_ent).is_none());
    }

    #[test]
    fn test_visitor_eats_from_stockpile() {
        let mut world = setup_world();

        // Spawn Stockpile with 10 Food
        let stockpile = world.spawn((
            Stockpile {
                items: vec![(ItemType::Rations, 10)],
                ..Default::default()
            },
            GridPosition { x: 1, y: 0 },
        )).id();

        // Spawn Visitor at Stockpile
        let visitor = world.spawn((
            Visitor {
                state: VisitorState::Eat,
                target_stockpile: Some(stockpile),
                hunger: 5.0, // Needs 5 food
                ..Default::default()
            },
            GridPosition { x: 1, y: 0 },
        )).id();

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(visitor_behavior_system);
        schedule.run(&mut world);

        // Check Stockpile
        let s = world.get::<Stockpile>(stockpile).unwrap();
        // Should have 5 items left (10 - 5)
        let count = s.items.iter().find(|(t, _)| *t == ItemType::Rations).unwrap().1;
        assert_eq!(count, 5);

        // Visitor should be satisfied/leaving
        let v = world.get::<Visitor>(visitor).unwrap();
        assert_eq!(v.state, VisitorState::Leave);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define `Visitor` Component

```rust
// src/layer1/visitor.rs

use bevy_ecs::prelude::*;
use crate::layer1::pop::GridPosition;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VisitorState {
    #[default]
    Wander,
    DetectTarget,
    MoveToTarget,
    Eat,
    Leave,
}

#[derive(Component)]
pub struct Visitor {
    pub state: VisitorState,
    pub target_stockpile: Option<Entity>,
    pub target_position: Option<GridPosition>,
    pub hunger: f32, // Amount of resources to eat
    pub health: f32, // High value
    pub trample_damage: f32,
}

impl Default for Visitor {
    fn default() -> Self {
        Self {
            state: VisitorState::Wander,
            target_stockpile: None,
            target_position: None,
            hunger: 50.0,
            health: 5000.0,
            trample_damage: 1000.0,
        }
    }
}
```

### 2. Implement `visitor_behavior_system`

```rust
// src/layer1/visitor.rs

use crate::layer1::stockpile::Stockpile;
use crate::layer1::structure::Structure;
use crate::layer1::map::Map;

pub fn visitor_behavior_system(
    mut commands: Commands,
    mut visitors: Query<(Entity, &mut Visitor, &mut GridPosition)>,
    mut stockpiles: Query<(Entity, &mut Stockpile, &GridPosition), Without<Visitor>>,
    structures: Query<(Entity, &GridPosition), With<Structure>>,
    // map: Res<Map>, // If needed for terrain checks
) {
    for (visitor_ent, mut visitor, mut pos) in visitors.iter_mut() {
        match visitor.state {
            VisitorState::Wander => {
                // Check for stockpiles
                // Simple Euclidean distance for MVP
                let mut best_target = None;
                let mut min_dist = 1000; // Max range

                for (stock_ent, stockpile, stock_pos) in stockpiles.iter() {
                    if !stockpile.items.is_empty() { // Simplification: any item
                        let dist = pos.distance_chebyshev(stock_pos);
                        if dist < min_dist {
                            min_dist = dist;
                            best_target = Some((stock_ent, *stock_pos));
                        }
                    }
                }

                if let Some((target, target_pos)) = best_target {
                    visitor.state = VisitorState::MoveToTarget;
                    visitor.target_stockpile = Some(target);
                    visitor.target_position = Some(target_pos);
                }
            }
            VisitorState::MoveToTarget => {
                if let Some(target_pos) = visitor.target_position {
                    if *pos == target_pos {
                        visitor.state = VisitorState::Eat;
                    } else {
                        // Move one step closer (Chebyshev)
                        let dx = (target_pos.x as i32 - pos.x as i32).signum();
                        let dy = (target_pos.y as i32 - pos.y as i32).signum();
                        let next_pos = GridPosition {
                            x: (pos.x as i32 + dx) as u32,
                            y: (pos.y as i32 + dy) as u32,
                        };

                        // Check for Structure at next_pos and destroy it
                        for (struct_ent, struct_pos) in structures.iter() {
                            if *struct_pos == next_pos {
                                // Destroy structure
                                commands.entity(struct_ent).despawn_recursive();
                                // Add effect/log here
                            }
                        }

                        // Update position
                        *pos = next_pos;
                    }
                }
            }
            VisitorState::Eat => {
                if let Some(target) = visitor.target_stockpile {
                    if let Ok((_, mut stockpile, _)) = stockpiles.get_mut(target) {
                        // Consume items
                        for (_, count) in stockpile.items.iter_mut() {
                            if visitor.hunger <= 0.0 { break; }
                            let amount = (*count as f32).min(visitor.hunger);
                            *count -= amount as u32;
                            visitor.hunger -= amount;
                        }
                        stockpile.items.retain(|(_, c)| *c > 0);
                    }
                }
                visitor.state = VisitorState::Leave;
                visitor.target_stockpile = None;
                // Set target_position to map edge (0,0) or similar
                visitor.target_position = Some(GridPosition { x: 0, y: 0 });
            }
            VisitorState::Leave => {
                // Move towards exit
                // If at exit, despawn
                if let Some(target_pos) = visitor.target_position {
                     if *pos == target_pos {
                         commands.entity(visitor_ent).despawn();
                     } else {
                        // Move logic same as MoveToTarget (refactor into helper)
                        let dx = (target_pos.x as i32 - pos.x as i32).signum();
                        let dy = (target_pos.y as i32 - pos.y as i32).signum();
                        *pos = GridPosition {
                            x: (pos.x as i32 + dx) as u32,
                            y: (pos.y as i32 + dy) as u32,
                        };
                     }
                }
            }
            _ => {}
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Pathfinding**: Use A* but with `Wall` cost = 10 instead of infinite. This makes it prefer open ground but willing to smash walls if the path is significantly shorter.
- **Consumption Logic**: Specifically target `Food` and `Energy` items. Ignore `Stone` or `Scrap`.
- **Visuals**: Camera shake when moving. Particle effects when destroying walls.
- **Byproducts**: Visitor leaves `AlienDung` (fertilizer) or `ShedSkin` (rare material) every X steps.
- **Events**: Fire `VisitorArrivedEvent` and `StructureDestroyedEvent` for UI/Logs.

## Acceptance Criteria

- [ ] `Visitor` component implemented.
- [ ] Visitor spawns and idles or wanders.
- [ ] Visitor detects nearby Stockpiles with resources.
- [ ] Visitor moves towards target, destroying structures in the way.
- [ ] Visitor consumes resources from Stockpile.
- [ ] Visitor leaves map after eating.
- [ ] All tests in `visitor_tests.rs` pass.

## Technical Guidance

- Ensure `Structure` destruction handles child entities (like visuals) correctly via `despawn_recursive()`.
- Use `GridPosition::distance_chebyshev` for range checks.
- Be careful with `Query` mutability conflicts when iterating `visitors` and `stockpiles`. You might need to collect commands/updates and apply them after the loop.

## Questions

- **Q**: Does the Visitor take damage?
- **A**: Yes, but has massive health. If killed, drops massive loot.
- **Q**: Do turrets shoot it?
- **A**: Yes, if it is tagged `Hostile` (maybe only after it eats?). For MVP, treat as Neutral until attacked.
