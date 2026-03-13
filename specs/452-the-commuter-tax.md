# 452: The Commuter Tax

## Overview

"Taxing your workers for simply moving through your colony's infrastructure."

Paved roads and advanced transit lines (like pneumatic tubes or moving walkways) can have a "Toll" assigned to them. Pops who use them pay a tiny fraction of their personal Credits or Morale. High tolls speed up travel but drain citizen wealth, while free roads lead to congestion.

If a Pop cannot afford the toll (in credits or morale tolerance), they are forced to use alternative routes, potentially walking through dangerous, unpaved "Wild" zones. This introduces a tension between fast, efficient logistics funded by user fees versus accessible transit that avoids creating a two-tiered society.

## Dependencies

- `025` — Hauling Logistics / Movement
- `031` — Pop Morale
- `004` — Basic Building (for roads/infrastructure)

## RED Phase: Tests First

Write these tests in `src/layer1/infrastructure/commuter_tax_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::{Pop, Mood};
    use crate::layer1::map::GridPosition;
    use crate::layer1::infrastructure::transit::{TransitInfrastructure, Toll, Wealth, transit_toll_system};

    #[test]
    fn test_pop_pays_toll_on_transit() {
        let mut world = World::new();

        // 1. Create a road with a toll
        let road_pos = GridPosition { x: 5, y: 5 };
        world.spawn((
            TransitInfrastructure { speed_multiplier: 2.0 },
            Toll { cost: 1.0 },
            road_pos,
        ));

        // 2. Create a Pop with wealth moving onto the road
        let pop = world.spawn((
            Pop,
            road_pos, // Pop is on the road
            Wealth { credits: 10.0 },
        )).id();

        // 3. Run the toll system
        let mut schedule = Schedule::default();
        schedule.add_systems(transit_toll_system);
        schedule.run(&mut world);

        // 4. Assert Pop lost credits
        let pop_wealth = world.get::<Wealth>(pop).unwrap();
        assert_eq!(pop_wealth.credits, 9.0, "Pop should have paid 1.0 credit for the toll");
    }

    #[test]
    fn test_pop_morale_penalty_if_cannot_pay() {
        let mut world = World::new();

        // 1. Create an expensive road
        let road_pos = GridPosition { x: 5, y: 5 };
        world.spawn((
            TransitInfrastructure { speed_multiplier: 2.0 },
            Toll { cost: 5.0 },
            road_pos,
        ));

        // 2. Create a broke Pop
        let pop = world.spawn((
            Pop,
            road_pos,
            Wealth { credits: 0.0 },
            Mood { stress: 10.0, ..Default::default() },
        )).id();

        // 3. Run the toll system
        let mut schedule = Schedule::default();
        schedule.add_systems(transit_toll_system);
        schedule.run(&mut world);

        // 4. Assert Pop is broke but highly stressed
        let pop_wealth = world.get::<Wealth>(pop).unwrap();
        assert_eq!(pop_wealth.credits, 0.0);
        let pop_mood = world.get::<Mood>(pop).unwrap();
        assert!(pop_mood.stress > 10.0, "Pop should incur stress if they cannot afford the toll");
    }
}
```

## GREEN Phase: Minimal Implementation

```rust
// src/layer1/infrastructure/transit.rs

use bevy_ecs::prelude::*;
use crate::layer1::pop::Mood;
use crate::layer1::map::GridPosition;

#[derive(Component)]
pub struct TransitInfrastructure {
    pub speed_multiplier: f32,
}

#[derive(Component)]
pub struct Toll {
    pub cost: f32,
}

#[derive(Component, Default)]
pub struct Wealth {
    pub credits: f32,
}

pub fn transit_toll_system(
    mut pops: Query<(&GridPosition, &mut Wealth, &mut Mood)>,
    roads: Query<(&GridPosition, &Toll), With<TransitInfrastructure>>,
) {
    for (pop_pos, mut wealth, mut mood) in pops.iter_mut() {
        // Find if pop is on a toll road
        for (road_pos, toll) in roads.iter() {
            if pop_pos == road_pos {
                if wealth.credits >= toll.cost {
                    wealth.credits -= toll.cost;
                } else {
                    // Pop can't afford it, penalty to stress
                    mood.stress += toll.cost * 2.0;
                }
                // Break out of inner loop since pop can only be on one road tile
                break;
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Performance Optimization**: `transit_toll_system` currently performs a naive nested O(N*M) iteration over all Pops and all Roads. Replace the inner loop with a spatial index lookup (like a `HashMap<GridPosition, Entity>`) for O(1) road lookup per Pop.
- **Pathfinding Integration**: The pathfinding algorithm (`A*`) must consider Toll costs as part of the traversal weight. Poor pops should naturally path around high-toll roads, creating desire paths in the wilderness.
- **Continuous vs Step tolling**: Charging per tile every tick is too aggressive. Consider tracking when a Pop enters and exits a transit network, charging a flat rate, or applying the toll once per movement step rather than every simulation tick.

## Acceptance Criteria
- [ ] `Toll` and `Wealth` components implemented.
- [ ] Pops on toll tiles correctly lose `Wealth` or gain `Stress`.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `commuter_tax` logic.

## Technical Guidance
- **Movement Tracking**: To prevent a Pop from being charged 60 times a second while traversing a single tile, tie the `transit_toll_system` to a specific `MovementStepEvent` or `TileEnterEvent` rather than running it universally every `Update` tick.
- **Wealth Initialization**: Ensure new Pops spawn with a default `Wealth` component.

## Questions
*Builder: add questions here if spec is unclear. Architect will address.*
