# 086: The Stowaway

## Overview

Visitors are not always friendly. A **Stowaway** is a hidden entity that arrives with a visitor group but secretly detaches to hide inside a building (like a Stockpile or Warehouse). While hidden, they consume global resources (Food) and may cause other disruptions. They can be discovered by workers or through random events, at which point they emerge as a new Pop entity (Refugee, Thief, or Saboteur).

## Dependencies

- `074` — Visitor System (Arrival mechanism)
- `022` — Resource Stockpiles (Hiding spots)
- `016` — Utility AI (Behavior after reveal)

## RED Phase: Tests First

Write these tests in `src/layer1/stowaway_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::stowaway::{Stowaway, InfiltrationRisk, infiltration_system, theft_system, discovery_system};
    use crate::layer1::visitor::{Visitor, VisitorState};
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::map::GridPosition;
    use crate::shared::time::SimulationTime;

    #[test]
    fn test_stowaway_component_defaults() {
        let stowaway = Stowaway::default();
        assert_eq!(stowaway.stealth, 1.0); // 100% hidden
        assert_eq!(stowaway.hunger, 0.0);
    }

    #[test]
    fn test_infiltration_adds_stowaway_to_building() {
        let mut world = World::new();

        // Spawn Visitor near Building
        let visitor = world.spawn((
            Visitor { state: VisitorState::Loitering, ..Default::default() },
            GridPosition { x: 10, y: 10 },
            InfiltrationRisk { chance: 1.0 }, // Force infiltration
        )).id();

        let building = world.spawn((
            Building { building_type: BuildingType::Stockpile },
            GridPosition { x: 10, y: 10 },
        )).id();

        // Run system
        infiltration_system(&mut world);

        // Visitor should be despawned (or removed from world/transformed)
        assert!(world.get::<Visitor>(visitor).is_none());

        // Building should have Stowaway component
        assert!(world.get::<Stowaway>(building).is_some());
    }

    #[test]
    fn test_stowaway_steals_food() {
        let mut world = World::new();
        world.insert_resource(ColonyResources { food: 100.0, ..Default::default() });
        world.insert_resource(SimulationTime { tick: 100, ..Default::default() });

        // Spawn Building with Stowaway
        world.spawn((
            Building { building_type: BuildingType::Stockpile },
            Stowaway { hunger: 50.0, ..Default::default() }, // Hungry
        ));

        // Run system
        theft_system(&mut world);

        let resources = world.resource::<ColonyResources>();
        assert!(resources.food < 100.0, "Food should be stolen");
    }

    #[test]
    fn test_discovery_removes_component_spawns_pop() {
        let mut world = World::new();

        // Spawn Building with Stowaway (low stealth)
        let building = world.spawn((
            Building { building_type: BuildingType::Stockpile },
            Stowaway { stealth: 0.0, ..Default::default() }, // Revealed
            GridPosition { x: 5, y: 5 },
        )).id();

        // Run system
        discovery_system(&mut world);

        // Stowaway component removed
        assert!(world.get::<Stowaway>(building).is_none());

        // New Pop spawned at location
        let pop_count = world.query::<&crate::layer1::pop::Pop>().iter(&world).count();
        assert_eq!(pop_count, 1);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Components

```rust
// src/layer1/stowaway.rs

use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Stowaway {
    pub stealth: f32, // 0.0 to 1.0. 0.0 = Revealed.
    pub hunger: f32,
    pub last_theft_tick: u64,
}

impl Default for Stowaway {
    fn default() -> Self {
        Self {
            stealth: 1.0,
            hunger: 0.0,
            last_theft_tick: 0,
        }
    }
}

#[derive(Component)]
pub struct InfiltrationRisk {
    pub chance: f32, // Probability per tick to infiltrate
}
```

### 2. Infiltration System

```rust
use crate::layer1::visitor::Visitor;
use crate::layer1::building::{Building, BuildingType};
use crate::layer1::map::GridPosition;
use rand::Rng;

