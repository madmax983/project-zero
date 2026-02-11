# 083: Building Shifts

## Overview

Players can configure the operating hours of buildings to manage workforce efficiency and adapt to the day/night cycle. By default, most buildings operate during the Day. Enabling "Night Shift" allows 24/7 production but requires a workforce willing (or forced) to work at night.

This adds a layer of management:
- **Efficiency**: Run factories 24/7 to maximize output.
- **Power**: Turn off power-hungry buildings at night to save batteries.
- **Safety**: Restrict outdoor work (Farms) to daylight to avoid nocturnal predators (future).

## Dependencies

- `016` — Utility AI System (for `evaluate_actions_system`)
- `065` — Day/Night Cycle (for `DayNightCycle`, `TimeOfDay`)
- `066` — Building Work AI (for `evaluate_refine`, `evaluate_farm`)

## RED Phase: Tests First

Write these tests in `src/layer1/shifts_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::day_night::{DayNightCycle, TimeOfDay};
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::shifts::{ShiftSchedule, is_building_open};
    use crate::layer1::map::GridPosition;
    use crate::layer1::utility_ai::{UtilityWeights, UtilityConfig};
    use crate::layer1::resources::ColonyResources;

    // We need to test integration with evaluate_refine.
    // Assuming evaluate_refine is public or we can access it via a helper.
    use crate::layer1::actions::refine::evaluate_refine;

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(DayNightCycle::default());
        world.insert_resource(UtilityConfig::default());
        world.insert_resource(ColonyResources::default());
        world
    }

    #[test]
    fn test_shift_schedule_default() {
        // Default should be Day only (06:00 - 18:00 approx)
        let schedule = ShiftSchedule::default();
        assert!(schedule.day_shift);
        assert!(!schedule.night_shift);
    }

    #[test]
    fn test_is_building_open_day() {
        let schedule = ShiftSchedule { day_shift: true, night_shift: false };

        // Dawn is considered Day shift
        assert!(is_building_open(&schedule, TimeOfDay::Dawn));
        // Day is Day shift
        assert!(is_building_open(&schedule, TimeOfDay::Day));
        // Dusk is Day shift
        assert!(is_building_open(&schedule, TimeOfDay::Dusk));
    }

    #[test]
    fn test_is_building_open_night() {
        let schedule = ShiftSchedule { day_shift: true, night_shift: false };
        assert!(!is_building_open(&schedule, TimeOfDay::Night));

        let night_schedule = ShiftSchedule { day_shift: false, night_shift: true };
        assert!(is_building_open(&night_schedule, TimeOfDay::Night));
        assert!(!is_building_open(&night_schedule, TimeOfDay::Day));
    }

    #[test]
    fn test_evaluate_refine_respects_closed_status() {
        let mut world = setup_world();

        // Set time to NIGHT
        world.resource_mut::<DayNightCycle>().time_of_day = TimeOfDay::Night;
        // Give resources so affordability isn't the blocker
        world.resource_mut::<ColonyResources>().wood = 100.0;

        // Spawn Lumber Mill with Default Schedule (Day Only)
        let mill = world.spawn((
            Building { building_type: BuildingType::LumberMill },
            GridPosition { x: 0, y: 0 },
            ShiftSchedule::default(), // Day only
            crate::layer1::resources::RefiningProgress::default(),
        )).id();

        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default();
        let resources = world.resource::<ColonyResources>();
        let cycle = world.resource::<DayNightCycle>();

        // Query needs to include ShiftSchedule
        // We'll need to modify evaluate_refine signature or query to include it.
        // For this test, we construct the input iterator manually if evaluate_refine accepts it,
        // OR we rely on the system test pattern.

        // Let's assume we update evaluate_refine to take an iterator including ShiftSchedule.
        let buildings = world.query::<(Entity, &GridPosition, &Building, &crate::layer1::resources::RefiningProgress, &ShiftSchedule)>();

        // This won't compile until we update evaluate_refine signature.
        // But conceptually:
        let result = evaluate_refine(
            &pop_pos,
            &weights,
            resources,
            cycle, // New argument!
            buildings.iter(&world)
        );

        assert!(result.is_none(), "Should not evaluate work for closed building");
    }

    #[test]
    fn test_evaluate_refine_works_when_open() {
        let mut world = setup_world();
        world.resource_mut::<DayNightCycle>().time_of_day = TimeOfDay::Day;
        world.resource_mut::<ColonyResources>().wood = 100.0;

        let mill = world.spawn((
            Building { building_type: BuildingType::LumberMill },
            GridPosition { x: 0, y: 0 },
            ShiftSchedule::default(),
            crate::layer1::resources::RefiningProgress::default(),
        )).id();

        let pop_pos = GridPosition { x: 0, y: 0 };
        let weights = UtilityWeights::default();
        let resources = world.resource::<ColonyResources>();
        let cycle = world.resource::<DayNightCycle>();
        let buildings = world.query::<(Entity, &GridPosition, &Building, &crate::layer1::resources::RefiningProgress, &ShiftSchedule)>();

        let result = evaluate_refine(
            &pop_pos,
            &weights,
            resources,
            cycle,
            buildings.iter(&world)
        );

        assert!(result.is_some(), "Should find work when open");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Create `ShiftSchedule` Component

Create `src/layer1/shifts.rs`.

```rust
use bevy_ecs::prelude::*;
use crate::layer1::day_night::TimeOfDay;

