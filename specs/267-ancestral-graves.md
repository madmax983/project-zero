# 267: Ancestral Graves

## 1. Overview

The history of the colony is written on the land itself. When pops die, they leave behind "Grave" tiles. These graves become points of interest for surviving relatives who visit them to receive mood buffs. However, graves cannot be built over without incurring a severe "Sacrilege" penalty, forcing players to either respect the dead or prioritize efficiency at the cost of morale.

## 2. Dependencies

- `004` — Pop Entity
- `006` — Building Placement System
- `031` — Pop Morale
- `047` — Pop Relationships

## 3. RED Phase: Tests First

```rust
// src/layer1/grave_tests.rs

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use crate::layer1::map::{GridPosition, TerrainGrid, TerrainType};
    use crate::layer1::building::{BuildingGrid, BuildingType};
    use crate::layer1::pop::Pop;
    use crate::layer1::relationships::{Relationships, Affinity};
    use crate::layer1::morale::MoraleModifier;
    use crate::layer1::needs::Needs;

    #[test]
    fn test_grave_spawns_on_pop_death() {
        let mut world = World::new();
        let pos = GridPosition { x: 5, y: 5 };

        let pop_entity = world.spawn((
            Pop,
            pos,
            Name::new("Miner Bob"),
        )).id();

        // Trigger death logic
        handle_pop_death(&mut world, pop_entity);

        // Verify grave exists at the position
        let graves = world.query_filtered::<&GridPosition, With<Grave>>().iter(&world).collect::<Vec<_>>();
        assert_eq!(graves.len(), 1);
        assert_eq!(*graves[0], pos);

        let grave_comp = world.query::<&Grave>().single(&world);
        assert_eq!(grave_comp.pop_name, "Miner Bob");
    }

    #[test]
    fn test_visiting_grave_grants_mood_buff_to_relative() {
        let mut world = World::new();
        let pos = GridPosition { x: 5, y: 5 };

        let relative_entity = world.spawn((
            Pop,
            pos, // Standing on grave
            Relationships {
                // High affinity indicates a close relationship
                affinities: vec![(Entity::PLACEHOLDER, Affinity::High)]
            },
            Needs::default(),
        )).id();

        // Setup grave with the relative's dead kin
        world.spawn((
            Grave {
                pop_name: "Miner Bob".to_string(),
                original_entity: Entity::PLACEHOLDER,
            },
            pos,
        ));

        // Run visit system
        visit_grave_system(&mut world);

        // Verify buff applied
        let needs = world.get::<Needs>(relative_entity).unwrap();
        assert!(needs.morale() > 0.5); // Baseline is 0.5, expect an increase
    }

    #[test]
    fn test_building_over_grave_causes_sacrilege() {
        let mut world = World::new();
        let pos = GridPosition { x: 5, y: 5 };

        let grave_entity = world.spawn((
            Grave {
                pop_name: "Miner Bob".to_string(),
                original_entity: Entity::PLACEHOLDER,
            },
            pos,
        )).id();

        // Attempt to place building over grave
        let result = place_building(&mut world, BuildingType::Factory, pos);
        assert!(result.is_ok()); // The building should be allowed

        // The grave should be destroyed or marked overwritten
        assert!(world.get_entity(grave_entity).is_none());

        // Global sacrilege penalty event should be emitted
        let events = world.resource::<Events<SacrilegeEvent>>();
        let reader = events.get_reader();
        assert_eq!(reader.len(&events), 1);
    }
}
```

## 4. GREEN Phase: Minimal Implementation

```rust
// src/layer1/grave.rs

use bevy_ecs::prelude::*;
use crate::layer1::map::GridPosition;
use crate::layer1::relationships::Relationships;
use crate::layer1::needs::Needs;
use crate::layer1::building::BuildingType;

#[derive(Component)]
pub struct Grave {
    pub pop_name: String,
    pub original_entity: Entity,
}

pub struct SacrilegeEvent {
    pub position: GridPosition,
}

pub fn handle_pop_death(world: &mut World, pop_entity: Entity) {
    let pos = *world.get::<GridPosition>(pop_entity).unwrap();
    let name = world.get::<Name>(pop_entity).unwrap().as_str().to_string();

    // Despawn pop
    world.despawn(pop_entity);

    // Spawn Grave
    world.spawn((
        Grave { pop_name: name, original_entity: pop_entity },
        pos,
    ));
}

pub fn visit_grave_system(
    mut query: Query<(&GridPosition, &Relationships, &mut Needs), With<Pop>>,
    grave_query: Query<(&GridPosition, &Grave)>,
) {
    for (grave_pos, grave) in grave_query.iter() {
        for (pop_pos, relationships, mut needs) in query.iter_mut() {
            if pop_pos == grave_pos {
                // Check if they were related (simplification for MVP: just needs high affinity)
                if relationships.affinities.iter().any(|(e, aff)| *e == grave.original_entity && *aff >= 80) {
                    needs.leisure = (needs.leisure + 0.1).clamp(0.0, 1.0); // Restores some leisure/morale
                }
            }
        }
    }
}

pub fn place_building(world: &mut World, building: BuildingType, pos: GridPosition) -> Result<(), &'static str> {
    // Check for graves
    let mut grave_to_remove = None;
    let mut query = world.query::<(Entity, &GridPosition, &Grave)>();
    for (entity, g_pos, _) in query.iter(world) {
        if *g_pos == pos {
            grave_to_remove = Some(entity);
            break;
        }
    }

    if let Some(e) = grave_to_remove {
        world.despawn(e);
        world.resource_mut::<Events<SacrilegeEvent>>().send(SacrilegeEvent { position: pos });
    }

    // Standard building placement logic
    world.spawn((building, pos));
    Ok(())
}
```

## 5. REFACTOR Phase: Quality & Design

- The `handle_pop_death` logic should be integrated directly into the `death_system` established in `034` (Pop Health and Damage) to ensure unity in how pops die and cleanup happens.
- Ensure that destroying a grave for building triggers a colony-wide morale penalty via a `Sacrilege` modifier using `031` (Pop Morale).
- The definition of "relative" currently assumes checking the original entity ID inside the `Relationships` component. Since the pop is despawned, this requires care—`Relationships` might need to retain dead entity mappings or switch to an ID-based system rather than direct ECS entity references if entities are recycled.
- The `visit_grave_system` currently requires exact position overlap. A 1-tile radius check would be more forgiving for pathfinding.

## 6. Acceptance Criteria (Testable!)

- [ ] All tests in RED phase pass.
- [ ] `cargo test` returns 0 failures.
- [ ] `cargo clippy -- -D warnings` passes.
- [ ] Test coverage ≥85% for `src/layer1/grave.rs`.
- [ ] A grave correctly spawns on the tile when a Pop dies.
- [ ] Relatives visiting the grave receive a measurable mood/leisure boost.
- [ ] Building over a grave despawns it and emits a `SacrilegeEvent`.

## 7. Technical Guidance

- Graves do not physically block building placement in the system but the UI should warn the player about the severe penalty before allowing the placement.
- When expanding the `Relationships` component, ensure it accounts for tracking dead pops, perhaps using a `Uuid` instead of an ECS `Entity` if `Entity` reuse causes false positive relationships.

## 8. Questions

*Builder: add questions here if spec is unclear. Architect will address.*
*Architect:* See related specifications for design details. MVP implementation should follow standard conventions.
