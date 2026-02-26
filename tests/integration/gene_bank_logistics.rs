
use scale::layer1::building::{Building, BuildingType};
use scale::layer1::gene_bank::{GeneBank, GeneticData, GeneticSample};
use scale::layer1::hauling::haul_system;
use scale::layer1::items::{CarryingItem, Item, ItemType};
use scale::layer1::map::GridPosition;
use scale::layer1::pop::Pop;
use scale::layer1::resources::ColonyResources;
use scale::layer1::terrain::TerrainType;
use scale::layer1::utility_ai::{ActionType, PopAction};
use scale::layer1::utility_ai::UtilityWeights;
use scale::layer1::execution::{AtTarget, MovementTarget};
use scale::shared::time::SimulationTime;
use bevy_ecs::prelude::*;

#[test]
fn test_gene_bank_hauling_integration() {
    let mut world = World::new();
    world.insert_resource(SimulationTime::default());
    world.insert_resource(ColonyResources::default());
    world.insert_resource(scale::layer1::zone::ZoneGrid::new(10, 10)); // Required by haul_system

    // 1. Spawn Pop (Hauler)
    let pop = world
        .spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            PopAction {
                current: ActionType::Haul,
                ..Default::default()
            },
            UtilityWeights::default(),
        ))
        .id();

    // 2. Spawn Genetic Sample Item
    let sample_data = GeneticData::Flora(TerrainType::Tree);
    let item = world
        .spawn((
            Item {
                item_type: ItemType::GeneticSample,
            },
            GeneticSample {
                data: sample_data.clone(),
            },
            GridPosition { x: 2, y: 0 },
        ))
        .id();

    // 3. Spawn Gene Bank Building
    let gene_bank = world
        .spawn((
            Building {
                building_type: BuildingType::GeneBank,
            },
            GeneBank::default(),
            GridPosition { x: 5, y: 0 },
        ))
        .id();

    // --- Phase 1: Pickup ---

    // Teleport pop to item to simulate arrival
    *world.get_mut::<GridPosition>(pop).unwrap() = GridPosition { x: 2, y: 0 };
    world.entity_mut(pop).insert(AtTarget);

    // Run haul system to pickup
    haul_system(&mut world);

    // Verify pickup
    assert!(world.get::<CarryingItem>(pop).is_some(), "Pop should pick up the sample");
    assert!(world.get::<GridPosition>(item).is_none(), "Item should be off the grid");

    // --- Phase 2: Find Target ---

    // Run haul system to find target
    haul_system(&mut world);

    // Verify target is Gene Bank
    let target = world.get::<MovementTarget>(pop);
    assert!(target.is_some(), "Pop should have a target");
    assert_eq!(target.unwrap().target_entity, gene_bank, "Target should be the Gene Bank");

    // --- Phase 3: Dropoff ---

    // Teleport pop to Gene Bank
    *world.get_mut::<GridPosition>(pop).unwrap() = GridPosition { x: 5, y: 0 };
    world.entity_mut(pop).insert(AtTarget);

    // Run haul system to dropoff
    haul_system(&mut world);

    // Verify Sample is stored in Gene Bank
    let bank_comp = world.get::<GeneBank>(gene_bank).unwrap();
    assert!(bank_comp.has_sample(&sample_data), "Gene Bank should contain the sample");

    // Verify Item Despawned
    assert!(world.get_entity(item).is_err(), "Item entity should be despawned after storage");
    assert!(world.get::<CarryingItem>(pop).is_none(), "Pop should be empty");
}
