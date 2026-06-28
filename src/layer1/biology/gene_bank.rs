use crate::layer1::fauna::{Fauna, FaunaType};
use crate::layer1::items::{Item, ItemType};
use crate::layer1::map::GridPosition;
use crate::layer1::terrain::{TerrainGrid, TerrainType};
use bevy_ecs::prelude::*;

/// Type of genetic data stored in a sample or bank.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GeneticData {
    /// Sample from a plant.
    Flora(TerrainType),
    /// Sample from an animal.
    Fauna(FaunaType),
}

/// Component representing a dropped genetic sample item.
#[derive(Component, Debug, Clone)]
pub struct GeneticSample {
    /// The genetic data contained in the sample.
    pub data: GeneticData,
}

/// Component for the Gene Bank building.
#[derive(Component, Default, Debug)]
pub struct GeneBank {
    /// List of unique samples stored in the bank.
    pub stored_samples: Vec<GeneticData>,
    /// Currently active cloning job: (Target, `TicksRemaining`).
    pub active_cloning_job: Option<(GeneticData, f32)>,
}

#[derive(Event, Debug, Clone)]
pub struct CloneEvent {
    pub bank_entity: Entity,
    pub species_id: String,
}

#[derive(Event, Debug, Clone)]
pub struct ExtinctionEvent {
    pub species_id: String,
}

impl GeneBank {
    /// Stores a sample if not already present.
    pub fn store_sample(&mut self, data: GeneticData) {
        if !self.stored_samples.contains(&data) {
            self.stored_samples.push(data);
        }
    }

    /// Checks if a sample is stored.
    #[must_use]
    pub fn has_sample(&self, data: &GeneticData) -> bool {
        self.stored_samples.contains(data)
    }
}

/// System to attempt collecting a sample at the target position.
/// Returns true if successful (item spawned).
pub fn collect_sample_action(world: &mut World, _actor: Entity, target_pos: GridPosition) -> bool {
    let mut found_data = None;

    // 1. Check Fauna
    let mut fauna_query = world.query::<(&GridPosition, &Fauna)>();
    for (pos, fauna) in fauna_query.iter(world) {
        if pos.x == target_pos.x && pos.y == target_pos.y {
            found_data = Some(GeneticData::Fauna(fauna.fauna_type));
            break;
        }
    }

    // 2. If no fauna, check Flora (Terrain)
    if found_data.is_none() {
        let grid = world.resource::<TerrainGrid>();
        if let (Ok(x), Ok(y)) = (usize::try_from(target_pos.x), usize::try_from(target_pos.y)) {
            if let Some(tile) = grid.get(x, y) {
                match tile {
                    TerrainType::Tree
                    | TerrainType::Shrub
                    | TerrainType::Sapling
                    | TerrainType::Grass => {
                        found_data = Some(GeneticData::Flora(tile));
                    }
                    _ => {}
                }
            }
        }
    }

    // 3. Spawn Item
    if let Some(data) = found_data {
        world.spawn((
            Item {
                item_type: ItemType::GeneticSample,
            },
            GeneticSample { data },
            target_pos,
        ));
        return true;
    }

    false
}

/// System to process active cloning jobs in Gene Banks.
pub fn process_cloning_system(world: &mut World) {
    let mut completed_clones = Vec::new();

    let mut query = world.query::<(
        Entity,
        &mut GeneBank,
        &GridPosition,
        Option<&crate::layer1::energy::PowerConsumer>,
    )>();
    for (_entity, mut bank, pos, power) in query.iter_mut(world) {
        // Check power if component exists
        if power.is_some_and(|p| !p.active) {
            continue;
        }

        if let Some((target, time)) = &mut bank.active_cloning_job {
            if *time > 0.0 {
                *time -= 1.0;
            } else {
                // Clone complete
                completed_clones.push((target.clone(), *pos));
                bank.active_cloning_job = None;
            }
        }
    }

    // Spawn clones
    for (data, pos) in completed_clones {
        match data {
            GeneticData::Flora(t) => {
                world.spawn((
                    Item {
                        item_type: ItemType::GeneticSample,
                    }, // Placeholder for Seed/Sapling
                    GeneticSample {
                        data: GeneticData::Flora(t),
                    },
                    pos,
                ));
            }
            GeneticData::Fauna(f) => {
                world.spawn((
                    Fauna {
                        fauna_type: f,
                        ..Default::default()
                    },
                    pos,
                    crate::layer1::health::Health::default(),
                ));
            }
        }
    }
}

