# 057: Funeral Rites

## Overview

Death is a significant event in the colony. When a Pop dies, they should not simply disappear (despawn). Instead, they should leave behind a **Corpse**. Unburied corpses are a health hazard and a source of significant **Grief** (morale penalty) for the colony.

This spec introduces:
1.  **Corpse Entity**: Spawns when a Pop dies. Contains the name of the deceased.
2.  **Grave Building**: A place to bury corpses.
3.  **Funeral Action**: A job for other Pops to carry the corpse to a grave and bury it.
4.  **Closure**: Completing a funeral replaces "Grief" with "Closure" or "Mourning" memories.

## Dependencies

- `034` — Pop Health (Death system)
- `031` — Pop Morale (Grief/Mood effects)
- `036` — Pop Memory (Memories of death)
- `047` — Pop Relationships (Impact on friends/family)
- `006` — Building Placement (Grave building)

## RED Phase: Tests First

Write these tests BEFORE any implementation. They will initially FAIL.

```rust
// src/layer1/funeral_tests.rs

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::health::{Health, death_system};
    use crate::layer1::pop::{Pop, PopName};
    use crate::layer1::map::GridPosition;
    use crate::layer1::funeral::{Corpse, Grave, FuneralAction, grief_system};
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::needs::Needs;
    use crate::layer1::memory::{Memories, MemoryType};
    use crate::layer1::execution::{Assignment, AssignmentType};

    #[test]
    fn test_death_spawns_corpse() {
        let mut world = World::new();
        let entity = world.spawn((
            Pop,
            PopName("TestSubject".to_string()),
            Health { current: -10.0, max: 100.0 }, // Dead
            GridPosition { x: 5, y: 5 },
        )).id();

        // Run death system
        death_system(&mut world);

        // Pop should be despawned
        assert!(world.get_entity(entity).is_err());

        // Corpse should be spawned at same location
        let mut query = world.query::<(&Corpse, &GridPosition)>();
        let corpse = query.single(&world);
        assert_eq!(corpse.0.name, "TestSubject");
        assert_eq!(corpse.1.x, 5);
        assert_eq!(corpse.1.y, 5);
    }

    #[test]
    fn test_corpse_decay_causes_grief() {
        let mut world = World::new();

        // Spawn Corpse
        world.spawn((
            Corpse { name: "Dearly Departed".to_string(), decay: 0.5 },
            GridPosition { x: 0, y: 0 },
        ));

        // Spawn Witness (Pop) nearby
        let witness = world.spawn((
            Pop,
            GridPosition { x: 1, y: 0 }, // Adjacent
            Needs::default(), // Has morale
            Memories::default(),
        )).id();

        // Run grief system
        grief_system(&mut world);

        // Check if witness has negative memory or reduced morale
        // (Assuming grief_system adds a 'SawCorpse' memory or directly impacts morale)
        let memories = world.get::<Memories>(witness).unwrap();
        // MemoryType::SawCorpse should be defined
        assert!(memories.contains(MemoryType::SawCorpse));
    }

    #[test]
    fn test_grave_accepts_corpse() {
        let mut world = World::new();

        let grave = world.spawn((
            Building { building_type: BuildingType::Grave },
            Grave { occupied: false, corpse_name: None },
        )).id();

        let corpse = world.spawn((
            Corpse { name: "Bob".to_string(), decay: 0.0 },
        )).id();

        // Simulate funeral completion (abstraction of the action)
        // In reality, this would be an action execution function
        crate::layer1::funeral::bury_corpse(&mut world, grave, corpse);

        let grave_comp = world.get::<Grave>(grave).unwrap();
        assert!(grave_comp.occupied);
        assert_eq!(grave_comp.corpse_name.as_deref(), Some("Bob"));

        // Corpse entity should be despawned
        assert!(world.get_entity(corpse).is_err());
    }

    #[test]
    fn test_burial_grants_closure() {
        let mut world = World::new();
        // Setup pop with Grief memory
        let pop = world.spawn((
            Pop,
            Memories::default(), // Should have Grief
        )).id();

        // Add Grief manually
        world.get_mut::<Memories>(pop).unwrap().add(MemoryType::WitnessedDeath, 0);

        // Perform burial (this might be a system or action effect)
        // For test simplicity, assume a function applies the effect
        crate::layer1::funeral::apply_closure(&mut world, pop);

        let memories = world.get::<Memories>(pop).unwrap();
        // Should have 'AttendedFuneral' or similar positive/neutral memory
        assert!(memories.contains(MemoryType::AttendedFuneral));
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Update `BuildingType`

Add `Grave` to `src/layer1/building.rs`.

```rust
pub enum BuildingType {
    // ...
    Grave,
}

