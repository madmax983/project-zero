use crate::layer1::building::{try_place_building, BuildingType};
use crate::layer1::housing::Housing;
use crate::layer1::map::GridPosition;
use crate::layer1::morale::{MoodModifier, Morale};
use crate::layer1::pop::Pop;
use bevy_ecs::prelude::*;
use rand::{seq::SliceRandom, Rng};

/// Types of personal structures that Pops can build.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PersonalStructureType {
    /// A small storage shed.
    Shed,
    /// A decorative garden.
    Garden,
    /// A small religious shrine.
    Shrine,
}

/// Component attached to a building indicating it is a personal structure.
#[derive(Component)]
pub struct PersonalStructure {
    /// The owner of this structure.
    pub owner: Entity,
    /// The type of structure.
    pub structure_type: PersonalStructureType,
    /// The beauty bonus provided by this structure.
    pub beauty_bonus: f32,
}

/// Component attached to a Pop indicating they own a personal structure.
#[derive(Component)]
pub struct OwnsStructure(pub Entity);

/// System that allows idle pops to build personal structures adjacent to their home.
pub fn check_spontaneous_build_system(world: &mut World) {
    // 1. Identify potential builders
    // We need to collect (PopEntity, HousingPos) pairs to avoid keeping borrows on World
    let mut builders = Vec::new();

    {
        let mut housing_query = world.query::<(&GridPosition, &Housing)>();
        // We can't query Pop inside the loop easily if we borrow world for housing_query.
        // But housing.residents has Entity IDs.

        for (pos, housing) in housing_query.iter(world) {
            for &resident in &housing.residents {
                // Check if resident exists and doesn't own a structure
                if world.get::<Pop>(resident).is_some()
                    && world.get::<OwnsStructure>(resident).is_none()
                {
                    // 1% chance per tick (simulated here with random)
                    // In tests we might force this.
                    let mut rng = rand::thread_rng();
                    if rng.gen_bool(0.01) {
                        builders.push((resident, *pos));
                    }
                }
            }
        }
    }

    // 2. Process builders
    for (pop_entity, home_pos) in builders {
        // Find a valid spot
        let mut target_pos = None;
        let mut neighbors = [
            (0, 1),
            (0, -1),
            (1, 0),
            (-1, 0),
            (1, 1),
            (1, -1),
            (-1, 1),
            (-1, -1),
        ];

        let mut rng = rand::thread_rng();
        neighbors.shuffle(&mut rng);

        for (dx, dy) in neighbors {
            let nx = home_pos.x + dx;
            let ny = home_pos.y + dy;

            // Validate spot using `can_place_building` from building.rs
            if crate::layer1::building::can_place_building(world, nx, ny) {
                target_pos = Some((nx, ny));
                break;
            }
        }

        if let Some((x, y)) = target_pos {
            // Pick random type
            let mut rng = rand::thread_rng();
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

            // Attempt to build
            if try_place_building(world, x, y, building_type) {
                // Find the new entity
                // Since we just placed it at (x, y), we can query for it.
                // We need to find the entity with GridPosition(x,y) and Building component.
                let mut found_structure = None;

                // New scope for query
                {
                    let mut q = world
                        .query::<(Entity, &GridPosition, &crate::layer1::building::Building)>();
                    for (e, p, _) in q.iter(world) {
                        if p.x == x && p.y == y {
                            found_structure = Some(e);
                            break;
                        }
                    }
                }

                if let Some(structure_entity) = found_structure {
                    // Attach components
                    let beauty_bonus = match structure_type {
                        PersonalStructureType::Shed => 0.0,
                        PersonalStructureType::Garden => 5.0,
                        PersonalStructureType::Shrine => 2.0,
                    };

                    world
                        .entity_mut(structure_entity)
                        .insert(PersonalStructure {
                            owner: pop_entity,
                            structure_type,
                            beauty_bonus,
                        });

                    world
                        .entity_mut(pop_entity)
                        .insert(OwnsStructure(structure_entity));
                }
            }
        }
    }
}

/// System that checks if a personal structure has been destroyed and applies morale penalties.
pub fn demolish_personal_structure_system(world: &mut World) {
    // Exclusive system version for tests/simplicity if allowed,
    // or just run the logic manually using world access.
    // The spec example used `&mut World`.

    let mut upgrades = Vec::new();

    // 1. Identify pops who lost their structure
    {
        let mut pop_query = world.query::<(Entity, &mut Morale, &OwnsStructure)>();
        // We also need to check if the structure entity exists.
        // accessing world inside query iter is tricky.
        // We collect first.

        let mut check_list = Vec::new();
        for (e, _, owns) in pop_query.iter(world) {
            check_list.push((e, owns.0));
        }

        for (pop_e, struct_e) in check_list {
            if world.get_entity(struct_e).is_err() {
                upgrades.push(pop_e);
            }
        }
    }

    // 2. Apply penalties
    for pop_e in upgrades {
        if let Some(mut morale) = world.get_mut::<Morale>(pop_e) {
            morale.modifiers.push(MoodModifier {
                label: "Personal Structure Demolished".to_string(),
                value: -0.2,
                duration: 100,
            });
        }
        world.entity_mut(pop_e).remove::<OwnsStructure>();
    }
}