#[derive(Component, Debug, Clone, Copy)]
pub struct ShiftSchedule {
    pub day_shift: bool,
    pub night_shift: bool,
}

impl Default for ShiftSchedule {
    fn default() -> Self {
        Self {
            day_shift: true,
            night_shift: false,
        }
    }
}

/// Helper to check if building is open
#[must_use]
pub fn is_building_open(schedule: &ShiftSchedule, time: TimeOfDay) -> bool {
    match time {
        TimeOfDay::Dawn | TimeOfDay::Day | TimeOfDay::Dusk => schedule.day_shift,
        TimeOfDay::Night => schedule.night_shift,
    }
}
```

### 2. Update `evaluate_refine`

Modify `src/layer1/actions/refine.rs`.

```rust
use crate::layer1::shifts::{ShiftSchedule, is_building_open};
use crate::layer1::day_night::DayNightCycle;

// Update signature
pub fn evaluate_refine<'a>(
    pop_pos: &GridPosition,
    weights: &UtilityWeights,
    resources: &ColonyResources,
    cycle: &DayNightCycle, // Added
    buildings: impl Iterator<Item = (Entity, &'a GridPosition, &'a Building, &'a RefiningProgress, Option<&'a ShiftSchedule>)>, // Added Option<ShiftSchedule>
) -> Option<(f32, Entity)> {

    // ... inside loop ...
    for (entity, pos, building, progress, schedule_opt) in buildings {
        // Check shifts
        if let Some(schedule) = schedule_opt {
            if !is_building_open(schedule, cycle.time_of_day) {
                continue;
            }
        }

        // ... rest of logic ...
    }
}
```

### 3. Update `evaluate_farm`

Modify `src/layer1/actions/farm.rs` similarly to accept `DayNightCycle` and check `ShiftSchedule`.

### 4. Update Main System

Update `evaluate_actions_system` in `src/layer1/utility_ai.rs` to fetch `DayNightCycle` and pass it to evaluation functions.

### 5. Add Component to Buildings

In `src/layer1/building.rs`, ensure newly placed buildings (that accept workers) get `ShiftSchedule::default()`.
- Add to `spawn_building`.

## REFACTOR Phase: Quality & Design

- **UI Integration**:
    - Add buttons to `InspectionWindow` (Spec 015) to toggle `day_shift` / `night_shift`.
    - Display "CLOSED" status in UI if selected building is closed.
- **Visuals**:
    - Dim lights of closed buildings (Spec 053 Integration).
- **Optimization**:
    - Evaluate shifts before evaluating pathfinding/utility to save cycles.

## Acceptance Criteria

- [ ] `ShiftSchedule` component exists.
- [ ] Buildings default to Day Shift only.
- [ ] `evaluate_refine` returns `None` for Night Shift if disabled.
- [ ] `evaluate_farm` returns `None` for Night Shift if disabled.
- [ ] Tests pass.
- [ ] System integrates with `DayNightCycle`.

## Technical Guidance

- Be careful with `Option<&ShiftSchedule>`. Some buildings (like Storage) might not have shifts. Only worker-buildings need it. If missing, assume Open (or Closed? Assume Open 24/7 is safer for passive buildings, but for Work buildings, they should have the component).
- `evaluate_refine` iterates buildings. Make sure to update the `Query` in `evaluate_actions_system` to include `Option<&ShiftSchedule>`.