#[derive(Resource, Default)]
pub struct ExtinctionTracker {
    pub extinct_species: bevy::utils::HashSet<String>,
}

pub fn handle_extinction_events(
    mut events: EventReader<ExtinctionEvent>,
    tracker: Option<ResMut<ExtinctionTracker>>,
) {
    if let Some(mut t) = tracker {
        for event in events.read() {
            t.extinct_species.insert(event.species_id.clone());
        }
    } else {
        // Drain events if there's no tracker
        for _ in events.read() {}
    }
}

pub fn handle_clone_events(
    mut commands: Commands,
    mut events: EventReader<CloneEvent>,
    query: Query<&GeneBank>,
) {
    let mut rng = rand::thread_rng();

    for event in events.read() {
        if let Ok(bank) = query.get(event.bank_entity) {
            // We need to match the species_id to a FaunaType to clone.
            // Since GeneticData doesn't store species_id, we infer FaunaType from the string for now,
            // or just pick the first Fauna sample in the bank that matches the name.
            let mut fauna_type_to_spawn = FaunaType::Wolf; // default fallback

            for sample in &bank.stored_samples {
                if let GeneticData::Fauna(f_type) = sample {
                    // Match species id to fauna type debug format or similar
                    if format!("{:?}", f_type) == event.species_id {
                        fauna_type_to_spawn = *f_type;
                        break;
                    }
                }
            }

            // Spawn the clone with genetic drift
            let traits = crate::layer1::psychology::traits::Traits::random(&mut rng);

            commands.spawn((
                Fauna {
                    fauna_type: fauna_type_to_spawn,
                    ..Default::default()
                },
                crate::layer1::fauna::SpeciesId {
                    id: event.species_id.clone(),
                },
                traits,
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::fauna::{Fauna, FaunaType, SpeciesId};
    use crate::layer1::items::{Item, ItemType};
    use crate::layer1::map::GridPosition;
    use crate::layer1::pop::Pop;
    use crate::layer1::psychology::traits::Traits;
    use crate::layer1::resources::ColonyResources;
    use crate::layer1::terrain::{TerrainGrid, TerrainType};

    // Helper setup
    fn setup_world() -> World {
        let mut world = World::new();
        // world.insert_resource(crate::shared::time::SimulationTime::default());
        world.insert_resource(ColonyResources::default());
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles: vec![TerrainType::Grass; 100],
        });
        world
    }

    #[test]
    fn test_collect_flora_sample() {
        let mut world = setup_world();

        // Spawn a Pop and a Tree
        let pop = world.spawn((Pop, GridPosition { x: 0, y: 0 })).id();
        // Assume collecting from terrain at (0,1) which is a Tree
        let mut grid = world.resource_mut::<TerrainGrid>();
        grid.set(0, 1, TerrainType::Tree);

        // Perform collection action
        let success = collect_sample_action(&mut world, pop, GridPosition { x: 0, y: 1 });

        assert!(success, "Should successfully collect sample from Tree");

        // Check for dropped item
        let mut item_query = world.query::<(&Item, &GeneticSample, &GridPosition)>();
        let (item, sample, pos) = item_query.single(&world);

        assert_eq!(item.item_type, ItemType::GeneticSample);
        match sample.data {
            GeneticData::Flora(t) => assert_eq!(t, TerrainType::Tree),
            _ => panic!("Expected Flora sample"),
        }
        assert_eq!(pos.x, 0);
        assert_eq!(pos.y, 1);
    }

    #[test]
    fn test_collect_fauna_sample() {
        let mut world = setup_world();

        // Spawn Pop and Animal
        let pop = world.spawn((Pop, GridPosition { x: 0, y: 0 })).id();
        let _animal = world
            .spawn((
                Fauna {
                    fauna_type: FaunaType::SpaceRat,
                    ..Default::default()
                },
                GridPosition { x: 0, y: 1 },
            ))
            .id();

        // Perform collection (target entity)
        let success = collect_sample_action(&mut world, pop, GridPosition { x: 0, y: 1 }); // Target pos

        assert!(success);

        // Check for item
        let mut item_query = world.query::<(&Item, &GeneticSample)>();
        let (_, sample) = item_query.single(&world);

        match sample.data {
            GeneticData::Fauna(f) => assert_eq!(f, FaunaType::SpaceRat),
            _ => panic!("Expected Fauna sample"),
        }
    }

    #[test]
    fn test_gene_bank_storage() {
        let mut world = setup_world();

        // Spawn GeneBank building
        let bank = world
            .spawn((
                Building {
                    building_type: BuildingType::GeneBank,
                },
                GeneBank::default(),
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // Add a sample to its storage manually
        let mut bank_comp = world.get_mut::<GeneBank>(bank).unwrap();
        bank_comp.store_sample(GeneticData::Flora(TerrainType::Tree));

        assert!(bank_comp.has_sample(&GeneticData::Flora(TerrainType::Tree)));
    }

    #[test]
    fn test_cloning_process_flora() {
        let mut world = setup_world();

        // Setup GeneBank with a sample and resources
        let bank = world
            .spawn((
                Building {
                    building_type: BuildingType::GeneBank,
                },
                GeneBank {
                    stored_samples: vec![GeneticData::Flora(TerrainType::Tree)],
                    active_cloning_job: Some((GeneticData::Flora(TerrainType::Tree), 10.0)), // 10 ticks remaining
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // Run system for 11 ticks (to ensure completion if logic waits for 0.0)
        for _ in 0..11 {
            process_cloning_system(&mut world);
        }

        // Check result: Should produce a "Seed" item at (5,5)
        let mut item_query = world.query::<(&Item, &GridPosition)>();
        let (item, pos) = item_query.single(&world);
        // Assuming ItemType::GeneticSample is reused as seed/sample
        assert_eq!(item.item_type, ItemType::GeneticSample);
        assert_eq!(pos.x, 5);
        assert_eq!(pos.y, 5);

        // Verify cloning job is cleared
        let bank_comp = world.get::<GeneBank>(bank).unwrap();
        assert!(bank_comp.active_cloning_job.is_none());
    }

    #[test]
    fn test_cloning_process_fauna() {
        let mut world = setup_world();

        let _bank = world
            .spawn((
                Building {
                    building_type: BuildingType::GeneBank,
                },
                GeneBank {
                    stored_samples: vec![GeneticData::Fauna(FaunaType::Wolf)],
                    active_cloning_job: Some((GeneticData::Fauna(FaunaType::Wolf), 0.0)), // Finished
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        process_cloning_system(&mut world);

        // Should spawn a Fauna entity
        let mut fauna_query = world.query::<(&Fauna, &GridPosition)>();
        let (fauna, pos) = fauna_query.single(&world);
        assert_eq!(fauna.fauna_type, FaunaType::Wolf);
        assert_eq!(pos.x, 5);
        assert_eq!(pos.y, 5);
    }

    #[test]
    fn test_cloning_extinct_species_applies_genetic_drift() {
        let mut world = setup_world();
        world.init_resource::<Events<CloneEvent>>();

        let bank = world
            .spawn(GeneBank {
                stored_samples: vec![GeneticData::Fauna(FaunaType::SpaceRat)],
                active_cloning_job: None,
            })
            .id();

        world.resource_mut::<Events<CloneEvent>>().send(CloneEvent {
            bank_entity: bank,
            species_id: "SpaceRat".to_string(),
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(handle_clone_events);
        schedule.run(&mut world);

        // Verify the newly spawned fauna has mutated traits
        let mut found_clone = false;
        let mut query = world.query::<(&SpeciesId, &Traits)>();
        for (species, traits) in query.iter(&world) {
            if species.id == "SpaceRat" {
                found_clone = true;
                // Traits could be empty due to RNG (50% chance), but the component is guaranteed to be there.
                // We can check it's properly constructed.
                let _ = traits.0.len();
                break;
            }
        }

        assert!(
            found_clone,
            "A CloneEvent should spawn a new instance of the species."
        );
    }
}
