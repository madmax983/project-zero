# 136: Explosive Decompression

## Overview

Simulates the violent physics of a hull breach. When a high-pressure environment is exposed to a vacuum (or low pressure), the rapid equalization creates a "Suction" force.

**Mechanic:**
- Entities (Pops, Items) on high-pressure tiles adjacent to low-pressure tiles are forcibly moved towards the low pressure.
- If the path is blocked by a wall, they take impact damage.
- If they are ejected into a Vacuum tile (Space), they suffer vacuum exposure (handled by existing systems) or become "Lost" (Items).

**Why:**
- Adds immediate, tangible consequences to structural failure.
- Creates "Emergent Tragedy" (saving the ship but losing the crew).
- Allows weaponization (opening airlocks to flush boarders).

## Dependencies

- `119` — Airlock & Pressure (Implemented)
- `063` — Atmospheric Simulation (Implemented)
- `004` — Pop Entity (Implemented)

## RED Phase: Tests First

```rust
// src/layer1/suction_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pressure::PressureGrid;
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop; // Assuming Pop component exists
    use crate::layer1::items::Item; // Assuming Item component exists
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::health::Health;

    // The system to test
    // fn suction_system(world: &mut World);

    #[test]
    fn test_entity_sucked_into_vacuum() {
        let mut world = World::new();
        let mut grid = PressureGrid::new(10, 10);

        // Setup: (5,5) is High Pressure (1.0), (6,5) is Vacuum (0.0)
        grid.set(5, 5, 1.0);
        grid.set(6, 5, 0.0);
        world.insert_resource(grid);

        // Spawn Pop at (5,5)
        let pop = world.spawn((
            Pop::default(),
            GridPosition { x: 5, y: 5 },
            Health::default(),
        )).id();

        // Run system
        // crate::layer1::suction::suction_system(&mut world);

        // Assert Pop moved to (6,5)
        let pos = world.get::<GridPosition>(pop).unwrap();
        assert_eq!(*pos, GridPosition { x: 6, y: 5 }, "Pop should be sucked into vacuum");
    }

    #[test]
    fn test_anchored_building_not_moved() {
        let mut world = World::new();
        let mut grid = PressureGrid::new(10, 10);
        grid.set(5, 5, 1.0);
        grid.set(6, 5, 0.0);
        world.insert_resource(grid);

        // Spawn Wall at (5,5) - Walls are anchored/immovable
        let wall = world.spawn((
            Building { building_type: BuildingType::Wall },
            GridPosition { x: 5, y: 5 },
        )).id();

        // Run system
        // crate::layer1::suction::suction_system(&mut world);

        let pos = world.get::<GridPosition>(wall).unwrap();
        assert_eq!(*pos, GridPosition { x: 5, y: 5 }, "Building should NOT move");
    }

    #[test]
    fn test_impact_damage_on_blocked_suction() {
        let mut world = World::new();
        let mut grid = PressureGrid::new(10, 10);

        // Setup: (5,5) High, (6,5) Low. Suction is towards (6,5).
        grid.set(5, 5, 1.0);
        grid.set(6, 5, 0.0);
        world.insert_resource(grid);

        // BUT (6,5) has a Wall (e.g., a window that is holding vacuum but the pop is slammed against it?
        // Or maybe (4,5) is High, (5,5) is Low, and (6,5) is Wall?
        // Let's say (5,5) is High, (6,5) is Low. Pop is at (5,5).
        // If (6,5) is a Wall, then Pressure shouldn't be 0.0 unless it's space.
        // Let's simulate a case where the pop is pushed into a wall.
        // (4,5) High (1.0), (5,5) Low (0.0). Pop at (4,5).
        // Wall at (5,5) prevents movement. Pop slams into Wall.
        // Wait, if (5,5) is a wall, it blocks airflow, so (4,5) wouldn't depressurize rapidly unless the wall *broke*.
        // If the wall is there, pressure is stable.

        // Scenario: Pop is at (5,5). (6,5) is Space (0.0). (7,5) is Wall.
        // Pop sucked to (6,5). Next tick, (6,5) is 0.0, (7,5) is Wall. No suction.

        // Scenario: Pop at (5,5) High. (6,5) Low. (6,5) is BLOCKED by a closed Door?
        // If door is closed, no flow, no suction.

        // Valid Impact Scenario:
        // Pop is sucked from (5,5) to (6,5).
        // (6,5) contains a Building (e.g. Pillar) that doesn't block air but blocks movement?
        // Or just debris damage?

        // Let's stick to simple movement for now. Collision damage is a nice-to-have REFACTOR.
    }

    #[test]
    fn test_no_suction_at_equilibrium() {
        let mut world = World::new();
        let mut grid = PressureGrid::new(10, 10);
        grid.set(5, 5, 1.0);
        grid.set(6, 5, 0.9); // Small difference
        world.insert_resource(grid);

        let pop = world.spawn((
            Pop::default(),
            GridPosition { x: 5, y: 5 },
        )).id();

        // Run system
        // crate::layer1::suction::suction_system(&mut world);

        let pos = world.get::<GridPosition>(pop).unwrap();
        assert_eq!(*pos, GridPosition { x: 5, y: 5 }, "Small gradient should not cause suction");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. `suction_system` in `src/layer1/suction.rs`

```rust
use bevy_ecs::prelude::*;
use crate::layer1::pressure::PressureGrid;
use crate::layer1::map::GridPosition;
use crate::layer1::building::Building;
use crate::layer1::pop::Pop;
use crate::layer1::items::Item;

