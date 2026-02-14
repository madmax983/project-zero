//! Spontaneous Architecture system (Spec 110).
//!
//! Idle pops build personal structures (Sheds, Gardens, Shrines) adjacent to their homes.
//! These structures provide happiness but block efficient planning.

use crate::layer1::building::{
    Building, BuildingType, MaterialType, OccupiedTiles, can_place_building, spawn_building,
};
use crate::layer1::housing::Housing;
use crate::layer1::map::GridPosition;
use crate::layer1::morale::{MoodModifier, Morale};
use crate::layer1::resources::ColonyResources;
use crate::shared::log::MessageLog;
use bevy_ecs::prelude::*;
use rand::Rng;
use rand::seq::SliceRandom;

/// Types of personal structures a Pop can build.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub enum PersonalStructureType {
    /// A small shed for hobbies or storage.
    Shed,
    /// A decorative garden.
    Garden,
    /// A small shrine for contemplation.
    Shrine,
}

impl PersonalStructureType {
    /// Returns the corresponding `BuildingType`.
    #[must_use]
    pub const fn as_building_type(&self) -> BuildingType {
        match self {
            Self::Shed => BuildingType::PersonalShed,
            Self::Garden => BuildingType::PersonalGarden,
            Self::Shrine => BuildingType::PersonalShrine,
        }
    }

    /// Choose a random type.
    #[must_use]
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        match rng.gen_range(0..3) {
            0 => Self::Shed,
            1 => Self::Garden,
            _ => Self::Shrine,
        }
    }
}

/// Component defining a personal structure.
#[derive(Component)]
pub struct PersonalStructure {
    /// The Entity ID of the Pop that owns this structure.
    pub owner: Entity,
    /// The type of structure.
    pub structure_type: PersonalStructureType,
    /// The beauty bonus this structure provides to the owner.
    pub beauty_bonus: f32,
}

/// Component on a Pop that tracks their personal structure.
#[derive(Component)]
pub struct ClaimedStructure(pub Entity);

/// System to check for idle pops and spawn personal structures.
pub fn check_spontaneous_build_system(world: &mut World) {
    // 1. Identify candidates: Pops who are residents of housing and do NOT have a ClaimedStructure.

    let mut potential_builds = Vec::new();

    // Iterate all Housing buildings to find residents
    let mut housing_query = world.query::<(&GridPosition, &Housing)>();
    for (home_pos, housing) in housing_query.iter(world) {
        for &resident in &housing.residents {
            // Check if resident is valid (exists)
            if world.get_entity(resident).is_ok() {
                // Check if resident already has a structure
                if world.get::<ClaimedStructure>(resident).is_none() {
                    // Check if resident is actually "Idle" (Optional for MVP/Green, assume yes or low chance)
                    // For MVP, we assume 100% chance for eligible pops to try building,
                    // limited by resources and space.
                    potential_builds.push((resident, *home_pos));
                }
            }
        }
    }

    // 2. Attempt to build for each candidate
    for (pop_entity, home_pos) in potential_builds {
        attempt_build_for_pop(world, pop_entity, home_pos);
    }
}

