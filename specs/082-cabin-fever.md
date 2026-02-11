# 082: Cabin Fever

## Overview

Simulates the psychological toll of confinement and overcrowding. Pops who spend too long indoors without seeing the sky or who are constantly surrounded by others will accumulate "Cabin Fever" stress. This reduces Morale and can trigger aggressive mental breaks. Players must balance safety (thick walls, deep bunkers) with psychological health (courtyards, private rooms).

## Dependencies

- `004` — Pop Entity
- `005` — Pop Needs (Morale)
- `071` — Structural Integrity (RoofGrid)
- `050` — Civil Unrest (MentalBreakType)
- `064` — Room Quality (Mitigation)

## RED Phase: Tests First

Write these tests in `src/layer1/cabin_fever_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::{Pop, MentalState, MentalBreakType};
    use crate::layer1::needs::Needs;
    use crate::layer1::map::GridPosition;
    use crate::layer1::structural_integrity::RoofGrid;
    use crate::layer1::cabin_fever::{CabinFever, update_cabin_fever_system, apply_cabin_fever_morale_system};

    fn setup_world() -> World {
        let mut world = World::new();
        crate::setup::init_task_pools();
        world.insert_resource(crate::layer1::utility_ai::UtilityConfig::default());
        world.insert_resource(crate::shared::time::SimulationTime::default());
        // Setup 10x10 map
        let mut roof = RoofGrid::new(10, 10);
        // (5,5) is indoors
        roof.set(5, 5, true);
        world.insert_resource(roof);
        world
    }

    #[test]
    fn test_confinement_accumulation() {
        let mut world = setup_world();

        let pop = world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 }, // Indoors
            CabinFever::default(),
        )).id();

        // Run system for 10 ticks
        for _ in 0..10 {
            update_cabin_fever_system(&mut world);
        }

        let fever = world.get::<CabinFever>(pop).unwrap();
        assert!(fever.confinement > 0.0);
    }

    #[test]
    fn test_confinement_relief() {
        let mut world = setup_world();

        let pop = world.spawn((
            Pop,
            GridPosition { x: 0, y: 0 }, // Outdoors (no roof set)
            CabinFever {
                confinement: 50.0,
                ..Default::default()
            },
        )).id();

        update_cabin_fever_system(&mut world);

        let fever = world.get::<CabinFever>(pop).unwrap();
        assert!(fever.confinement < 50.0);
    }

    #[test]
    fn test_crowding_accumulation() {
        let mut world = setup_world();

        // Spawn 5 pops at (5,5)
        let pops: Vec<Entity> = (0..5).map(|_| {
            world.spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                CabinFever::default(),
            )).id()
        }).collect();

        // Run system
        // Note: Crowding check might need spatial index in real impl, or simple O(N^2) for tests
        update_cabin_fever_system(&mut world);

        let fever = world.get::<CabinFever>(pops[0]).unwrap();
        assert!(fever.crowding > 0.0);
    }

    #[test]
    fn test_cabin_fever_affects_morale() {
        let mut world = setup_world();

        let pop = world.spawn((
            Pop,
            Needs {
                leisure: 1.0, // Max morale initially
                ..Default::default()
            },
            CabinFever {
                confinement: 100.0, // High fever
                crowding: 0.0,
            },
        )).id();

        apply_cabin_fever_morale_system(&mut world);

        let needs = world.get::<Needs>(pop).unwrap();
        assert!(needs.leisure < 1.0);
    }

    #[test]
    fn test_high_fever_triggers_break() {
        let mut world = setup_world();

        let pop = world.spawn((
            Pop,
            Needs { leisure: 0.0, ..Default::default() }, // Already low
            MentalState::Normal,
            CabinFever {
                confinement: 100.0, // Maxed out
                crowding: 100.0,
            },
        )).id();

        // Run break check (from existing system, but enhanced)
        // Check if logic handles new break type or generic break
        crate::layer1::unrest::check_mental_break_system(&mut world);

        let state = world.get::<MentalState>(pop).unwrap();
        assert!(matches!(state, MentalState::Broken(_)));
        // Ideally specifically Aggressive or similar, but generic Broken is MVP
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Component

```rust
// src/layer1/cabin_fever.rs

use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::structural_integrity::RoofGrid;
use crate::layer1::needs::Needs;

#[derive(Component, Debug, Clone, Default)]
pub struct CabinFever {
    pub confinement: f32, // 0.0 to 100.0
    pub crowding: f32,    // 0.0 to 100.0
}

impl CabinFever {
    pub fn total_stress(&self) -> f32 {
        (self.confinement + self.crowding).min(100.0)
    }
}
```

### 2. Implement Systems

```rust
// src/layer1/cabin_fever.rs

pub fn update_cabin_fever_system(
    roof: Res<RoofGrid>,
    mut query: Query<(Entity, &GridPosition, &mut CabinFever)>,
    all_pops: Query<&GridPosition, With<crate::layer1::pop::Pop>>,
) {
    // 1. Confinement Logic
    for (entity, pos, mut fever) in &mut query {
        if roof.has_roof(pos.x, pos.y) {
            fever.confinement = (fever.confinement + 0.1).min(100.0);
        } else {
            fever.confinement = (fever.confinement - 0.5).max(0.0);
        }

        // 2. Crowding Logic (Naive O(N*M) for MVP Green)
        let mut neighbors = 0;
        for other_pos in &all_pops {
            if (other_pos.x - pos.x).abs() <= 1 && (other_pos.y - pos.y).abs() <= 1 {
                neighbors += 1;
            }
        }

        // neighbors includes self, so > 3 means 2 others adjacent
        if neighbors > 3 {
             fever.crowding = (fever.crowding + 0.2).min(100.0);
        } else {
             fever.crowding = (fever.crowding - 0.1).max(0.0);
        }
    }
}

pub fn apply_cabin_fever_morale_system(
    mut query: Query<(&CabinFever, &mut Needs)>,
) {
    for (fever, mut needs) in &mut query {
        let penalty = fever.total_stress() * 0.001; // Scale down
        needs.leisure = (needs.leisure - penalty).max(0.0);
    }
}
```

### 3. Register Systems

Add `update_cabin_fever_system` and `apply_cabin_fever_morale_system` to `src/simulation.rs` in `ColonyUpdate` schedule.

## REFACTOR Phase: Quality & Design

- **Spatial Index**: The O(N^2) crowding check is bad. Use a `GridMap<Entity>` or `KdTree` to query neighbors efficiently.
- **Room Quality**: High quality rooms (Decor > 50) should negate confinement gain.
- **UI**: Display "Cabin Fever" in Pop Inspection UI.
- **Traits**: Add `Claustrophobic` (faster gain) and `Agoraphobic` (gain outdoors instead) traits.
- **Visuals**: Pops with high cabin fever could show a "sweat" or "angry" icon.

## Acceptance Criteria

- [ ] `CabinFever` component exists.
- [ ] Pops indoors accumulate confinement stress.
- [ ] Pops outdoors reduce confinement stress.
- [ ] Crowded pops accumulate crowding stress.
- [ ] High stress reduces Morale (Leisure).
- [ ] Tests pass.

## Technical Guidance

- Use `RoofGrid` from Spec 071.
- Ensure systems run frequently enough (every tick or every X ticks).
- Confinement gain should be slow (days to max out), relief should be faster (hours).
