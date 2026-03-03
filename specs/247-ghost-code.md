# 247: Ghost Code

## Overview

"Machines have souls, or at least memory leaks."

When a building is deconstructed, it leaves behind invisible **Data Residue** on the tile. This residue contains fragmented code/protocols from the previous structure.
If a new building is constructed on top of Data Residue, it may inherit **Ghost Behaviors**—glitches or traits derived from the previous building's function.

- **Residue**: Invisible marker on tile after deconstruction.
- **Infection**: New building constructed on Residue gets `GhostCode` component.
- **Purge**: Action to clear residue before building (requires time/labor).

## Dependencies

- `006` — Building Placement (Construction events)
- `020` — Construction Costs (Deconstruction events)

## RED Phase: Tests First

Write these tests in `src/layer1/tech/ghost_code_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::tech::ghost_code::{DataResidue, GhostCode, GhostTrait, residue_system, ghost_infection_system};
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::map::GridPosition;
    use crate::layer1::events::{BuildingCompletedEvent, BuildingRemovedEvent};

    #[test]
    fn test_deconstruction_leaves_residue() {
        let mut world = World::new();
        let mut events = world.resource_mut::<Events<BuildingRemovedEvent>>();

        let pos = GridPosition { x: 10, y: 10 };
        // Simulate deconstruction event
        events.send(BuildingRemovedEvent {
            entity: Entity::PLACEHOLDER, // Mock entity
            position: pos,
            building_type: BuildingType::Turret,
        });

        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(residue_system);
        schedule.run(&mut world);

        // Check for Residue entity at pos
        let mut found = false;
        for (residue, p) in world.query::<(&DataResidue, &GridPosition)>().iter(&world) {
            if *p == pos {
                assert_eq!(residue.source_type, BuildingType::Turret);
                found = true;
            }
        }
        assert!(found, "DataResidue should be spawned at deconstruction site");
    }

    #[test]
    fn test_construction_inherits_ghost_code() {
        let mut world = World::new();
        let pos = GridPosition { x: 5, y: 5 };

        // 1. Spawn Residue (from old MedicalBay)
        world.spawn((
            DataResidue { source_type: BuildingType::MedicalBay },
            pos
        ));

        // 2. Build New Building (Turret)
        let new_building = world.spawn((
            Building { building_type: BuildingType::Turret, ..Default::default() },
            pos
        )).id();

        // 3. Trigger Completion Event
        let mut events = world.resource_mut::<Events<BuildingCompletedEvent>>();
        events.send(BuildingCompletedEvent { entity: new_building });

        // 4. Run Infection System
        let mut schedule = Schedule::default();
        schedule.add_systems(ghost_infection_system);
        schedule.run(&mut world);

        // 5. Assert GhostCode presence
        let ghost = world.get::<GhostCode>(new_building).expect("Building should acquire GhostCode");
        assert_eq!(ghost.traits.len(), 1);
        // MedicalBay residue on Turret might cause "PacifistTargeting" or similar
        assert!(format!("{:?}", ghost.traits[0]).contains("Medical"));
    }

    #[test]
    fn test_purge_action_removes_residue() {
        let mut world = World::new();
        let pos = GridPosition { x: 0, y: 0 };
        let residue_entity = world.spawn((
            DataResidue { source_type: BuildingType::Wall },
            pos
        )).id();

        // Perform Purge Action
        crate::layer1::tech::ghost_code::perform_purge(&mut world, residue_entity);

        assert!(world.get_entity(residue_entity).is_none(), "Residue should be despawned after purge");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Components

```rust
// src/layer1/tech/ghost_code.rs

use bevy_ecs::prelude::*;
use crate::layer1::building::BuildingType;
use crate::layer1::map::GridPosition;
use crate::layer1::events::{BuildingCompletedEvent, BuildingRemovedEvent};

#[derive(Component, Debug, Clone)]
pub struct DataResidue {
    pub source_type: BuildingType,
}

#[derive(Component, Debug, Default)]
pub struct GhostCode {
    pub traits: Vec<GhostTrait>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GhostTrait {
    PhantomPower, // Consumes power for no reason
    LegacyTargeting, // Turret tries to heal / Medbay tries to shoot
    GhostProtocol(String), // Flavor text
}

// Map building types to ghost traits
impl DataResidue {
    pub fn get_ghost_trait(&self) -> GhostTrait {
        match self.source_type {
            BuildingType::Turret => GhostTrait::LegacyTargeting,
            BuildingType::MedicalBay => GhostTrait::GhostProtocol("Triage".into()),
            _ => GhostTrait::PhantomPower,
        }
    }
}
```

### 2. Systems

```rust
pub fn residue_system(
    mut commands: Commands,
    mut events: EventReader<BuildingRemovedEvent>,
) {
    for event in events.read() {
        commands.spawn((
            DataResidue { source_type: event.building_type },
            event.position,
            // Name it for debug
            Name::new("Data Residue"),
        ));
    }
}

pub fn ghost_infection_system(
    mut commands: Commands,
    mut events: EventReader<BuildingCompletedEvent>,
    building_query: Query<&GridPosition, With<Building>>,
    residue_query: Query<(Entity, &DataResidue, &GridPosition)>,
) {
    for event in events.read() {
        if let Ok(build_pos) = building_query.get(event.entity) {
            // Check for residue at this position
            for (residue_entity, residue, residue_pos) in residue_query.iter() {
                if build_pos == residue_pos {
                    // Infect!
                    commands.entity(event.entity).insert(GhostCode {
                        traits: vec![residue.get_ghost_trait()],
                    });

                    // Consume residue? Or does it persist?
                    // Let's say it persists unless purged, but maybe gets "overwritten" (consumed) by the new build?
                    // For now, consume it to prevent infinite stacking.
                    commands.entity(residue_entity).despawn();
                }
            }
        }
    }
}

pub fn perform_purge(world: &mut World, residue_entity: Entity) {
    world.despawn(residue_entity);
}
```

## REFACTOR Phase: Quality & Design

- **Visualization**: Residue needs to be invisible normally, but visible with "Tech View" or specific Scanner tools. Add a `Visibility` component toggled by view mode.
- **Purge Job**: `perform_purge` should be the result of a `Job::Clean` or `Job::SysAdmin`.
- **Trait Logic**: Implement actual effects for `GhostTrait`. e.g., `LegacyTargeting` modifies `Turret` behavior in `combat.rs`.

## Acceptance Criteria

- [ ] `DataResidue` spawns on deconstruction.
- [ ] New buildings absorb `DataResidue` and gain `GhostCode`.
- [ ] `GhostTrait` logic is mapped (even if placeholder effects).
- [ ] Tests pass.

## Technical Guidance

- Use `GridPosition` matching carefully.
- Ensure `BuildingRemovedEvent` contains `BuildingType`. If not, refactor `020` or wherever that event is defined to include it.
- Residue should probably *not* block movement or construction.

## Questions

- *Builder: Does residue stack?*
  *Architect: No, only the ghost behavior of the most recently demolished building on that tile is retained.*
*Architect: No, the most recent building demolished on that tile overwrites previous residue to prevent excessive memory usage.*
  - *Architect: No, residue does not stack. A tile either has residue or it doesn't. Deconstructing multiple buildings on the same tile just refreshes the residue.*
  - *Architect: No, latest residue overwrites old, or they merge. Keep it simple: One residue per tile.*
