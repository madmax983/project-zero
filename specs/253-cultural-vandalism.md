# 253: Cultural Vandalism

## Overview

"The streets speak back to the palace."

When **Unrest** is high, unhappy Pops may target "Official" structures (Statues, Banners, Propaganda Screens) to perform **Vandalism**.
Vandalized structures lose their positive buffs (e.g., Morale, Loyalty) and instead project **negative** effects (e.g., "Rebellion" symbol increases Unrest further, or reduces Authority).
This creates a visual feedback loop for dissatisfaction: the colony starts looking like a slum/warzone.

- **Vandalism State**: Structures gain a `Vandalized` component.
- **Inversion**: `Statue` (+Mood) becomes `Defaced Statue` (+Unrest).
- **Cleanup**: "Clean" action required to restore function.

## Dependencies

- `050` — Civil Unrest (Trigger)
- `061` — Cultural Artifacts (Targets)
- `233` — Public Grievances (Sentiment)

## RED Phase: Tests First

Write these tests in `src/layer1/social/vandalism_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::social::vandalism::{Vandalized, vandalism_system, update_structure_buffs};
    use crate::layer1::structure::{Structure, StructureType};
    use crate::layer1::pop::{Pop, Mood};
    use crate::layer1::map::GridPosition;

    #[test]
    fn test_vandalism_application() {
        let mut world = World::new();
        // Spawn Angry Pop
        world.spawn((
            Pop,
            Mood { stress: 90.0, ..Default::default() }, // High stress/unrest
            GridPosition { x: 0, y: 0 },
        ));

        // Spawn Statue
        let statue = world.spawn((
            Structure { structure_type: StructureType::Statue, ..Default::default() },
            GridPosition { x: 0, y: 1 }, // Adjacent
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(vandalism_system);
        schedule.run(&mut world);

        assert!(world.get::<Vandalized>(statue).is_some());
    }

    #[test]
    fn test_vandalized_structure_inverts_buff() {
        let mut world = World::new();
        // Spawn Vandalized Statue with Buff emitter (mock)
        let statue = world.spawn((
            Structure { structure_type: StructureType::Statue, ..Default::default() },
            crate::layer1::buffs::Aura { effect: 10.0 }, // +10 Morale
            Vandalized,
        )).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(update_structure_buffs);
        schedule.run(&mut world);

        let aura = world.get::<crate::layer1::buffs::Aura>(statue).unwrap();
        assert!(aura.effect < 0.0); // Should be negative
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Components

```rust
// src/layer1/social/vandalism.rs

use bevy_ecs::prelude::*;
use crate::layer1::pop::Mood;
use crate::layer1::structure::{Structure, StructureType};
use crate::layer1::map::GridPosition;
use crate::layer1::buffs::Aura; // Assuming existing Aura system from 061/044

#[derive(Component)]
pub struct Vandalized;

pub fn vandalism_system(
    mut commands: Commands,
    pops: Query<(&Mood, &GridPosition)>,
    structures: Query<(Entity, &Structure, &GridPosition), Without<Vandalized>>,
) {
    for (mood, pop_pos) in pops.iter() {
        if mood.stress > 80.0 { // Arbitrary threshold for "Angry"
            for (entity, structure, struct_pos) in structures.iter() {
                // Target only cultural/official structures
                if matches!(structure.structure_type, StructureType::Statue | StructureType::Banner) {
                    if pop_pos.distance_chebyshev(*struct_pos) <= 1 {
                        // Vandalize
                        commands.entity(entity).insert(Vandalized);
                        // Optional: Add visual "Graffiti" child entity
                    }
                }
            }
        }
    }
}

pub fn update_structure_buffs(
    mut query: Query<(&mut Aura, &Structure), (With<Vandalized>, Changed<Vandalized>)>,
) {
    for (mut aura, structure) in query.iter_mut() {
        // Invert effect
        // Assuming Aura value corresponds to Morale
        if aura.effect > 0.0 {
            aura.effect = -aura.effect * 1.5; // "Rebellion" is stronger than "Loyalty"
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Cooldown**: Vandalism shouldn't happen every tick. Use a "Riot" event or low probability check.
- **Cleaning**: "Janitor" or "Police" jobs should have a `CleanVandalism` action that removes the component and restores the buff.
- **Sentiment**: Integrate with `Spec 233` (Public Grievances). Vandalism creates a `Grievance` note on the Bulletin Board automatically.

## Acceptance Criteria

- [ ] `Vandalized` component exists.
- [ ] Angry pops apply it to nearby symbols.
- [ ] Buffs invert when vandalized.
- [ ] Tests pass.

## Technical Guidance

- Use `StructureType` enum to identify targets.
- Ensure `Aura` system (if it exists) recalculates when values change.

## Questions

*Builder: Does vandalism destroy the building?*
*Architect:* No, it only appends a Vandalized component that inversions its Aura effect.