impl BuildingType {
    pub const fn char(&self) -> char {
        match self {
            Self::Grave => '†', // or similar
            // ...
        }
    }

    pub const fn cost(&self) -> ColonyResources {
        match self {
            Self::Grave => ColonyResources { stone: 5.0, ..ColonyResources::zeroed() },
            // ...
        }
    }
}
```

### 2. Create `src/layer1/funeral.rs`

```rust
use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::memory::{Memories, MemoryType};
use crate::layer1::building::{Building, BuildingType};

#[derive(Component, Debug, Clone)]
pub struct Corpse {
    pub name: String,
    pub decay: f32, // 0.0 to 1.0? Or ticks?
}

#[derive(Component, Debug, Clone, Default)]
pub struct Grave {
    pub occupied: bool,
    pub corpse_name: Option<String>,
}

pub fn grief_system(world: &mut World) {
    // 1. Find all corpses
    // 2. Find all pops near corpses
    // 3. Apply 'SawCorpse' memory (small mood debuff)
}

pub fn bury_corpse(world: &mut World, grave_entity: Entity, corpse_entity: Entity) {
    let corpse_name = if let Some(c) = world.get::<Corpse>(corpse_entity) {
        c.name.clone()
    } else {
        return;
    };

    if let Some(mut grave) = world.get_mut::<Grave>(grave_entity) {
        grave.occupied = true;
        grave.corpse_name = Some(corpse_name);
    }

    world.despawn(corpse_entity);
}

pub fn apply_closure(world: &mut World, pop_entity: Entity) {
    if let Some(mut memories) = world.get_mut::<Memories>(pop_entity) {
        memories.add(MemoryType::AttendedFuneral, 0); // Need current tick
    }
}
```

### 3. Modify `death_system` in `src/layer1/health.rs`

Use `crate::layer1::funeral::Corpse` (behind feature flag if needed, or core).

```rust
// In death_system iteration:
let corpse_name = if let Some(name) = world.get::<PopName>(entity) {
    name.0.clone()
} else { "Unknown".to_string() };

// Spawn Corpse before despawn
world.spawn((
    Corpse { name: corpse_name, decay: 0.0 },
    *pos, // GridPosition
));
```

### 4. Update Utility AI

Add `ActionType::BuryCorpse` and `AssignmentType::Funeral`.
Pops should prioritize burying corpses if a `Grave` (unoccupied) exists.

## REFACTOR Phase: Quality & Design

- **Decay**: Corpses should eventually turn into `Bone` resources if not buried? Or trigger `Disease`.
- **Relationship Integration**: Only friends/family should prioritize the funeral, or get more "Closure" from it.
- **Graveyard Zone**: Instead of individual grave buildings, maybe a Zone? But Building `Grave` is simpler for MVP.
- **Visuals**: Corpse sprite needed.

## Acceptance Criteria

- [ ] `Corpse` entity spawns on Pop death with correct name.
- [ ] `Grave` building is constructible.
- [ ] Pops automatically haul corpses to empty graves (`BuryCorpse` action).
- [ ] Unburied corpses cause negative memories/morale.
- [ ] Buried corpses (Funeral) grant closure memory.
- [ ] Tests pass.

## Technical Guidance

- Ensure `death_system` doesn't double-spawn if called multiple times (it shouldn't, as it despawns the pop).
- `ActionType` enum in `utility_ai.rs` needs update.
- `AssignmentType` enum in `execution.rs` needs update.

## Questions

- Should enemies (Raiders) leave corpses? (Yes, treat same as colonists for hygiene, but maybe different morale effect).
  - *Architect:* Yes, they leave corpses and cause hygiene/morale debuffs if left unburied.
