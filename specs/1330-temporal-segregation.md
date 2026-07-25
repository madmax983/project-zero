# 1330: Temporal Segregation

## 1. Overview
Watching a colony divide not by ideology, but by the rhythm of their waking hours, creating two separate societies that share the same physical space. Pops develop distinct circadian rhythms based on their job types and environmental conditions. Over time, distinct "Day" and "Night" shifts start forming their own micro-cultures, loyalties, and needs, refusing to interact with each other.

## 2. Dependencies
- `016` — Utility AI (Action evaluation)
- `005` — Pop Needs

## 3. RED Phase: Tests First

```rust
// src/layer1/temporal_segregation_tests.rs

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::temporal_segregation::{CircadianRhythm, Shift, temporal_drift_system, shift_conflict_system, ShiftFriction};
    use crate::layer1::needs::Needs;
    use crate::layer1::day_night::{DayNightCycle, TimeOfDay};

    #[test]
    fn test_circadian_rhythm_drift() {
        // Arrange: Setup test data
        let mut world = World::new();
        let pop = world.spawn((
            Pop,
            CircadianRhythm { current_shift: Shift::Day, shift_affinity: 0.0 },
            Needs { rest: 0.0, ..Default::default() } // Exhausted pop
        )).id();

        // Act: Call the feature simulating night work
        // Pop works at night, driving affinity towards Night shift
        world.insert_resource(DayNightCycle { time_of_day: TimeOfDay::Night, ..Default::default() });

        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(temporal_drift_system);
        schedule.run(&mut world);

        // Assert: Verify expected behavior
        let rhythm = world.get::<CircadianRhythm>(pop).unwrap();
        assert!(rhythm.shift_affinity > 0.0, "Affinity should drift towards Night when working at night");
    }

    #[test]
    fn test_shift_conflict_penalty() {
        // Test boundary conditions
        let mut world = World::new();
        let day_pop = world.spawn((Pop, CircadianRhythm { current_shift: Shift::Day, shift_affinity: -1.0 })).id();
        let night_pop = world.spawn((Pop, CircadianRhythm { current_shift: Shift::Night, shift_affinity: 1.0 })).id();

        // When day_pop and night_pop interact, there should be a friction/stress penalty
        world.insert_resource(ShiftFriction { active_friction: false });

        let mut schedule = bevy_ecs::schedule::Schedule::default();
        schedule.add_systems(shift_conflict_system);
        schedule.run(&mut world);

        // Assert that a conflict has been registered
        let friction = world.get_resource::<ShiftFriction>().unwrap();
        assert!(friction.active_friction, "Friction should be active when both shifts exist");
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/temporal_segregation.rs
use bevy_ecs::prelude::*;
use crate::layer1::pop::Pop;
use crate::layer1::needs::Needs;
use crate::layer1::day_night::{DayNightCycle, TimeOfDay};

#[derive(Component, Default, PartialEq, Eq, Clone, Copy)]
pub enum Shift {
    #[default]
    Day,
    Night,
}

#[derive(Component, Default, Clone, Copy)]
pub struct CircadianRhythm {
    pub current_shift: Shift,
    pub shift_affinity: f32, // -1.0 (Day) to 1.0 (Night)
}

#[derive(Resource, Default)]
pub struct ShiftFriction {
    pub active_friction: bool,
}

pub fn temporal_drift_system(mut query: Query<(&Needs, &mut CircadianRhythm), With<Pop>>, cycle: Res<DayNightCycle>) {
    // The SIMPLEST code that makes tests pass
    for (needs, mut rhythm) in query.iter_mut() {
        if cycle.time_of_day == TimeOfDay::Night {
            rhythm.shift_affinity += 0.1;
        } else if cycle.time_of_day == TimeOfDay::Day {
            rhythm.shift_affinity -= 0.1;
        }
        rhythm.shift_affinity = rhythm.shift_affinity.clamp(-1.0, 1.0);

        if rhythm.shift_affinity > 0.5 {
            rhythm.current_shift = Shift::Night;
        } else if rhythm.shift_affinity < -0.5 {
            rhythm.current_shift = Shift::Day;
        }
    }
}

pub fn shift_conflict_system(query: Query<&CircadianRhythm, With<Pop>>, mut friction: ResMut<ShiftFriction>) {
    // Minimal implementation: if both shifts exist, apply some global friction.
    let mut has_day = false;
    let mut has_night = false;

    for rhythm in query.iter() {
        match rhythm.current_shift {
            Shift::Day => has_day = true,
            Shift::Night => has_night = true,
        }
    }

    if has_day && has_night {
        friction.active_friction = true;
    } else {
        friction.active_friction = false;
    }
}
```

## 5. REFACTOR Phase: Quality & Design
- **Quality**: Ensure `shift_affinity` drift only occurs when a pop is actually awake/working, rather than just passively existing. Hook into the execution system to check if they are working.
- **Performance**: Use Bevy events for social interactions to check for shift conflicts on a case-by-case basis instead of a global O(N^2) loop or simple global flag.
- **Design**: Integrate with the `UtilityAI` so that Night-shift pops naturally prefer to sleep during the day and work at night.
- **API Improvements**: Define clear thresholds as constants for shift transitions.

## 6. Acceptance Criteria (Testable!)
- [ ] All tests in RED phase pass
- [ ] `cargo test` returns 0 failures
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Test coverage ≥85% for new code
- [ ] Pops develop circadian rhythms based on when they work
- [ ] Interactions between opposing shifts create friction/stress

## 7. Technical Guidance
- **Code structure suggestions**: Create a new module `src/layer1/temporal_segregation.rs`
- **Integration points**: Hook into `evaluate_actions_system` to modify `ActionType::Sleep` scoring based on `CircadianRhythm`.
- **Gotchas and common mistakes**: Do not let `shift_affinity` drift indefinitely; always clamp it. Ensure new Pops spawn with a neutral or default rhythm.

## 8. Questions
*Builder: add questions here if spec is unclear.*
