# 110: Spontaneous Architecture

## Overview

Idle hands do the devil's work—or build charming little sheds.
This feature allows idle pops to "claim" empty tiles adjacent to their homes and build personal structures (Sheds, Gardens, Shrines) using colony resources.
These structures provide happiness to the owner but block efficient planning for the player. Demolishing them causes sadness.

**Tension:** Organic, happy chaos vs. Rigid, efficient zoning.

## Dependencies

- `007` — Housing (to know where "home" is)
- `016` — Utility AI (to add the idle behavior)
- `006` — Building Placement (to mark tiles as occupied)
- `045` — Structure Durability (for the structure entity)

## RED Phase: Tests First

Write these tests in `src/layer1/spontaneous_architecture_tests.rs`. They will initially FAIL.

```rust
#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use crate::layer1::pop::{Pop, GridPosition};
    use crate::layer1::building::{Building, BuildingType, Housing, OccupiedTiles};
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::spontaneous_architecture::{
        PersonalStructure, PersonalStructureType, check_spontaneous_build_system,
        demolish_personal_structure_system
    };
    use crate::layer1::morale::Morale;
    use std::collections::HashSet;

    #[test]
    fn test_personal_structure_component() {
        let owner = Entity::from_raw(1);
        let structure = PersonalStructure {
            owner,
            structure_type: PersonalStructureType::Garden,
            beauty_bonus: 5.0,
        };
        assert_eq!(structure.owner, owner);
        assert_eq!(structure.structure_type, PersonalStructureType::Garden);
    }

    #[test]
    fn test_check_spontaneous_build_finds_adjacent_empty_tile() {
        let mut world = World::new();
        // Setup Grid
        world.insert_resource(crate::layer1::terrain::TerrainGrid::new(10, 10));
        world.insert_resource(OccupiedTiles::default());
        world.insert_resource(ColonyResources { wood: 100.0, stone: 100.0, ..Default::default() });

        // Setup Pop with Home
        let home_pos = GridPosition { x: 5, y: 5 };
        let home = world.spawn((
            Building { building_type: BuildingType::Housing },
            Housing { capacity: 1, residents: vec![] }, // Will add resident below
            home_pos,
        )).id();

        let pop = world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 }, // At home
            // Idle state would be checked by system logic, here we test the outcome
        )).id();

        // Manually link pop to home (simulating residence)
        world.get_mut::<Housing>(home).unwrap().residents.push(pop);

        // Run system
        // We mock the "Idle" check by ensuring system only runs on idle pops or forcing it
        // For unit test, we call the logic directly or setup the conditions
        check_spontaneous_build_system(&mut world);

        // Assert: A personal structure should spawn adjacent to (5,5)
        let structures: Vec<_> = world.query::<(&PersonalStructure, &GridPosition)>().iter(&world).collect();
        assert_eq!(structures.len(), 1);

        let (_, pos) = structures[0];
        let dx = (pos.x - home_pos.x).abs();
        let dy = (pos.y - home_pos.y).abs();
        assert!(dx <= 1 && dy <= 1 && (dx + dy) > 0, "Must be adjacent");

        // Assert: Tile occupied
        let occupied = world.resource::<OccupiedTiles>();
        assert!(occupied.0.contains(&(pos.x, pos.y)));
    }

    #[test]
    fn test_spontaneous_build_consumes_resources() {
        let mut world = World::new();
        world.insert_resource(crate::layer1::terrain::TerrainGrid::new(10, 10));
        world.insert_resource(OccupiedTiles::default());
        world.insert_resource(ColonyResources { wood: 10.0, ..Default::default() }); // Exact cost for Shed

        let home = world.spawn((
            Building { building_type: BuildingType::Housing },
            Housing { capacity: 1, residents: vec![] },
            GridPosition { x: 5, y: 5 },
        )).id();

        let pop = world.spawn(Pop).id();
        world.get_mut::<Housing>(home).unwrap().residents.push(pop);

        // Force build specific type if possible, or rely on RNG/Traits in implementation
        // For test, assume default or mocked choice
        check_spontaneous_build_system(&mut world);

        let res = world.resource::<ColonyResources>();
        assert!(res.wood < 10.0, "Should consume wood for shed");
    }

    #[test]
    fn test_demolish_causes_sadness() {
        let mut world = World::new();

        let owner = world.spawn((
            Pop,
            Morale { value: 0.8, modifiers: vec![] },
        )).id();

        let structure = world.spawn((
            PersonalStructure {
                owner,
                structure_type: PersonalStructureType::Garden,
                beauty_bonus: 5.0,
            },
            // Simulate "Being Demolished" - usually handled by a command or event
            // Here we test the reaction system that watches for removal
        )).id();

        // Despawn the structure (simulating demolition)
        world.despawn(structure);

        // Run reaction system
        demolish_personal_structure_system(&mut world); // This needs to track despawns or use an event

        // Assert: Morale dropped
        let morale = world.get::<Morale>(owner).unwrap();
        assert!(morale.value < 0.8);
    }

    #[test]
    fn test_cannot_build_if_blocked() {
        let mut world = World::new();
        world.insert_resource(crate::layer1::terrain::TerrainGrid::new(10, 10));
        let mut occupied = OccupiedTiles::default();
        // Surround (5,5)
        for x in 4..=6 {
            for y in 4..=6 {
                if x != 5 || y != 5 {
                    occupied.0.insert((x, y));
                }
            }
        }
        world.insert_resource(occupied);
        world.insert_resource(ColonyResources::default());

        let home = world.spawn((
            Building { building_type: BuildingType::Housing },
            Housing { capacity: 1, residents: vec![] },
            GridPosition { x: 5, y: 5 },
        )).id();
        let pop = world.spawn(Pop).id();
        world.get_mut::<Housing>(home).unwrap().residents.push(pop);

        check_spontaneous_build_system(&mut world);

        let count = world.query::<&PersonalStructure>().iter(&world).count();
        assert_eq!(count, 0, "Should not build if blocked");
    }
}
```

