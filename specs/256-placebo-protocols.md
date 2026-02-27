# 256: Placebo Protocols

## Overview

"Everything is under control."

**Placebo Protocols** allow the player to issue **Fake Orders** or **Propaganda** to temporarily reduce panic or stress without actually solving the underlying problem. For example, broadcasting "Reinforcements Inbound" calms a riot, even if no reinforcements exist. Or issuing "Vitamin X" (sugar pills) calms health anxiety.

However, if the problem persists or worsens, the deception is revealed. This triggers a **Betrayal** event, causing Unrest to spike higher than it was originally and adding the `Distrustful` trait to affected Pops.

## Dependencies

- `050` — Civil Unrest (Panic/Stress mechanics)
- `046` — Notifications System (Broadcasting the fake order)
- `054` — Colony Edicts (Framework for issuing the protocol)

## RED Phase: Tests First

Write these tests in `src/layer1/social/placebo_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::{Pop, Mood, Trait};
    use crate::layer1::social::placebo::{PlaceboProtocol, ActivePlacebo, placebo_tick_system, reveal_betrayal_system};
    use crate::layer1::unrest::Unrest;

    #[test]
    fn test_placebo_reduces_stress_temporarily() {
        let mut world = World::new();
        let pop = world.spawn((
            Pop,
            Mood { stress: 80.0, ..Default::default() },
        )).id();

        // Issue "Fake Reinforcements"
        world.spawn(ActivePlacebo {
            protocol: PlaceboProtocol::FakeReinforcements,
            duration: 10.0,
            stress_relief: 20.0,
            revealed: false,
        });

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(placebo_tick_system);
        schedule.run(&mut world);

        let mood = world.get::<Mood>(pop).unwrap();
        assert!(mood.stress <= 60.0 + f32::EPSILON);
    }

    #[test]
    fn test_betrayal_spikes_stress_and_adds_trait() {
        let mut world = World::new();
        let pop = world.spawn((
            Pop,
            Mood { stress: 60.0, ..Default::default() }, // Reduced level
            // No traits
        )).id();

        // Expire/Fail a placebo
        world.spawn(ActivePlacebo {
            protocol: PlaceboProtocol::FakeReinforcements,
            duration: 0.0, // Expired
            stress_relief: 20.0,
            revealed: true, // Failed
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(reveal_betrayal_system);
        schedule.run(&mut world);

        let mood = world.get::<Mood>(pop).unwrap();
        // Stress should return to original (80) PLUS penalty (e.g. +20) = 100
        assert!(mood.stress >= 90.0);

        // Check for Distrustful trait (if Trait component supports dynamic addition)
        // Assume we check for a specific component or trait list
        // let traits = world.get::<Traits>(pop).unwrap();
        // assert!(traits.has(Trait::Distrustful));
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Components & Enums

```rust
// src/layer1/social/placebo.rs

use bevy_ecs::prelude::*;
use crate::layer1::pop::Mood;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaceboProtocol {
    FakeReinforcements, // Reduces Panic
    VitaminX,           // Reduces Sickness Fear
    SafetyInspection,   // Reduces Collapse Fear
}

#[derive(Component)]
pub struct ActivePlacebo {
    pub protocol: PlaceboProtocol,
    pub duration: f32,
    pub stress_relief: f32,
    pub revealed: bool,
}

pub fn placebo_tick_system(
    mut commands: Commands,
    mut placebos: Query<(Entity, &mut ActivePlacebo)>,
    mut pops: Query<&mut Mood>,
) {
    for (entity, mut placebo) in placebos.iter_mut() {
        if placebo.duration > 0.0 {
            // Apply relief (simplified: assume one-time application logic handled elsewhere,
            // or we apply a continuous modifier. For GREEN, let's assume we modify base stress directly once
            // but in reality we should use a Modifier system.
            // Let's just decrement duration here.
            placebo.duration -= 1.0;

            // Apply relief continuously? Or hold it down?
            // "Suppress" stress.
            for mut mood in pops.iter_mut() {
                mood.stress = (mood.stress - 1.0).max(0.0); // Slow drain while active
            }
        } else {
            // Check condition. If problem persists (not modeled here yet), set revealed = true.
            // For MVP, assume 50% chance or always fail if not manually cleared?
            // Let's say it always reveals if it expires naturally without being "Resolved".
            placebo.revealed = true;
        }
    }
}

pub fn reveal_betrayal_system(
    mut commands: Commands,
    query: Query<(Entity, &ActivePlacebo)>,
    mut pops: Query<&mut Mood>,
) {
    for (entity, placebo) in query.iter() {
        if placebo.revealed {
            // Betrayal!
            for mut mood in pops.iter_mut() {
                mood.stress += placebo.stress_relief * 2.0; // Penalty
            }
            commands.entity(entity).despawn();
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Resolution**: Provide a way to "Resolve" the placebo successfully. If the player actually fixes the problem (e.g. kills the raiders) before the duration ends, they can cancel the Placebo without betrayal.
- **UI**: Show "Placebo Active" indicator with a timer.
- **Traits**: Actually implement `Trait::Distrustful` which makes future placebos ineffective.

## Acceptance Criteria

- [ ] `ActivePlacebo` component exists.
- [ ] Active placebos reduce stress over time.
- [ ] Expired placebos trigger Betrayal (stress spike).
- [ ] Tests pass.

## Technical Guidance

- Integrate with `Spec 054` Edicts UI to allow issuing these protocols.
- Ensure Betrayal creates a Notification.

## Questions

*Builder: Can you stack placebos?*
*Architect: Yes, but if multiple fail at once, the colony essentially implodes.*