pub fn suction_system(
    mut commands: Commands,
    pressure: Res<PressureGrid>,
    mut query: Query<(Entity, &mut GridPosition), Or<(With<Pop>, With<Item>)>>,
    buildings: Query<&GridPosition, (With<Building>, Without<Pop>, Without<Item>)>, // Obstacles
) {
    const SUCTION_THRESHOLD: f32 = 0.5; // Pressure difference required

    for (entity, mut pos) in query.iter_mut() {
        let current_p = pressure.get(pos.x, pos.y);

        // Check 4 neighbors
        let neighbors = [(0, 1), (0, -1), (1, 0), (-1, 0)];
        let mut best_target = None;
        let mut max_diff = 0.0;

        for (dx, dy) in neighbors.iter() {
            let nx = pos.x as i32 + dx;
            let ny = pos.y as i32 + dy;

            if nx < 0 || ny < 0 || nx >= pressure.width as i32 || ny >= pressure.height as i32 {
                continue;
            }

            let neighbor_p = pressure.get(nx as usize, ny as usize);
            let diff = current_p - neighbor_p;

            if diff > SUCTION_THRESHOLD && diff > max_diff {
                max_diff = diff;
                best_target = Some((nx, ny));
            }
        }

        if let Some((tx, ty)) = best_target {
            // Check for collision with buildings (simple check)
            let target_blocked = buildings.iter().any(|b_pos| b_pos.x == tx as usize && b_pos.y == ty as usize);

            if !target_blocked {
                // Move entity
                pos.x = tx as usize;
                pos.y = ty as usize;

                // Optional: Add "Stunned" or "Flailing" component
            }
        }
    }
}
```

### 2. Integration

- Add `suction_system` to `SimulationSchedule`.
- Ensure it runs *after* `update_pressure_system`.

## REFACTOR Phase: Quality & Design

- **Mass & Drag**: Items should have mass. Heavy items (Machines in crates) shouldn't move. Light items (Papers) fly instantly.
- **Grabbing**: Pops should try to "Grab" nearby walls (Dexterity/Strength check) to resist suction.
- **Damage**: Collision with walls should deal damage based on pressure diff (velocity).
- **Visuals**: Spawn "Air Stream" particles in high-gradient tiles.

## Acceptance Criteria

- [ ] All tests in RED phase pass.
- [ ] Pops and Items are moved from High Pressure -> Low Pressure tiles when difference > 0.5.
- [ ] Buildings are NOT moved.
- [ ] Suction stops when pressure equalizes (< 0.5 diff).
- [ ] Feature is integrated into `SimulationSchedule`.