#[cfg(test)]
mod tests {
    use crate::layer1::building::{Building, BuildingType, OccupiedTiles};
    use crate::layer1::housing::Housing;
    use crate::layer1::map::GridPosition;
    use crate::layer1::morale::Morale;
    use crate::layer1::pop::Pop;
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::spontaneous_architecture::{
        check_spontaneous_build_system, demolish_personal_structure_system, OwnsStructure,
        PersonalStructure, PersonalStructureType,
    };
    use crate::layer1::terrain::{TerrainGrid, TerrainType};
    use bevy_ecs::prelude::*;

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

        // Setup Pop with Home
        let home_pos = GridPosition { x: 5, y: 5 };
        let pop = world
            .spawn((
                Pop,
                // GridPosition { x: 5, y: 5 }, // At home (not strictly needed for logic, but good for context)
                // Idle state would be checked by system logic, here we test the outcome
            ))
            .id();

        let _home = world
            .spawn((
                Building {
                    building_type: BuildingType::Housing,
                },
                Housing {
                    capacity: 1,
                    residents: vec![pop],
                },
                home_pos,
            ))
            .id();

        // Run system multiple times to overcome RNG or mock RNG (since we used gen_bool(0.01))

        let mut built = false;
        for _ in 0..1000 {
            check_spontaneous_build_system(&mut world);
            if world.query::<&PersonalStructure>().iter(&world).count() > 0 {
                built = true;
                break;
            }
        }
        assert!(built, "Should have built a structure eventually");

        // Assert: A personal structure should spawn adjacent to (5,5)
        let structures: Vec<_> = world
            .query::<(&PersonalStructure, &GridPosition)>()
            .iter(&world)
            .collect();
        assert_eq!(structures.len(), 1);

        let (_, pos) = structures[0];
        let dx = pos.x.abs_diff(home_pos.x).min(i32::MAX as u32) as i32;
        let dy = pos.y.abs_diff(home_pos.y).min(i32::MAX as u32) as i32;
        assert!(dx <= 1 && dy <= 1 && (dx + dy) > 0, "Must be adjacent");

        // Assert: Tile occupied
        let occupied = world.resource::<OccupiedTiles>();
        assert!(occupied.0.contains(&(pos.x, pos.y)));
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
        world.insert_resource(ColonyResources {
            wood: 1000.0,
            stone: 1000.0,
            ..Default::default()
        });

        let pop = world.spawn(Pop).id();
        let _home = world
            .spawn((
                Building {
                    building_type: BuildingType::Housing,
                },
                Housing {
                    capacity: 1,
                    residents: vec![pop],
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // Loop until build
        let mut built = false;
        for _ in 0..1000 {
            check_spontaneous_build_system(&mut world);
            if world.query::<&PersonalStructure>().iter(&world).count() > 0 {
                built = true;
                break;
            }
        }
        assert!(built);

        let res = world.resource::<ColonyResources>();
        // Costs are 10 or 5.
        assert!(
            res.wood < 1000.0 || res.stone < 1000.0,
            "Should consume resources"
        );
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
                Building {
                    building_type: BuildingType::PersonalGarden,
                }, // Add building component to make it "real"
                PersonalStructure {
                    owner,
                    structure_type: PersonalStructureType::Garden,
                    beauty_bonus: 5.0,
                },
            ))
            .id();

        // Attach ownership
        world.entity_mut(owner).insert(OwnsStructure(structure));

        // Despawn the structure (simulating demolition)
        world.despawn(structure);

        // Run reaction system
        demolish_personal_structure_system(&mut world);

        // Assert: Morale dropped
        let morale = world.get::<Morale>(owner).unwrap();
        // Base is 0.8. Penalty is -0.2. So should be 0.6 + modifiers sum.
        // Wait, morale.value is updated by `update_morale_cache_system`.
        // We haven't run that. But modifiers list should contain the entry.
        assert!(!morale.modifiers.is_empty());
        assert_eq!(morale.modifiers[0].label, "Personal Structure Demolished");
        assert_eq!(morale.modifiers[0].value, -0.2);
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
            stone: 100.0,
            ..Default::default()
        });

        let pop = world.spawn(Pop).id();
        let _home = world
            .spawn((
                Building {
                    building_type: BuildingType::Housing,
                },
                Housing {
                    capacity: 1,
                    residents: vec![pop],
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        for _ in 0..100 {
            check_spontaneous_build_system(&mut world);
        }

        let count = world.query::<&PersonalStructure>().iter(&world).count();
        assert_eq!(count, 0, "Should not build if blocked");
    }
}
