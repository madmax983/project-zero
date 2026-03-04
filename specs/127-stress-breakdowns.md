# 127: Stress Breakdowns

## Overview

When a Pop's Morale stays critically low (< 0.15) for an extended period, they suffer a **Mental Breakdown**. This overrides their normal AI behavior with a disruptive state determined by their Traits (e.g., Pyromaniac -> Fire Starting, Glutton -> Binge Eating, Default -> Dazing). The breakdown lasts for a duration, after which the Pop gains a temporary **Catharsis** buff (high morale) to prevent death spirals.

## Dependencies

- `031` — Pop Morale (Input signal)
- `084` — Pop Traits (Determines breakdown type)
- `033` — Fire Propagation (For Pyromania effect)
- `005` — Pop Needs (For Binge Eating effect)

## RED Phase: Tests First

Write these tests in `src/layer1/stress_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::Pop;
    use crate::layer1::needs::Needs;
    use crate::layer1::traits::{Trait, Traits};
    use crate::layer1::stress::{StressTracker, Breakdown, BreakdownType, Catharsis, check_stress_breakdown_system};

    #[test]
    fn test_stress_accumulation() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(check_stress_breakdown_system);

        // Pop with very low morale (0.05)
        let pop = world.spawn((
            Pop::default(),
            Needs { hunger: 0.05, rest: 0.05, leisure: 0.05 }, // Morale = 0.05
            StressTracker::default(),
            Traits(std::collections::HashSet::new()),
        )).id();

        // Run schedule 10 times
        for _ in 0..10 {
            schedule.run(&mut world);
        }

        let tracker = world.get::<StressTracker>(pop).unwrap();
        assert!(tracker.ticks_at_low_morale > 0, "Should accumulate stress ticks");
    }

    #[test]
    fn test_breakdown_trigger() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(check_stress_breakdown_system);

        // Pop with maxed out stress ticks
        let pop = world.spawn((
            Pop::default(),
            Needs { hunger: 0.0, rest: 0.0, leisure: 0.0 },
            StressTracker { ticks_at_low_morale: 1000 }, // Assume threshold is < 1000
            Traits(std::collections::HashSet::new()),
        )).id();

        schedule.run(&mut world);

        // Should have Breakdown component
        assert!(world.get::<Breakdown>(pop).is_some());
        // Should reset tracker
        assert_eq!(world.get::<StressTracker>(pop).unwrap().ticks_at_low_morale, 0);
    }

    #[test]
    fn test_breakdown_type_pyromaniac() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(check_stress_breakdown_system);

        // We assume 'Pyromaniac' is added to Trait enum during implementation
        let mut traits = std::collections::HashSet::new();
        traits.insert(Trait::Pyromaniac);

        let pop = world.spawn((
            Pop::default(),
            Needs { hunger: 0.0, rest: 0.0, leisure: 0.0 },
            StressTracker { ticks_at_low_morale: 1000 },
            Traits(traits),
        )).id();

        schedule.run(&mut world);

        let breakdown = world.get::<Breakdown>(pop).unwrap();
        assert_eq!(breakdown.breakdown_type, BreakdownType::FireStarting);
    }

    #[test]
    fn test_breakdown_type_default() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(check_stress_breakdown_system);

        let pop = world.spawn((
            Pop::default(),
            Needs { hunger: 0.0, rest: 0.0, leisure: 0.0 },
            StressTracker { ticks_at_low_morale: 1000 },
            Traits(std::collections::HashSet::new()), // No traits
        )).id();

        schedule.run(&mut world);

        let breakdown = world.get::<Breakdown>(pop).unwrap();
        assert_eq!(breakdown.breakdown_type, BreakdownType::Dazing);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Data Structures (`src/layer1/stress.rs`)

```rust
use bevy_ecs::prelude::*;

#[derive(Component, Default, Debug)]
pub struct StressTracker {
    pub ticks_at_low_morale: u32,
}