pub fn infiltration_system(
    mut commands: Commands,
    visitors: Query<(Entity, &GridPosition, &InfiltrationRisk), With<Visitor>>,
    buildings: Query<(Entity, &GridPosition, &Building)>,
) {
    let mut rng = rand::thread_rng();

    for (visitor_entity, visitor_pos, risk) in visitors.iter() {
        if rng.gen::<f32>() < risk.chance {
            // Find nearby suitable building (Stockpile)
            for (building_entity, building_pos, building) in buildings.iter() {
                if building.building_type == BuildingType::Stockpile && visitor_pos == building_pos {
                    // Infiltrate!
                    commands.entity(visitor_entity).despawn(); // Visitor "disappears"
                    commands.entity(building_entity).insert(Stowaway::default());
                    // Log event: "A visitor has vanished..."
                    break;
                }
            }
        }
    }
}
```

### 3. Theft System

```rust
use crate::layer1::resources::ColonyResources;
use crate::shared::time::SimulationTime;

pub fn theft_system(
    mut resources: ResMut<ColonyResources>,
    time: Res<SimulationTime>,
    mut query: Query<&mut Stowaway>,
) {
    for mut stowaway in query.iter_mut() {
        if time.tick > stowaway.last_theft_tick + 100 && stowaway.hunger > 10.0 {
            // Steal food
            let stolen = 5.0f32.min(resources.food);
            resources.food -= stolen;
            stowaway.hunger -= stolen;
            stowaway.last_theft_tick = time.tick;

            // Notification: "Food is missing from the stockpile!"
        }
        stowaway.hunger += 0.1; // Passive hunger
    }
}
```

### 4. Discovery System

```rust
use crate::layer1::pop::{Pop, Name};

pub fn discovery_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Stowaway, &GridPosition)>,
) {
    for (entity, mut stowaway, pos) in query.iter_mut() {
        // Decrease stealth randomly or if workers present
        stowaway.stealth -= 0.01;

        if stowaway.stealth <= 0.0 {
            // Reveal!
            commands.entity(entity).remove::<Stowaway>();

            // Spawn new Pop
            // Determine type: Refugee (joins) or Thief (flees/arrested)
            // For MVP: Just a Refugee
            commands.spawn((
                Pop::default(),
                *pos,
                Name::new("Stowaway"),
                // Add specific "Refugee" trait/component
            ));

            // Log event: "A stowaway was found hiding in the stockpile!"
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Stealth Mechanics**: Stealth should decrease faster if `ActionType::Haul` is performed at the building (workers present).
- **Infiltration Logic**: Only allow infiltration if the Visitor is *alone* or at night? Or make `InfiltrationRisk` a component added to suspicious visitors at spawn.
- **UI**: Add a subtle visual cue to the building (e.g., "Rustling" particle effect) when stealth is low.
- **Balance**: Theft rate shouldn't starve the colony instantly. 5.0 food is a meal.

## Acceptance Criteria

- [ ] `Stowaway` component defined.
- [ ] Visitors can infiltrate Stockpiles, removing the Visitor entity and adding `Stowaway` to the building.
- [ ] Stowaways consume global Food resources over time.
- [ ] Stowaways are revealed when stealth hits 0, spawning a new Pop.
- [ ] Tests pass.

## Technical Guidance

- **Visitor Integration**: Update `spawn_visitor_system` (from `074`) to occasionally add `InfiltrationRisk` to new visitors.
- **Event Log**: Use `NarrativeGenerator` or `Notifications` to hint at the stowaway ("Strange noises heard near the stockpile...").
- **Pop Spawning**: Use the standard `spawn_pop` helper if available, or manually construct the bundle.

## Questions

- Should Stowaways be able to kill workers? (No, MVP is theft/resource drain).
- Can players manually search buildings? (Yes, add a `Search` action later).
