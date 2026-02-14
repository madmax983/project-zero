use crate::layer1::building::{BuildingType, OccupiedTiles, try_place_building};
use crate::layer1::housing::Housing;
use crate::layer1::map::GridPosition;
use crate::layer1::morale::Morale;
use crate::layer1::terrain::{TerrainGrid, TerrainType};
use crate::layer1::utility_types::{ActionType, PopAction};
use bevy_ecs::prelude::*;
use rand::Rng;

/// Types of personal structures a pop can build.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PersonalStructureType {
    /// A small shed for tools or hobbies.
    Shed,
    /// A small garden for relaxation.
    Garden,
    /// A small shrine for contemplation.
    Shrine,
}

/// Component attached to a personal structure entity.
#[derive(Component)]
pub struct PersonalStructure {
    /// The owner of this structure.
    pub owner: Entity,
    /// The type of structure.
    pub structure_type: PersonalStructureType,
    /// The beauty value added by this structure.
    pub beauty_bonus: f32,
}

/// Component attached to a Pop that owns a personal structure.
#[derive(Component)]
pub struct ClaimedPersonalStructure(pub Entity);

/// System that allows idle pops to build personal structures near their home.
pub fn check_spontaneous_build_system(world: &mut World) {
    // 1. Find potential builders (Pop with Housing, no existing structure)
    // We collect entities first to avoid borrow checker issues when we need mutable world later
    let mut candidates = Vec::new();

    let mut query = world.query_filtered::<(
        Entity,
        &GridPosition,
        &Housing,
        Option<&PopAction>,
    ), Without<ClaimedPersonalStructure>>();
    for (entity, pos, _, action) in query.iter(world) {
        // Only idle pops build
        if action.is_some_and(|act| act.current != ActionType::Idle) {
            continue;
        }
        candidates.push((entity, *pos));
    }

    if candidates.is_empty() {
        return;
    }

    // 2. Pick one random candidate (to avoid mass construction in one tick)
    let mut rng = rand::thread_rng();
    let idx = rng.gen_range(0..candidates.len());
    let (pop_entity, pop_pos) = candidates[idx];

    // 3. Find adjacent empty tile
    let mut potential_tiles = Vec::new();
    let neighbors = [
        (0, 1),
        (0, -1),
        (1, 0),
        (-1, 0),
        (1, 1),
        (1, -1),
        (-1, 1),
        (-1, -1),
    ];

    {
        let occupied = world.resource::<OccupiedTiles>();
        let terrain = world.resource::<TerrainGrid>();

        for (dx, dy) in neighbors {
            let nx = pop_pos.x + dx;
            let ny = pop_pos.y + dy;

            if nx >= 0 && ny >= 0 {
                #[allow(clippy::cast_sign_loss)]
                let (ux, uy) = (nx as usize, ny as usize);

                if ux < terrain.width
                    && uy < terrain.height
                    && !occupied.0.contains(&(nx, ny))
                    && terrain
                        .get(ux, uy)
                        .is_some_and(|tile| matches!(tile, TerrainType::Grass | TerrainType::Dirt))
                {
                    potential_tiles.push((nx, ny));
                }
            }
        }
    }

    if potential_tiles.is_empty() {
        return;
    }

    let tile_idx = rng.gen_range(0..potential_tiles.len());
    let (tx, ty) = potential_tiles[tile_idx];

    // 4. Pick random structure type
    let structure_type = match rng.gen_range(0..3) {
        0 => PersonalStructureType::Shed,
        1 => PersonalStructureType::Garden,
        _ => PersonalStructureType::Shrine,
    };

    let building_type = match structure_type {
        PersonalStructureType::Shed => BuildingType::PersonalShed,
        PersonalStructureType::Garden => BuildingType::PersonalGarden,
        PersonalStructureType::Shrine => BuildingType::PersonalShrine,
    };

    // 5. Try to build
    // try_place_building handles resource checks and deduction
    if let Some(structure_entity) = try_place_building(world, tx, ty, building_type) {
        // 6. Attach PersonalStructure component
        let beauty = match structure_type {
            PersonalStructureType::Garden => 5.0,
            PersonalStructureType::Shrine => 2.0,
            PersonalStructureType::Shed => 0.0,
        };

        world
            .entity_mut(structure_entity)
            .insert(PersonalStructure {
                owner: pop_entity,
                structure_type,
                beauty_bonus: beauty,
            });

        // 7. Mark pop as owner
        world
            .entity_mut(pop_entity)
            .insert(ClaimedPersonalStructure(structure_entity));
    }
}

