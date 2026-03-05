# 083: Building Shifts

## 1. Overview

Colony productivity is limited by the circadian rhythm of its inhabitants. Currently, buildings operate whenever a worker is available. To allow for 24-hour production cycles and specialized scheduling, players need the ability to designate "Day Shifts" and "Night Shifts" for specific buildings.

This spec introduces the `ShiftSchedule` component, which controls when a building offers work tasks. It integrates with the `DayNightCycle` to filter availability.

## 2. Dependencies

- `specs/007-building-housing.md` (Building entities)
- `specs/016-utility-ai-system.md` (Work evaluation)
- `specs/065-day-night-cycle.md` (Time of day)

## 3. RED Phase: Tests First

```rust
// Define the tests that will drive implementation
// These should FAIL initially

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::ShiftSchedule;
    use crate::layer1::day_night::TimeOfDay;

    #[test]
    fn test_shift_schedule_default() {
        // Default behavior: Day shift enabled, Night shift disabled
        let schedule = ShiftSchedule::default();
        assert!(schedule.day_shift, "Day shift should be enabled by default");
        assert!(!schedule.night_shift, "Night shift should be disabled by default");
    }

    #[test]
    fn test_shift_active_during_day() {
        let schedule = ShiftSchedule { day_shift: true, night_shift: false };

        // Day shifts cover Dawn, Day, and Dusk
        assert!(schedule.is_active(TimeOfDay::Dawn), "Should be active at Dawn");
        assert!(schedule.is_active(TimeOfDay::Day), "Should be active at Day");
        assert!(schedule.is_active(TimeOfDay::Dusk), "Should be active at Dusk");
        assert!(!schedule.is_active(TimeOfDay::Night), "Should NOT be active at Night");
    }

    #[test]
    fn test_shift_active_during_night() {
        let schedule = ShiftSchedule { day_shift: false, night_shift: true };

        assert!(!schedule.is_active(TimeOfDay::Dawn), "Should NOT be active at Dawn");
        assert!(!schedule.is_active(TimeOfDay::Day), "Should NOT be active at Day");
        assert!(!schedule.is_active(TimeOfDay::Dusk), "Should NOT be active at Dusk");
        assert!(schedule.is_active(TimeOfDay::Night), "Should be active at Night");
    }

    #[test]
    fn test_shift_active_always() {
        let schedule = ShiftSchedule { day_shift: true, night_shift: true };

        assert!(schedule.is_active(TimeOfDay::Dawn));
        assert!(schedule.is_active(TimeOfDay::Day));
        assert!(schedule.is_active(TimeOfDay::Dusk));
        assert!(schedule.is_active(TimeOfDay::Night));
    }
}

// In integration tests for actions (e.g., src/layer1/actions/refine_tests.rs)

#[test]
fn test_evaluate_refine_respects_shifts() {
    let mut world = World::new();
    // ... setup world with resources, time, building ...

    // Set Time to Night
    {
        let mut cycle = world.resource_mut::<DayNightCycle>();
        cycle.time_of_day = TimeOfDay::Night;
    }

    // Set Building to Day Shift Only
    {
        let mut schedule = world.get_mut::<ShiftSchedule>(building_entity).unwrap();
        schedule.day_shift = true;
        schedule.night_shift = false;
    }

    // Evaluate Refine -> Should be None
    let result = evaluate_refine(&pop_pos, &weights, &resources, buildings_iter);
    assert!(result.is_none(), "Should not work at night if night shift is disabled");

    // Enable Night Shift
    {
        let mut schedule = world.get_mut::<ShiftSchedule>(building_entity).unwrap();
        schedule.night_shift = true;
    }

    // Evaluate Refine -> Should be Some
    let result = evaluate_refine(&pop_pos, &weights, &resources, buildings_iter);
    assert!(result.is_some(), "Should work at night if night shift is enabled");
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/building.rs

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

impl ShiftSchedule {
    pub fn is_active(&self, time: crate::layer1::day_night::TimeOfDay) -> bool {
        use crate::layer1::day_night::TimeOfDay;
        match time {
            TimeOfDay::Night => self.night_shift,
            _ => self.day_shift,
        }
    }
}

// Update spawn_building to add ShiftSchedule to relevant buildings
// (LumberMill, StoneMason, Smelter, Smithy, Library, Farm, Plantation, Weaver, Tailor, Hospital)

// src/layer1/actions/refine.rs (and farm.rs, research.rs)

pub fn evaluate_refine<'a>(
    // ... existing args ...
    cycle: &DayNightCycle, // Add this argument
    buildings: impl Iterator<Item = (Entity, &'a GridPosition, &'a Building, &'a RefiningProgress, Option<&'a ShiftSchedule>)>,
) -> Option<(f32, Entity)> {

    for (entity, pos, building, progress, schedule) in buildings {
        // ... existing recipe checks ...

        // Check Shift Schedule
        if let Some(schedule) = schedule {
            if !schedule.is_active(cycle.time_of_day) {
                continue;
            }
        }

        // ... continue with scoring ...
    }
    // ...
}
```

## 5. REFACTOR Phase: Quality & Design

-   **Mood Penalties:** Working the Night Shift should ideally apply a mood penalty (e.g., "Tired (Night Shift)") unless the pop has the `Nocturnal` trait. This can be added to the `work_complete` or `refine_complete` systems later.
-   **UI Integration:** The player needs a UI to toggle these shifts. This is outside the scope of Layer 1 logic but critical for the feature's utility.
-   **Performance:** The `is_active` check is very cheap, so it shouldn't impact performance significantly. However, ensuring we don't query `ShiftSchedule` where not needed (e.g., Housing) is good practice.

## 6. Acceptance Criteria (Testable!)

- [ ] `ShiftSchedule` component exists and implements `Default`.
- [ ] `spawn_building` adds `ShiftSchedule` to production buildings.
- [ ] `evaluate_refine`, `evaluate_farm`, and `evaluate_research` take `DayNightCycle` and `ShiftSchedule` into account.
- [ ] Unit tests for `ShiftSchedule` pass.
- [ ] Integration tests verify that work is unavailable during closed shifts.
- [ ] `cargo test` passes.
- [ ] `cargo clippy` passes.

## 7. Technical Guidance

-   **Component Query:** When updating the `evaluate_*` functions, remember that `ShiftSchedule` might be optional (e.g., for modded buildings or legacy saves), so use `Option<&ShiftSchedule>` in the iterator or query.
-   **System Arguments:** You will need to pass the `DayNightCycle` resource into the systems that call `evaluate_*`.
-   **Default State:** Ensure that `ShiftSchedule::default()` allows Day shifts so that existing behavior (working during the day) is preserved without manual intervention.

## 8. Questions

- *Builder: Should `Dawn` and `Dusk` be configurable separately?*
  *Architect:* No, group them together as `Day` for the MVP to reduce UI complexity.
- *Builder: Does this apply to Construction tasks?*
  *Architect:* No, construction is an on-demand designation and bypasses standard building shifts.