## GREEN Phase: Minimal Implementation

### 1. Define Components

In `src/layer1/spontaneous_architecture.rs`:

```rust
use bevy_ecs::prelude::*;
use crate::layer1::building::{BuildingType, MaterialType, OccupiedTiles, try_place_building};
use crate::layer1::pop::{Pop, GridPosition};
use crate::layer1::housing::Housing;
use crate::layer1::resources::ColonyResources;
use crate::layer1::morale::Morale; // Assuming Morale component exists

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PersonalStructureType {
    Shed,
    Garden,
    Shrine,
}

#[derive(Component)]
pub struct PersonalStructure {
    pub owner: Entity,
    pub structure_type: PersonalStructureType,
    pub beauty_bonus: f32,
}
```

### 2. Implement Build System

```rust
pub fn check_spontaneous_build_system(world: &mut World) {
    // 1. Find idle pops with housing
    // For MVP, just random chance for any pop in housing
    // In real implementation, check UtilityAI idle state

    let mut build_requests = Vec::new();

    let housing_query = world.query::<(Entity, &GridPosition, &Housing)>();
    // Collect potential builders to avoid borrowing conflict
    // (This is a simplified selection logic for the Green phase)

    // ... logic to pick a random resident and a random adjacent tile ...
    // ... check if tile is empty using OccupiedTiles ...
    // ... push to build_requests ...

    for req in build_requests {
        // consumes resources via try_place_building logic or custom logic
        // spawns PersonalStructure entity
    }
}
```

### 3. Update BuildingType

Update `src/layer1/building.rs` to include:
- `PersonalShed`
- `PersonalGarden`
- `PersonalShrine`

Update `try_place_building` or create `try_place_personal_structure` that handles the `PersonalStructure` component attachment.

### 4. Demolition Reaction

In `src/layer1/spontaneous_architecture.rs`:

```rust
// Requires an event or checking `RemovedComponents<PersonalStructure>`
pub fn demolish_personal_structure_system(
    mut removed: RemovedComponents<PersonalStructure>,
    mut morale_query: Query<&mut Morale>,
    // We need to store the owner info somewhere before removal,
    // or use an Event triggered by the Demolish tool.
    // For Green phase, we can use an event listener pattern if `demolish` triggers one.
    // OR, we store a lookup: Owner -> Structure.
) {
    // Implementation detail:
    // When a PersonalStructure is despawned, we might lose the `owner` data
    // unless we use `RemovedComponents` which only gives the Entity ID.
    // Solution: The Demolish tool should fire a `StructureDemolished(Entity)` event
    // BEFORE despawn, or we store the link on the Pop (`HasStructure(Entity)`).

    // Simpler for Green: Pop has `ClaimedStructure(Entity)`.
    // If that entity doesn't exist anymore, apply penalty and remove component.
}
```

## REFACTOR Phase: Quality & Design

- **Link to Traits**: `NatureLover` builds Gardens, `Industrious` builds Sheds.
- **Link to UtilityAI**: Only build when `ActionType::Idle` and bored.
- **Visuals**: Use distinct ASCII chars or colors (e.g., 's' for shed, '*' for garden).
- **Demolish Logic**: Ensure the Demolish tool (Spec 006/030) properly triggers the sadness.
- **Limit**: One structure per pop.

## Acceptance Criteria

- [ ] `PersonalStructure` component exists.
- [ ] `BuildingType` enum extended.
- [ ] Idle pops build structures adjacent to their home.
- [ ] Construction consumes colony resources (Shed=Wood, Shrine=Stone).
- [ ] Tiles are marked occupied.
- [ ] Demolishing the structure reduces the owner's Morale.
- [ ] Tests pass.

## Technical Guidance

- **Avoid Infinite Sprawl**: Pops should strictly be limited to 1 personal structure. Check if they already have one (`Query<&PersonalStructure>`).
- **Demolition**: The easiest way to track destruction is to have a system that checks `if world.get_entity(structure_id).is_none()` for every pop that claims to own one.
```rust
#[derive(Component)]
struct OwnsStructure(Entity);

fn check_demolition(mut commands: Commands, mut query: Query<(Entity, &mut Morale, &OwnsStructure)>, world: &World) {
    for (pop, mut morale, ownership) in query.iter_mut() {
        if world.get_entity(ownership.0).is_none() {
            morale.add_modifier("They destroyed my shed!", -0.2, 100);
            commands.entity(pop).remove::<OwnsStructure>();
        }
    }
}
```