/// System that checks if a personal structure has been destroyed and penalizes the owner.
pub fn demolish_personal_structure_system(world: &mut World) {
    // Check all pops with claims
    let mut broken_hearts = Vec::new();

    let mut query = world.query::<(Entity, &ClaimedPersonalStructure)>();
    for (pop_entity, claim) in query.iter(world) {
        if world.get_entity(claim.0).is_err() {
            // Structure is gone!
            broken_hearts.push(pop_entity);
        }
    }

    for pop_entity in broken_hearts {
        // Remove claim
        world
            .entity_mut(pop_entity)
            .remove::<ClaimedPersonalStructure>();

        // Apply sadness
        if let Some(mut morale) = world.get_mut::<Morale>(pop_entity) {
            morale.add_modifier("Personal Structure Destroyed", -0.2, 100);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::{Building, BuildingType, OccupiedTiles};
    use crate::layer1::housing::Housing;
    use crate::layer1::map::GridPosition;
    use crate::layer1::morale::Morale;
    use crate::layer1::pop::Pop;
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::terrain::TerrainGrid;
    use bevy_ecs::prelude::*;
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
        let tiles = vec![crate::layer1::terrain::TerrainType::Grass; 100];
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(OccupiedTiles::default());
        world.insert_resource(ColonyResources {
            wood: 100.0,
            stone: 100.0,
            ..Default::default()
        });
        // Setup BuildMode resource required by try_place_building
        world.insert_resource(crate::layer1::BuildMode::default());
        // Setup TechState
        world.insert_resource(crate::layer1::tech::TechState::default());
        // Setup MessageLog
        world.insert_resource(crate::shared::log::MessageLog::default());

        // Setup Pop with Home
        let home_pos = GridPosition { x: 5, y: 5 };
        let home = world
            .spawn((
                Building {
                    building_type: BuildingType::Housing,
                },
                Housing {
                    capacity: 1,
                    residents: vec![],
                }, // Will add resident below
                home_pos,
            ))
            .id();

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 }, // At home
                                             // Idle state would be checked by system logic, here we test the outcome
            ))
            .id();

        // Manually link pop to home (simulating residence)
        world.get_mut::<Housing>(home).unwrap().residents.push(pop);

        // Run system multiple times to ensure RNG hits
        for _ in 0..10 {
            check_spontaneous_build_system(&mut world);
            let count = world.query::<&PersonalStructure>().iter(&world).count();
            if count > 0 {
                break;
            }
        }

        // Assert: A personal structure should spawn adjacent to (5,5)
        let structures: Vec<_> = world
            .query::<(&PersonalStructure, &GridPosition)>()
            .iter(&world)
            .collect();
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
        let tiles = vec![crate::layer1::terrain::TerrainType::Grass; 100];
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(OccupiedTiles::default());
        world.insert_resource(ColonyResources {
            wood: 10.0,
            stone: 10.0,
            ..Default::default()
        }); // Exact cost for Shed/Garden/Shrine
        world.insert_resource(crate::layer1::BuildMode::default());
        world.insert_resource(crate::layer1::tech::TechState::default());
        world.insert_resource(crate::shared::log::MessageLog::default());

        let home = world
            .spawn((
                Building {
                    building_type: BuildingType::Housing,
                },
                Housing {
                    capacity: 1,
                    residents: vec![],
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        let pop = world.spawn(Pop).id();
        world.get_mut::<Housing>(home).unwrap().residents.push(pop);

        // Force build
        for _ in 0..10 {
            check_spontaneous_build_system(&mut world);
            if world.query::<&PersonalStructure>().iter(&world).count() > 0 {
                break;
            }
        }

        let res = world.resource::<ColonyResources>();
        // Either wood or stone should be consumed
        assert!(
            res.wood < 10.0 || res.stone < 10.0,
            "Should consume resources"
        );
    }

    #[test]
    fn test_demolish_causes_sadness() {
        let mut world = World::new();

        let structure_entity = world.spawn_empty().id(); // Placeholder structure

        let owner = world
            .spawn((
                Pop,
                Morale {
                    value: 0.8,
                    modifiers: vec![],
                },
                crate::layer1::needs::Needs::default(),
                ClaimedPersonalStructure(structure_entity),
            ))
            .id();

        // Despawn the structure (simulating demolition)
        world.despawn(structure_entity);

        // Run reaction system
        demolish_personal_structure_system(&mut world);

        // Update morale cache to reflect changes
        let mut schedule = Schedule::default();
        schedule.add_systems(crate::layer1::morale::update_morale_cache_system);
        schedule.run(&mut world);

        // Assert: Morale dropped
        let morale = world.get::<Morale>(owner).unwrap();
        assert!(morale.value < 0.8);
        assert!(world.get::<ClaimedPersonalStructure>(owner).is_none());
    }

    #[test]
    fn test_cannot_build_if_blocked() {
        let mut world = World::new();
        let tiles = vec![crate::layer1::terrain::TerrainType::Grass; 100];
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
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
        world.insert_resource(ColonyResources {
            wood: 100.0,
            ..Default::default()
        });
        world.insert_resource(crate::layer1::BuildMode::default());
        world.insert_resource(crate::layer1::tech::TechState::default());
        world.insert_resource(crate::shared::log::MessageLog::default());

        let home = world
            .spawn((
                Building {
                    building_type: BuildingType::Housing,
                },
                Housing {
                    capacity: 1,
                    residents: vec![],
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();
        let pop = world.spawn(Pop).id();
        world.get_mut::<Housing>(home).unwrap().residents.push(pop);

        check_spontaneous_build_system(&mut world);

        let count = world.query::<&PersonalStructure>().iter(&world).count();
        assert_eq!(count, 0, "Should not build if blocked");
    }
}
