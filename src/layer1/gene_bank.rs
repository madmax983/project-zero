//! Gene Bank system.
//!
//! Handles storage of genetic samples and cloning of flora/fauna.

use bevy_ecs::prelude::*;
use crate::layer1::terrain::TerrainType;
use crate::layer1::fauna::{Fauna, FaunaType};
use crate::layer1::items::{Item, ItemType};
use crate::layer1::map::GridPosition;
use crate::layer1::utility_eval_types::PositionProxy;
use crate::layer1::utility_types::{UtilityWeights, calculate_context_score};
use crate::layer1::designation::{Designation, DesignationType};
use crate::layer1::terrain::TerrainGrid;
use crate::layer1::flora::Flora;
use crate::layer1::utility_types::ActionType;
use crate::layer1::execution::{MovementTarget, AtTarget};
use crate::layer1::utility_types::PopAction;

/// Genetic data stored in samples or banks.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GeneticData {
    /// Flora genetic data.
    Flora(TerrainType),
    /// Fauna genetic data.
    Fauna(FaunaType),
}

/// Item component containing genetic data.
#[derive(Component, Debug, Clone)]
pub struct GeneticSample {
    /// The genetic payload.
    pub data: GeneticData,
}

/// Gene Bank building component.
#[derive(Component, Default, Debug)]
pub struct GeneBank {
    /// Stored samples in this bank.
    pub stored_samples: Vec<GeneticData>,
    /// Currently active cloning job: (Target, TicksRemaining).
    pub active_cloning_job: Option<(GeneticData, f32)>,
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

/// Evaluates the utility of collecting a genetic sample.
#[must_use]
pub fn evaluate_collect_sample(
    pop_pos: &GridPosition,
    weights: &UtilityWeights,
    designations: &[PositionProxy],
) -> Option<(f32, Entity)> {
    let mut best_score = 0.0;
    let mut best_target = None;

    for proxy in designations {
        let score = calculate_context_score(*pop_pos, Some(proxy.pos), 1, 0, weights);
        if score > best_score {
            best_score = score;
            best_target = Some(proxy.entity);
        }
    }

    // Base utility 0.6 (similar to other work)
    if best_score > 0.0 {
        Some((best_score * 0.6, best_target?))
    } else {
        None
    }
}

/// Performs the collection action. Returns true if successful.
#[allow(clippy::cast_sign_loss)]
pub fn collect_sample_action(world: &mut World, _actor: Entity, target_pos: GridPosition) -> bool {
    let mut found_data = None;

    // 1. Check for Fauna at position
    let mut fauna_query = world.query::<(Entity, &GridPosition, &Fauna)>();
    for (_, pos, fauna) in fauna_query.iter(world) {
        if *pos == target_pos {
            found_data = Some(GeneticData::Fauna(fauna.fauna_type));
            break;
        }
    }

    // 2. If no fauna, check Flora
    if found_data.is_none() {
        let mut flora_query = world.query::<(Entity, &GridPosition, &Flora)>();
        for (_, pos, _) in flora_query.iter(world) {
            if *pos == target_pos {
                break; // Proceed to terrain check
            }
        }

        // Check Terrain
        let grid = world.resource::<TerrainGrid>();
        if let Some(tile) = grid.get(target_pos.x as usize, target_pos.y as usize) {
            match tile {
                TerrainType::Tree | TerrainType::Shrub | TerrainType::Sapling | TerrainType::Grass => {
                    found_data = Some(GeneticData::Flora(tile));
                },
                _ => {}
            }
        }
    }

    // 3. Spawn Item
    if let Some(data) = found_data {
        world.spawn((
            Item,
            ItemType::GeneticSample,
            GeneticSample { data },
            target_pos
        ));

        // Visual effect
        crate::layer1::particles::spawn_particle(
            world,
            target_pos,
            '🧬',
            ratatui::style::Color::Magenta,
            20,
        );

        if let Some(mut log) = world.get_resource_mut::<crate::shared::log::MessageLog>() {
            log.add("Genetic Sample collected.");
        }

        return true;
    }

    false
}

/// System to execute the collection action when a pop arrives at the target.
pub fn collect_sample_execution_system(world: &mut World) {
    // Find pops ready to collect
    let mut candidates = Vec::new();

    let mut query = world.query_filtered::<(Entity, &MovementTarget), With<AtTarget>>();
    for (entity, mt) in query.iter(world) {
        if mt.for_action == ActionType::CollectSample {
            candidates.push((entity, mt.target_entity, mt.target_position));
        }
    }

    for (pop_entity, des_entity, target_pos) in candidates {
        // Verify designation exists
        if world.get::<Designation>(des_entity).is_none() {
            // Designation gone, abort
            cleanup_pop(world, pop_entity);
            continue;
        }

        // Perform action
        let success = collect_sample_action(world, pop_entity, target_pos);

        if !success {
            // Failed (target moved or invalid)
            if let Some(mut log) = world.get_resource_mut::<crate::shared::log::MessageLog>() {
                log.add("Failed to collect sample: Target invalid.");
            }
        }
        // Remove designation
        world.despawn(des_entity);

        // Clean up pop state
        cleanup_pop(world, pop_entity);
    }
}

fn cleanup_pop(world: &mut World, pop_entity: Entity) {
    world.entity_mut(pop_entity)
        .remove::<MovementTarget>()
        .remove::<AtTarget>();

    if let Some(mut action) = world.get_mut::<PopAction>(pop_entity) {
        action.current = ActionType::Idle;
        action.current_utility = 0.0;
        action.ticks_committed = 0;
    }
}

/// System to process ongoing cloning jobs.
pub fn process_cloning_system(world: &mut World) {
    let mut completed_clones = Vec::new();

    // 1. Update timers
    let mut query = world.query::<(Entity, &mut GeneBank, &GridPosition)>();
    for (_entity, mut bank, pos) in query.iter_mut(world) {
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

    // 2. Spawn clones and Log
    for (data, pos) in completed_clones {
        match data {
            GeneticData::Flora(t) => {
                world.spawn((
                    Item,
                    ItemType::GeneticSample,
                    GeneticSample { data: GeneticData::Flora(t) },
                    pos
                ));
            },
            GeneticData::Fauna(f) => {
                world.spawn((
                    Fauna { fauna_type: f, ..Default::default() },
                    pos
                ));
            }
        }

        crate::layer1::particles::spawn_particle(
            world,
            pos,
            '✨',
            ratatui::style::Color::Cyan,
            20,
        );

        // Notify
        if let Some(mut log) = world.get_resource_mut::<crate::shared::log::MessageLog>() {
            log.add("Cloning complete!");
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use super::*;
    use crate::layer1::items::{Item, ItemType};
    use crate::layer1::fauna::{Fauna, FaunaType};
    use crate::layer1::terrain::{TerrainType, TerrainGrid};
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::pop::Pop;
    use crate::layer1::map::GridPosition;
    use crate::layer1::resources::ColonyResources;

    // Helper setup
    fn setup_world() -> World {
        crate::setup::init_task_pools(); // Ensure task pools are init
        let mut world = World::new();
        world.insert_resource(crate::shared::time::SimulationTime::default());
        world.insert_resource(ColonyResources::default());
        world.insert_resource(TerrainGrid { width: 10, height: 10, tiles: vec![TerrainType::Grass; 100] });
        world.insert_resource(crate::shared::log::MessageLog::default());
        return world;
    }

    #[test]
    fn test_collect_flora_sample() {
        let mut world = setup_world();

        // Spawn a Pop and a Tree
        let pop = world.spawn((Pop, GridPosition { x: 0, y: 0 })).id();
        // Assume collecting from terrain at (0,1) which is a Tree
        let mut grid = world.resource_mut::<TerrainGrid>();
        grid.set(0, 1, TerrainType::Tree);

        // Perform collection action (mocking the action execution)
        let success = collect_sample_action(&mut world, pop, GridPosition { x: 0, y: 1 });

        assert!(success, "Should successfully collect sample from Tree");

        // Check for dropped item
        let mut item_query = world.query::<(&ItemType, &GeneticSample, &GridPosition)>();
        let (item_type, sample, pos) = item_query.single(&world);

        assert_eq!(item_type, &ItemType::GeneticSample);
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
        let _animal = world.spawn((
            Fauna { fauna_type: FaunaType::SpaceRat, ..Default::default() },
            GridPosition { x: 0, y: 1 }
        )).id();

        // Perform collection (target entity)
        let success = collect_sample_action(&mut world, pop, GridPosition { x: 0, y: 1 }); // Target pos

        assert!(success);

        // Check for item
        let mut item_query = world.query::<(&ItemType, &GeneticSample)>();
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
        let bank = world.spawn((
            Building { building_type: BuildingType::GeneBank },
            GeneBank::default(),
            GridPosition { x: 5, y: 5 }
        )).id();

        // Add a sample to its storage manually
        let mut bank_comp = world.get_mut::<GeneBank>(bank).unwrap();
        bank_comp.store_sample(GeneticData::Flora(TerrainType::Tree));

        assert!(bank_comp.has_sample(&GeneticData::Flora(TerrainType::Tree)));
    }

    #[test]
    fn test_cloning_process_flora() {
        let mut world = setup_world();

        // Setup GeneBank with a sample and resources
        let bank = world.spawn((
            Building { building_type: BuildingType::GeneBank },
            GeneBank {
                stored_samples: vec![GeneticData::Flora(TerrainType::Tree)],
                active_cloning_job: Some((GeneticData::Flora(TerrainType::Tree), 10.0)), // 10 ticks remaining
                ..Default::default()
            },
            GridPosition { x: 5, y: 5 }
        )).id();

        // Run system for 11 ticks (10 to count down, 1 to complete)
        for _ in 0..11 {
            process_cloning_system(&mut world);
        }

        // Check result: Should produce a "Seed" item at (5,5)
        let mut item_query = world.query::<(&Item, &GridPosition)>();
        let (_item, pos) = item_query.single(&world);
        assert_eq!(pos.x, 5);
        assert_eq!(pos.y, 5);
        // Verify cloning job is cleared
        let bank_comp = world.get::<GeneBank>(bank).unwrap();
        assert!(bank_comp.active_cloning_job.is_none());
    }

    #[test]
    fn test_cloning_process_fauna() {
        let mut world = setup_world();

        let _bank = world.spawn((
            Building { building_type: BuildingType::GeneBank },
            GeneBank {
                stored_samples: vec![GeneticData::Fauna(FaunaType::Wolf)],
                active_cloning_job: Some((GeneticData::Fauna(FaunaType::Wolf), 0.0)), // Finished
                ..Default::default()
            },
            GridPosition { x: 5, y: 5 }
        )).id();

        process_cloning_system(&mut world);

        // Should spawn a Fauna entity
        let mut fauna_query = world.query::<(&Fauna, &GridPosition)>();
        let (fauna, pos) = fauna_query.single(&world);
        assert_eq!(fauna.fauna_type, FaunaType::Wolf);
        assert_eq!(pos.x, 5);
        assert_eq!(pos.y, 5);
    }

    #[test]
    fn test_evaluate_collect_sample_finds_nearest() {
        let mut world = setup_world();
        let weights = UtilityWeights::default();

        let target1 = world.spawn(GridPosition { x: 5, y: 0 }).id(); // Dist 5
        let target2 = world.spawn(GridPosition { x: 2, y: 0 }).id(); // Dist 2

        let designations = vec![
            PositionProxy { entity: target1, pos: GridPosition { x: 5, y: 0 } },
            PositionProxy { entity: target2, pos: GridPosition { x: 2, y: 0 } },
        ];

        let pop_pos = GridPosition { x: 0, y: 0 };

        let result = evaluate_collect_sample(&pop_pos, &weights, &designations);

        assert!(result.is_some());
        let (_, entity) = result.unwrap();
        assert_eq!(entity, target2, "Should pick closest target");
    }
}
