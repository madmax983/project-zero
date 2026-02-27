# 257: Subspace Pen Pals

## Overview

"Love and friendship bloom on the battlefield."

Pops working at **Comms Consoles** or Research Labs can form **Remote Bonds** with individuals from other Layer 3 Factions (civilizations). They exchange messages, sharing **Culture** (shifting their Ethics) and **Intel** (Map Data, Trade Routes).

This creates a tension between Openness and Security. Allowing open comms gains you free Intel and potential allies, but risks **Espionage** or **Cultural Contamination** (your Pops adopting alien ethics like "Hive Mind" or "Pacifism").

## Dependencies

- `047` — Pop Relationships (Relationship structure)
- `039` — Trade System (Intel as a resource)
- `068` — Pop Factions (Ethics/Alignment shift)

## RED Phase: Tests First

Write these tests in `src/layer1/social/pen_pals_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::social::pen_pals::{RemoteBond, update_pen_pals_system, PenPalEvent};
    use crate::layer1::pop::Pop;
    use crate::layer1::knowledge::Knowledge; // Or Intel resource
    use crate::layer1::factions::{FactionId, FactionMember};

    #[test]
    fn test_remote_bond_formation() {
        let mut world = World::new();
        let pop = world.spawn(Pop).id();

        // Simulate working at Comms
        // We'll use an event or direct component insertion for the test
        // Assume system chance to form bond
        world.spawn(RemoteBond {
            local_pop: pop,
            foreign_faction: FactionId::HiveMind, // Mock faction
            affinity: 10.0,
        });

        // Verify bond exists (query)
        let count = world.query::<&RemoteBond>().iter(&world).count();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_intel_gain_from_bond() {
        let mut world = World::new();
        world.insert_resource(crate::layer1::resources::ColonyResources::default());

        let pop = world.spawn(Pop).id();
        world.spawn(RemoteBond {
            local_pop: pop,
            foreign_faction: FactionId::HiveMind,
            affinity: 50.0, // High affinity = more intel
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(update_pen_pals_system);
        schedule.run(&mut world);

        // Check Intel/Knowledge increase
        let res = world.resource::<crate::layer1::resources::ColonyResources>();
        // Assuming Intel is tracked or just Knowledge for MVP
        assert!(res.knowledge > 0.0);
    }

    #[test]
    fn test_ethics_shift() {
        let mut world = World::new();
        let pop = world.spawn((
            Pop,
            FactionMember { faction_id: Some(FactionId::MinersGuild) },
        )).id();

        world.spawn(RemoteBond {
            local_pop: pop,
            foreign_faction: FactionId::FarmersGuild, // Different ethic
            affinity: 100.0, // Max affinity forces shift
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(update_pen_pals_system);
        schedule.run(&mut world);

        let member = world.get::<FactionMember>(pop).unwrap();
        // Should shift towards foreign faction (or a specific 'Sympathizer' faction)
        assert_eq!(member.faction_id, Some(FactionId::FarmersGuild));
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Components

```rust
// src/layer1/social/pen_pals.rs

use bevy_ecs::prelude::*;
use crate::layer1::factions::{FactionId, FactionMember};
use crate::layer1::resources::ColonyResources;

#[derive(Component)]
pub struct RemoteBond {
    pub local_pop: Entity,
    pub foreign_faction: FactionId,
    pub affinity: f32,
}

#[derive(Event)]
pub struct PenPalEvent {
    pub pop: Entity,
    pub message: String,
}

pub fn update_pen_pals_system(
    mut commands: Commands,
    mut bonds: Query<&mut RemoteBond>,
    mut resources: ResMut<ColonyResources>,
    mut pops: Query<&mut FactionMember>,
) {
    for mut bond in bonds.iter_mut() {
        // 1. Gain Intel (Knowledge)
        if bond.affinity > 0.0 {
            resources.knowledge += 0.1 * (bond.affinity / 100.0);
        }

        // 2. Ethics Shift check
        if bond.affinity > 80.0 {
            // Chance to convert
            if let Ok(mut member) = pops.get_mut(bond.local_pop) {
                // Simplified: Just switch to the foreign faction ID if it maps to a local one
                // Or we assume FactionId is shared.
                member.faction_id = Some(bond.foreign_faction);
            }
        }
    }
}
```

## REFACTOR Phase: Quality & Design

- **Comms Console Requirement**: The system should only run if the Pop is actively working at a `CommsConsole`.
- **Security Policy**: Add an Edict `Firewall Comms` that blocks Remote Bonds but stops Intel gain.
- **Espionage**: Add a chance for `ColonyResources` (Credits/Intel) to be *lost* if the pen pal is a spy.

## Acceptance Criteria

- [ ] `RemoteBond` component links Pop to Faction.
- [ ] Active bonds generate Knowledge/Intel.
- [ ] High affinity bonds trigger Faction switching.
- [ ] Tests pass.

## Technical Guidance

- Use `FactionId` from `068`.
- Ensure `RemoteBond` is a separate entity or component on the Pop. Separate entity is better for 1-to-many relationships (one pop, many pals).

## Questions

*Builder: Can I date a Hive Mind?*
*Architect: Yes, but your 'Collective' needs will increase.*