fn attempt_build_for_pop(world: &mut World, pop: Entity, home_pos: GridPosition) {
    // 1. Pick a type
    let structure_type = PersonalStructureType::random();
    let building_type = structure_type.as_building_type();

    // 2. Check cost
    let cost = building_type.cost(MaterialType::default());

    // 3. Find adjacent empty tile
    let mut rng = rand::thread_rng();
    let mut offsets = vec![
        (-1, -1),
        (0, -1),
        (1, -1),
        (-1, 0),
        (1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
    ];
    // Shuffle offsets
    offsets.shuffle(&mut rng);

    for (dx, dy) in offsets {
        let target_x = home_pos.x + dx;
        let target_y = home_pos.y + dy;

        // Check if we can place
        if can_place_building(world, target_x, target_y) {
            // Deduct resources
            let can_afford = world.resource_mut::<ColonyResources>().try_deduct(&cost);
            if !can_afford {
                // Can't afford, stop trying for this pop
                return;
            }

            // Mark occupied
            world
                .resource_mut::<OccupiedTiles>()
                .0
                .insert((target_x, target_y));

            // Spawn Building
            let material = MaterialType::default();
            spawn_building(world, target_x, target_y, building_type, material);

            // Find the entity we just spawned to attach PersonalStructure
            let spawned_entity = find_building_at(world, target_x, target_y);

            if let Some(building_entity) = spawned_entity {
                // Attach PersonalStructure
                world.entity_mut(building_entity).insert(PersonalStructure {
                    owner: pop,
                    structure_type,
                    beauty_bonus: building_type.beauty_value(),
                });

                // Attach ClaimedStructure to Pop
                world
                    .entity_mut(pop)
                    .insert(ClaimedStructure(building_entity));

                if let Some(mut log) = world.get_resource_mut::<MessageLog>() {
                    log.add(format!("A pop built a {}", building_type.label()));
                }
            }

            // Only build one
            return;
        }
    }
}

fn find_building_at(world: &mut World, x: i32, y: i32) -> Option<Entity> {
    let mut query = world.query::<(Entity, &GridPosition, &Building)>();
    for (entity, pos, _) in query.iter(world) {
        if pos.x == x && pos.y == y {
            return Some(entity);
        }
    }
    None
}

/// System to detect demolished structures and apply morale penalties.
pub fn demolish_personal_structure_system(world: &mut World) {
    // Collect candidates first to avoid borrow conflicts
    let mut to_check = Vec::new();

    let mut query = world.query::<(Entity, &ClaimedStructure)>();
    for (pop_entity, claimed) in query.iter(world) {
        to_check.push((pop_entity, claimed.0));
    }

    // Check existence and punish
    for (pop_entity, structure_entity) in to_check {
        // If get_entity returns Err, the entity is despawned.
        if world.get_entity(structure_entity).is_err() {
            // Structure is gone!
            if let Some(mut morale) = world.get_mut::<Morale>(pop_entity) {
                morale.modifiers.push(MoodModifier {
                    label: "Personal Structure Demolished".to_string(),
                    value: -0.2,
                    duration: 100,
                });
            }

            // Remove component
            world.entity_mut(pop_entity).remove::<ClaimedStructure>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::{Building, BuildingType, OccupiedTiles};
    use crate::layer1::map::GridPosition;
    use crate::layer1::morale::Morale;
    use crate::layer1::pop::Pop;
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};

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
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });
        world.insert_resource(OccupiedTiles::default());
        world.insert_resource(ColonyResources {
            wood: 100.0,
            stone: 100.0,
            ..Default::default()
        });
        // Need BuildMode for try_place_building if used, or defaults.
        // And TechState if needed. We'll rely on default behavior or mocks if possible.

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

        // Run system
        // We mock the "Idle" check by ensuring system only runs on idle pops or forcing it
        // For unit test, we call the logic directly or setup the conditions
        check_spontaneous_build_system(&mut world);

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

        // Assert: Pop has ClaimedStructure
        assert!(world.get::<ClaimedStructure>(pop).is_some());
    }

    #[test]
    fn test_spontaneous_build_consumes_resources() {
        let mut world = World::new();
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });
        world.insert_resource(OccupiedTiles::default());
        // Provide enough of both resources since choice is random
        world.insert_resource(ColonyResources {
            wood: 100.0,
            stone: 100.0,
            ..Default::default()
        });

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
        if let Some(mut housing) = world.get_mut::<Housing>(home) {
            housing.residents.push(pop);
        }

        check_spontaneous_build_system(&mut world);

        let res = world.resource::<ColonyResources>();
        let consumed = res.wood < 100.0 || res.stone < 100.0;
        assert!(consumed, "Should consume resources (wood or stone)");
    }

    #[test]
    fn test_demolish_causes_sadness() {
        let mut world = World::new();

        let owner = world
            .spawn((
                Pop,
                Morale {
                    value: 0.8,
                    modifiers: vec![],
                },
            ))
            .id();

        let structure = world
            .spawn((
                PersonalStructure {
                    owner,
                    structure_type: PersonalStructureType::Garden,
                    beauty_bonus: 5.0,
                },
                // Simulate "Being Demolished" - usually handled by a command or event
                // Here we test the reaction system that watches for removal
            ))
            .id();

        // Add ClaimedStructure to Pop so the system can track it
        world.entity_mut(owner).insert(ClaimedStructure(structure));

        // Despawn the structure (simulating demolition)
        world.despawn(structure);

        // Run reaction system
        demolish_personal_structure_system(&mut world);

        // Assert: Morale dropped
        let morale = world.get::<Morale>(owner).unwrap();

        // Morale value is cached by a system, so we check the modifiers directly
        assert!(!morale.modifiers.is_empty());
        assert_eq!(morale.modifiers[0].value, -0.2);

        // Assert: ClaimedStructure removed
        assert!(world.get::<ClaimedStructure>(owner).is_none());
    }

    #[test]
    fn test_cannot_build_if_blocked() {
        let mut world = World::new();
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
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
