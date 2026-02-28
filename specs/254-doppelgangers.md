# 254: Doppelgangers

## Overview

"Who is that working next to you?"

Introduces **Doppelgangers** (or Mimics), alien entities that replace existing Pops. They look identical to the original Pop but behave differently. Instead of working efficiently, they **Sabotage** tasks (consuming double resources, producing waste, or damaging machines) and spread **Paranoia**.

This feature adds a layer of internal security and suspicion. Players must observe Pop behavior or use medical scanners to identify and remove impostors.

## Dependencies

- `004` — Pop Entity (Identity)
- `009` — Job System (Sabotage triggers during work)
- `034` — Pop Health (Damage/Reveal mechanics)
- `046` — Notifications System (Alerts when sabotage happens)

## RED Phase: Tests First

Write these tests in `src/layer1/pop/doppelganger_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::{Pop, Name, Health};
    use crate::layer1::pop::doppelganger::{Mimic, sabotage_system, reveal_mimic, MimicState};
    use crate::layer1::job::{JobState, WorkProgress};
    use crate::layer1::map::GridPosition;

    #[test]
    fn test_mimic_replacement() {
        let mut world = World::new();
        let original_pop = world.spawn((
            Pop,
            Name::new("Miner Bob"),
            Health { current: 100.0, max: 100.0 },
            GridPosition { x: 10, y: 10 }
        )).id();

        // Perform replacement
        let mimic_entity = crate::layer1::pop::doppelganger::replace_pop_with_mimic(&mut world, original_pop);

        // Original should be despawned or marked missing
        assert!(world.get_entity(original_pop).is_none());

        // Mimic should exist and look like original
        let mimic_name = world.get::<Name>(mimic_entity).unwrap();
        assert_eq!(mimic_name.as_str(), "Miner Bob");

        // Mimic should have Mimic component
        assert!(world.get::<Mimic>(mimic_entity).is_some());
    }

    #[test]
    fn test_mimic_sabotage_during_work() {
        let mut world = World::new();
        let mimic = world.spawn((
            Pop,
            Mimic { state: MimicState::Hidden },
            JobState::Working,
            WorkProgress { current: 50.0, max: 100.0 }, // Mid-work
            // Some resource to sabotage?
        )).id();

        // Run sabotage system
        let mut schedule = Schedule::default();
        schedule.add_systems(sabotage_system);
        schedule.run(&mut world);

        // Check for sabotage effects
        // e.g., Progress reduced (undoing work) or health damaged
        let progress = world.get::<WorkProgress>(mimic).unwrap();
        // Sabotage: Mimic works backwards or stalls
        assert!(progress.current <= 50.0);
    }

    #[test]
    fn test_mimic_detection_via_scan() {
        let mut world = World::new();
        let mimic = world.spawn((
            Pop,
            Mimic { state: MimicState::Hidden },
            Name::new("Suspect"),
        )).id();

        // Perform Scan Action (simulated function call)
        let is_mimic = reveal_mimic(&mut world, mimic);

        assert!(is_mimic);

        // Mimic state should change to Revealed
        let mimic_comp = world.get::<Mimic>(mimic).unwrap();
        assert_eq!(mimic_comp.state, MimicState::Revealed);
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Components

```rust
// src/layer1/pop/doppelganger.rs

use bevy_ecs::prelude::*;
use crate::layer1::pop::{Pop, Name, Health};
use crate::layer1::map::GridPosition;
use crate::layer1::job::WorkProgress;

#[derive(Component, Debug, PartialEq, Clone)]
pub struct Mimic {
    pub state: MimicState,
    pub original_identity: String, // Store original info if we want to "rescue" them later
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MimicState {
    Hidden,
    Revealed,
}

pub fn replace_pop_with_mimic(world: &mut World, target: Entity) -> Entity {
    // 1. Copy data from target
    let name = world.get::<Name>(target).unwrap().clone();
    let pos = *world.get::<GridPosition>(target).unwrap();
    let health = *world.get::<Health>(target).unwrap();

    // 2. Despawn target
    world.despawn(target);

    // 3. Spawn Mimic
    world.spawn((
        Pop,
        name.clone(),
        pos,
        health,
        Mimic {
            state: MimicState::Hidden,
            original_identity: name.as_str().to_string(),
        },
    )).id()
}

pub fn sabotage_system(
    mut query: Query<(&mut WorkProgress, &Mimic)>,
) {
    for (mut progress, mimic) in query.iter_mut() {
        if mimic.state == MimicState::Hidden {
            // "Undo" work subtly
            // 10% chance to reduce progress instead of adding it?
            // For Green phase: just clamp progress to never finish
            if progress.current > 90.0 {
                progress.current = 80.0; // Infinite loop of almost finishing
            }
        }
    }
}

pub fn reveal_mimic(world: &mut World, target: Entity) -> bool {
    if let Some(mut mimic) = world.get_mut::<Mimic>(target) {
        mimic.state = MimicState::Revealed;
        // Trigger notification event here?
        return true;
    }
    false
}
```

## REFACTOR Phase: Quality & Design

- **Sabotage Variety**: Instead of just stalling work, they should consume resources (eat 2x food) or damage the machine they are using (durability loss).
- **Rescue**: The original Pop isn't dead, just "cocooned" somewhere. Finding the nest allows rescue.
- **Paranoia**: Sabotage acts should trigger `Paranoia` in nearby witnesses, lowering morale.
- **Visuals**: Revealed Mimics should change sprite/color (e.g., Green tint or glitch effect).

## Acceptance Criteria

- [ ] `Mimic` component exists.
- [ ] Replacement logic correctly swaps entities while preserving Name/Pos.
- [ ] `sabotage_system` prevents Mimics from completing work.
- [ ] `reveal_mimic` exposes the `Mimic` component.
- [ ] Tests pass.

## Technical Guidance

- Use `DespawnRecursive` if the Pop has children entities (tools, inventory).
- Ensure `replace_pop_with_mimic` transfers *all* critical components (Needs, Skills) so the game doesn't crash expecting them.
- `Mimic` should probably imply `Trait::Deceptive` if traits are used for social interactions.

## Questions

- *Builder: How does the Mimic spawn?*
*Architect: A Mimic replaces an existing pop when they are alone in a dark or remote area. The original pop is quietly "absorbed" and the Mimic takes their identity.*
  - *Architect: Random event during "High Strangeness" or after a "Meteor" event. For now, just the mechanic, spawn logic is an Event.*
