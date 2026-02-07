# 048: Hostile Fauna

## Overview

Introduces **Hostile Fauna** to the colony simulation.
Wild animals (e.g., Space Rats, Wolves) spawn on the map and pose a threat to colonists and food supplies.
This spec focuses on the **Fauna entity**, its **AI behavior** (Wander, Chase, Attack), and its interaction with the **Health system**.

This provides the primary external pressure for the colony, justifying the need for defenses (Spec 043) and medical care (Spec 038).

## Dependencies

- `034` — Pop Health (Health component)
- `004` — Pop Entity (Movement/Rendering foundations)
- `016` — Utility AI (Foundations for AI, though Fauna use a simpler State Machine)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/fauna_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::fauna::{Fauna, FaunaType, FaunaState, fauna_behavior_system};
    use crate::layer1::health::Health;
    use crate::layer1::pop::{Pop, GridPosition};
    use crate::layer1::map::Map; // or TerrainGrid if Map not available
    use crate::layer1::execution::{MovementTarget, AtTarget};

    // Helper to create a basic world with necessary resources
    fn setup_world() -> World {
        let mut world = World::new();
        // Mock necessary resources if needed (e.g. Time, Grid)
        world.insert_resource(crate::layer1::info::SimulationTime::default());
        world
    }

    #[test]
    fn test_fauna_spawn_defaults() {
        let rat = Fauna { fauna_type: FaunaType::SpaceRat, ..Default::default() };
        assert_eq!(rat.state, FaunaState::Wander);
        assert!(rat.detection_range > 0.0);
        assert!(rat.attack_damage > 0.0);
    }

    #[test]
    fn test_fauna_detects_target_in_range() {
        let mut world = setup_world();

        // Spawn Wolf
        let wolf = world.spawn((
            Fauna { fauna_type: FaunaType::Wolf, detection_range: 5.0, ..Default::default() },
            GridPosition { x: 0, y: 0 },
            Health::default(),
        )).id();

        // Spawn Pop nearby
        let pop = world.spawn((
            Pop,
            GridPosition { x: 2, y: 0 },
            Health::default(),
        )).id();

        // Run behavior system
        fauna_behavior_system(&mut world);

        // Wolf should be Chasing pop
        let wolf_comp = world.get::<Fauna>(wolf).unwrap();
        assert_eq!(wolf_comp.state, FaunaState::Chase);
        assert_eq!(wolf_comp.target, Some(pop));
    }

    #[test]
    fn test_fauna_ignores_target_out_of_range() {
        let mut world = setup_world();

        let wolf = world.spawn((
            Fauna { fauna_type: FaunaType::Wolf, detection_range: 5.0, ..Default::default() },
            GridPosition { x: 0, y: 0 },
            Health::default(),
        )).id();

        // Spawn Pop far away
        world.spawn((
            Pop,
            GridPosition { x: 10, y: 0 },
            Health::default(),
        ));

        fauna_behavior_system(&mut world);

        let wolf_comp = world.get::<Fauna>(wolf).unwrap();
        assert_eq!(wolf_comp.state, FaunaState::Wander);
        assert_eq!(wolf_comp.target, None);
    }

    #[test]
    fn test_fauna_attacks_adjacent_target() {
        let mut world = setup_world();

        // Spawn Wolf adjacent to Pop
        let wolf = world.spawn((
            Fauna { fauna_type: FaunaType::Wolf, attack_damage: 10.0, ..Default::default() },
            GridPosition { x: 0, y: 0 },
            Health::default(),
        )).id(); // Wolf

        let pop = world.spawn((
            Pop,
            GridPosition { x: 1, y: 0 }, // Adjacent
            Health { current: 100.0, max: 100.0 },
        )).id();

        // Manually set state to Chase/Attack for test setup
        // Real system would transition Chase -> Attack if adjacent
        // Here we test the damage application
        let mut wolf_mut = world.get_mut::<Fauna>(wolf).unwrap();
        wolf_mut.state = FaunaState::Chase;
        wolf_mut.target = Some(pop);

        fauna_behavior_system(&mut world);

        // Pop should take damage
        let health = world.get::<Health>(pop).unwrap();
        assert!(health.current < 100.0);
        // Wolf should stay in Chase/Attack mode
        let wolf_comp = world.get::<Fauna>(wolf).unwrap();
        assert!(matches!(wolf_comp.state, FaunaState::Chase | FaunaState::Attack));
    }

    #[test]
    fn test_fauna_sets_movement_target() {
        let mut world = setup_world();

        let wolf = world.spawn((
            Fauna { fauna_type: FaunaType::Wolf, ..Default::default() },
            GridPosition { x: 0, y: 0 },
            Health::default(),
        )).id();

        let pop = world.spawn((
            Pop,
            GridPosition { x: 5, y: 0 }, // Far enough to move, close enough to chase
            Health::default(),
        )).id();

        // Setup chase state
        let mut wolf_mut = world.get_mut::<Fauna>(wolf).unwrap();
        wolf_mut.state = FaunaState::Chase;
        wolf_mut.target = Some(pop);
        wolf_mut.detection_range = 10.0;

        fauna_behavior_system(&mut world);

        // Should have MovementTarget component added
        assert!(world.get::<MovementTarget>(wolf).is_some());
        let target = world.get::<MovementTarget>(wolf).unwrap();
        assert_eq!(target.target_position, GridPosition { x: 5, y: 0 });
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define `Fauna` Component and Enums

```rust
// src/layer1/fauna.rs

use bevy_ecs::prelude::*;
use crate::layer1::pop::GridPosition;
use crate::layer1::health::Health;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FaunaType {
    #[default]
    Wolf,
    SpaceRat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FaunaState {
    #[default]
    Wander,
    Chase,
    Attack,
    Flee,
}

#[derive(Component)]
pub struct Fauna {
    pub fauna_type: FaunaType,
    pub state: FaunaState,
    pub target: Option<Entity>,
    pub detection_range: f32,
    pub attack_damage: f32,
    pub attack_cooldown: u32, // Ticks until next attack
}

impl Default for Fauna {
    fn default() -> Self {
        Self {
            fauna_type: FaunaType::Wolf,
            state: FaunaState::Wander,
            target: None,
            detection_range: 8.0,
            attack_damage: 5.0,
            attack_cooldown: 0,
        }
    }
}
```

### 2. Implement `fauna_behavior_system`

This system handles the State Machine logic for all `Fauna` entities.

```rust
// src/layer1/fauna.rs

use crate::layer1::pop::Pop;
use crate::layer1::execution::{MovementTarget, AtTarget};
use crate::layer1::utility_ai::ActionType; // Reuse for movement context

pub fn fauna_behavior_system(world: &mut World) {
    // 1. Query all Fauna
    let mut fauna_updates = Vec::new();
    let mut attacks = Vec::new();

    // We need to query potential targets (Pops)
    // For performance in Green phase, simple iteration is acceptable.
    // Refactor phase will use spatial lookup.
    let pops: Vec<(Entity, GridPosition)> = world
        .query::<(Entity, &GridPosition, With<Pop>)>()
        .iter(world)
        .map(|(e, p, _)| (e, *p))
        .collect();

    let mut query = world.query::<(Entity, &mut Fauna, &GridPosition)>();

    for (entity, mut fauna, pos) in query.iter_mut(world) {
        if fauna.attack_cooldown > 0 {
            fauna.attack_cooldown -= 1;
        }

        match fauna.state {
            FaunaState::Wander => {
                // Look for targets
                let mut best_target = None;
                let mut min_dist = fauna.detection_range;

                for (target_e, target_pos) in &pops {
                    let dist = pos.distance_chebyshev(target_pos) as f32;
                    if dist <= min_dist {
                        min_dist = dist;
                        best_target = Some(*target_e);
                    }
                }

                if let Some(target) = best_target {
                    fauna.state = FaunaState::Chase;
                    fauna.target = Some(target);
                } else {
                    // Random wander logic (set MovementTarget to random adjacent tile)
                    // If no movement target, pick one.
                    // (Simplified for Green phase: just stay idle or simple random walk)
                }
            }
            FaunaState::Chase => {
                if let Some(target) = fauna.target {
                    // Check if target still valid
                    if let Some((_, target_pos)) = pops.iter().find(|(e, _)| *e == target) {
                        let dist = pos.distance_chebyshev(target_pos);

                        if dist <= 1 {
                            // Adjacent -> Attack
                            if fauna.attack_cooldown == 0 {
                                attacks.push((target, fauna.attack_damage));
                                fauna.attack_cooldown = 10; // Cooldown ticks
                            }
                            // Stay in Chase/Attack state (Attack is transient or just an event)
                        } else if dist as f32 > fauna.detection_range * 1.5 {
                            // Lost target
                            fauna.state = FaunaState::Wander;
                            fauna.target = None;
                        } else {
                            // Move towards target
                            // We need to add MovementTarget component.
                            // Since we are iterating mutably, we can't add components easily here.
                            // We will collect updates.
                            fauna_updates.push((entity, *target_pos));
                        }
                    } else {
                        // Target dead/gone
                        fauna.state = FaunaState::Wander;
                        fauna.target = None;
                    }
                }
            }
            _ => {}
        }
    }

    // Apply movement updates
    for (entity, target_pos) in fauna_updates {
        // Reuse MovementTarget.
        // Note: Movement system needs to support entities without Pop component if it checks for Pop.
        // Spec 004 implies MovementTarget is generic.
        world.entity_mut(entity).insert(MovementTarget {
            target_entity: Entity::PLACEHOLDER, // Or actual target
            target_position: target_pos,
            for_action: ActionType::Idle, // Dummy action
        });
    }

    // Apply damage
    for (target, damage) in attacks {
        if let Some(mut health) = world.get_mut::<Health>(target) {
            health.take_damage(damage);
        }
    }
}
```

### 3. Rendering Update

Ensure `RenderEntity` supports `Fauna`.
In `src/layer1/pop.rs` or `rendering.rs`:
```rust
// Map FaunaType to char/color
match fauna.fauna_type {
    FaunaType::Wolf => ('w', Color::Red),
    FaunaType::SpaceRat => ('r', Color::DarkGray),
}
```

## REFACTOR Phase: Quality & Design

- **Spatial Optimization**: The O(N*M) loop for detection (Fauna * Pops) will act poorly with many entities. Use a spatial grid or `OccupiedTiles` lookup if possible.
- **Movement Integration**: Ensure `movement_system` doesn't crash on non-Pop entities. It should rely on `GridPosition` and `MovementTarget` only.
- **State Machine**: Consider a more robust State pattern if complexity grows (e.g. `Trait State`).
- **Loot**: Add `LootTable` to Fauna so they drop Meat/Leather on death.
- **Combat Feedback**: Spawn visual effects or logs when attack happens ("Wolf bit Colonist!").

## Acceptance Criteria

- [ ] `Fauna` component exists with `Wolf` and `SpaceRat` types.
- [ ] Fauna spawn logic is tested (unit test).
- [ ] Fauna detects Pops within range.
- [ ] Fauna moves towards target (Chase).
- [ ] Fauna deals damage when adjacent (Attack).
- [ ] Tests pass.

## Technical Guidance

- `GridPosition::distance_chebyshev` is your friend.
- Do not put `MovementTarget` insertion inside the query loop if it conflicts with borrow rules. Collect commands and apply after.
- Ensure `Health` component is on Fauna so they can be killed too.
- `ActionType` is used by `MovementTarget`. You might need to add `ActionType::Combat` or generic `Move` if `Idle` is confusing.

## Questions

- Should Fauna attack buildings? (Maybe walls, eventually. For now, Pops only).
- Do Fauna eat crops? (Space Rats should eat crops. Future expansion).