#[derive(Component, Debug)]
pub struct Breakdown {
    pub breakdown_type: BreakdownType,
    pub duration_remaining: u32,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BreakdownType {
    Dazing,       // Wanders aimlessly
    FireStarting, // Starts fires (Pyromaniac)
    BingeEating,  // Eats food (Glutton)
    HideInRoom,   // Stays in room (Melancholic/Anxious)
    SadWander,    // Wanders crying
}

#[derive(Component, Debug)]
pub struct Catharsis {
    pub duration_remaining: u32,
    pub morale_bonus: f32, // +0.5 Morale
}
```

### 2. Systems

```rust
pub const LOW_MORALE_THRESHOLD: f32 = 0.15;
pub const BREAKDOWN_TICKS_REQUIRED: u32 = 100; // ~5-10 seconds
pub const BREAKDOWN_DURATION: u32 = 500;
pub const CATHARSIS_DURATION: u32 = 2000;

pub fn check_stress_breakdown_system(
    mut commands: Commands,
    mut query: Query<(Entity, &Needs, &mut StressTracker, Option<&Traits>, Option<&Breakdown>, Option<&Catharsis>)>,
) {
    for (entity, needs, mut tracker, traits, breakdown, catharsis) in query.iter_mut() {
        // If already broken or has catharsis, skip stress tracking
        if breakdown.is_some() || catharsis.is_some() {
            tracker.ticks_at_low_morale = 0;
            continue;
        }

        if needs.morale() < LOW_MORALE_THRESHOLD {
            tracker.ticks_at_low_morale += 1;
        } else {
            tracker.ticks_at_low_morale = tracker.ticks_at_low_morale.saturating_sub(1);
        }

        if tracker.ticks_at_low_morale >= BREAKDOWN_TICKS_REQUIRED {
            // Trigger Breakdown
            let b_type = determine_breakdown_type(traits);
            commands.entity(entity).insert(Breakdown {
                breakdown_type: b_type,
                duration_remaining: BREAKDOWN_DURATION,
            });
            tracker.ticks_at_low_morale = 0;

            // Notification (placeholder)
            // println!("Pop {:?} suffered a mental break: {:?}", entity, b_type);
        }
    }
}

fn determine_breakdown_type(traits: Option<&Traits>) -> BreakdownType {
    if let Some(t) = traits {
        if t.0.contains(&crate::layer1::traits::Trait::Pyromaniac) {
            return BreakdownType::FireStarting;
        }
        if t.0.contains(&crate::layer1::traits::Trait::Glutton) {
            return BreakdownType::BingeEating;
        }
        // Add more trait mappings here
    }
    BreakdownType::Dazing
}

pub fn update_breakdown_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Breakdown)>,
) {
    for (entity, mut breakdown) in query.iter_mut() {
        if breakdown.duration_remaining > 0 {
            breakdown.duration_remaining -= 1;
        } else {
            // End Breakdown, Add Catharsis
            commands.entity(entity).remove::<Breakdown>();
            commands.entity(entity).insert(Catharsis {
                duration_remaining: CATHARSIS_DURATION,
                morale_bonus: 0.5,
            });
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Utility AI Integration**: The `Breakdown` component must be checked in `evaluate_actions_system` (`016`/`021`) to block normal jobs.
- **Breakdown AI**: Need specific `ActionType::Breakdown` actions (e.g., `StartFire`, `BingeEat`).
- **Catharsis Effect**: Modify `Needs::morale()` to include `Catharsis` bonus.
- **UI**: Show "Broken" state in Pop Inspector.

## Acceptance Criteria

- [ ] `StressTracker` accumulates when morale < 0.15.
- [ ] `Breakdown` triggers after threshold.
- [ ] `BreakdownType` varies based on Traits.
- [ ] `Catharsis` applies after breakdown ends.
- [ ] Tests pass.

## Technical Guidance

- Modify `src/layer1/needs.rs` to accept `Catharsis` component in morale calculation (optional, or just add a separate modifier system).
- Update `Trait` enum in `src/layer1/traits.rs` to include `Pyromaniac` if not present.
- Ensure `Breakdown` component effectively pauses normal needs decay or job assignment? No, needs should still decay (starvation risk), but jobs should be blocked.

## Questions

*Builder: add questions here if spec is unclear.*
*Architect:* See related specifications for design details. MVP implementation should follow standard conventions.
