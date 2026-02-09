# 070: Heirloom Tech

## Overview

Implements **Heirloom Technology**, representing advanced "Old World" machines that the colony starts with but cannot reproduce or repair.
- **Ancient Reactor**: Provides massive power but degrades over time.
- **Ancient Fabricator**: Provides efficient refining but degrades over time.
- **Heirloom Component**: Marks an entity as "Unrepairable" until specific late-game tech is unlocked (out of scope for now).
- **Decay**: Heirlooms suffer slow, constant damage (entropy) representing wear and tear that cannot be fixed.

This creates a "ticking clock" for the early game, forcing players to transition to sustainable (but less efficient) tech before the heirlooms fail.

## Dependencies

- `045` — Structure Durability (for `Structure` component and `current_hp`)
- `042` — Energy System (for `PowerSource` component)
- `023` — Refining Industry (for `Refinery` component)
- `006` — Building Placement (for `BuildingType`)

## RED Phase: Tests First

Write these tests in `src/layer1/heirloom_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::structure::{Structure, process_repair};
    use crate::layer1::heirloom::{Heirloom, heirloom_decay_system};
    use crate::layer1::energy::PowerSource;
    use crate::layer1::GridPosition;

    #[test]
    fn test_heirloom_component_exists() {
        let h = Heirloom; // Marker component
    }

    #[test]
    fn test_ancient_reactor_properties() {
        // Verify AncientReactor has high power output and structure
        let mut world = World::new();

        // Simulate spawning an AncientReactor (e.g., via a helper or directly)
        let entity = world.spawn((
            Building { building_type: BuildingType::AncientReactor },
            Structure { current_hp: 1000.0, max_hp: 1000.0 },
            PowerSource { output: 50.0 }, // High output
            Heirloom,
            GridPosition { x: 0, y: 0 },
        )).id();

        let power = world.get::<PowerSource>(entity).unwrap();
        assert_eq!(power.output, 50.0);

        let heirloom = world.get::<Heirloom>(entity);
        assert!(heirloom.is_some());
    }

    #[test]
    fn test_heirloom_decay() {
        let mut world = World::new();

        let entity = world.spawn((
            Building { building_type: BuildingType::AncientReactor },
            Structure { current_hp: 1000.0, max_hp: 1000.0 },
            Heirloom,
            GridPosition { x: 0, y: 0 },
        )).id();

        // Run decay system
        heirloom_decay_system(&mut world);

        let structure = world.get::<Structure>(entity).unwrap();
        assert!(structure.current_hp < 1000.0);
        assert!(structure.current_hp > 990.0); // Should be slow decay
    }

    #[test]
    fn test_repair_prevention_on_heirloom() {
        let mut world = World::new();

        let entity = world.spawn((
            Building { building_type: BuildingType::AncientReactor },
            Structure { current_hp: 500.0, max_hp: 1000.0 },
            Heirloom,
            GridPosition { x: 0, y: 0 },
        )).id();

        // Designate for repair (mock designation entity)
        let designation = world.spawn((
            crate::layer1::designation::Designation {
                designation_type: crate::layer1::designation::DesignationType::Repair
            },
            GridPosition { x: 0, y: 0 },
        )).id();

        // Attempt repair
        // Assuming process_repair returns bool or modifies HP
        // If process_repair logic is updated to check Heirloom, it should do nothing.

        // We might need to check if the repair logic respects the Heirloom tag.
        // For this test, we assume process_repair will be modified.
        process_repair(&mut world, designation, 10.0);

        let structure = world.get::<Structure>(entity).unwrap();
        assert_eq!(structure.current_hp, 500.0); // No change
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define `Heirloom` Component

In `src/layer1/heirloom.rs`:

```rust
use bevy_ecs::prelude::*;

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Heirloom;
```

### 2. Update `BuildingType`

In `src/layer1/building.rs`:

```rust
pub enum BuildingType {
    // ...
    AncientReactor,
    AncientFabricator,
}

// Update properties:
// AncientReactor: Cost (None/Unbuildable), Char 'R', Color Red/Gold
// AncientFabricator: Cost (None), Char 'F', Color Cyan/Gold
```

### 3. Implement `heirloom_decay_system`

In `src/layer1/heirloom.rs`:

```rust
use crate::layer1::structure::Structure;

pub fn heirloom_decay_system(mut query: Query<&mut Structure, With<Heirloom>>) {
    // Decay rate per tick. e.g., 0.05 HP/tick.
    // 1000 HP / 0.05 = 20,000 ticks = ~200 days (2 years).
    let decay_rate = 0.05;

    for mut structure in query.iter_mut() {
        structure.current_hp -= decay_rate;
        if structure.current_hp < 0.0 {
            structure.current_hp = 0.0;
            // Destruction handled by structure check system
        }
    }
}
```

### 4. Update `process_repair`

In `src/layer1/structure.rs`:

```rust
use crate::layer1::heirloom::Heirloom;

pub fn process_repair(world: &mut World, designation_entity: Entity, amount: f32) {
    // ... find building ...

    // Check if building has Heirloom component
    if let Some(building_entity) = find_building_at_pos(world, pos) {
        if world.get::<Heirloom>(building_entity).is_some() {
            // Cannot repair Heirloom!
            // Maybe log a message or show notification "Cannot repair Ancient Tech"
            // Remove designation to stop workers from trying forever
            world.despawn(designation_entity);
            return;
        }

        // ... proceed with repair ...
    }
}
```

### 5. Spawn Heirlooms at Game Start

In `src/layer1/map.rs` or `setup.rs` (wherever initial map is generated):
- Place 1 `AncientReactor` and 1 `AncientFabricator` near the center.

## REFACTOR Phase: Quality & Design

- **Visual Feedback**: Heirlooms should visually look different (e.g., gold color).
- **Notifications**: When Heirloom HP drops below 25%, warn the player.
- **Explosion**: When Reactor dies, maybe it explodes? (Future feature).
- **Tech Unlock**: Eventually, "Xeno-Engineering" tech could allow repair.
- **Variable Decay**: Decay could increase if the machine is "working hard" (e.g. producing power). For now, constant decay is fine.

## Acceptance Criteria

- [ ] `Heirloom` component exists.
- [ ] `AncientReactor` and `AncientFabricator` types exist.
- [ ] Heirlooms lose HP over time automatically.
- [ ] Heirlooms cannot be repaired by standard workers.
- [ ] One of each spawns at game start.
- [ ] Tests pass.

## Technical Guidance

- Ensure `heirloom_decay_system` is added to the schedule in `simulation.rs`.
- The `process_repair` function needs access to the `World` to query for `Heirloom`. Ensure borrow rules are respected if calling from within a system.
