//! Utility AI Population Helpers
//!
//! This module contains helper functions for gathering data from the ECS World
//! and populating the `UtilityAIBuffer` with candidates for AI evaluation.
//!
//! It separates the "data gathering" concern from the "decision making" concern
//! in `utility_ai.rs`.

use crate::layer1::admin::Office;
use crate::layer1::building::{Building, BuildingType, ShiftSchedule};
use crate::layer1::designation::{Designation, DesignationType};
use crate::layer1::farm::Farm;
use crate::layer1::fauna::Fauna;
use crate::layer1::flora::Flora;
use crate::layer1::funeral::{Corpse, Grave};
use crate::layer1::housing::Housing;
use crate::layer1::items::Item;
use crate::layer1::justice::Wanted;
use crate::layer1::map::GridPosition;
use crate::layer1::medical::Hospital;
use crate::layer1::refining::get_refining_recipe;
use crate::layer1::resources::{RefiningProgress, ResourceItem};
use crate::layer1::science::Anomaly;
use crate::layer1::social::Tavern;
use crate::layer1::stockpile::Stockpile;
use crate::layer1::structure::{DeferMaintenance, Structure};
use crate::layer1::tech::Library;
use crate::layer1::utility_eval_types::{
    PopEvalData, PopEvaluationQuery, ScorableCandidate, UtilityAIBuffer, WorldContext,
};
use crate::layer1::utility_types::UtilityConfig;
use bevy_ecs::prelude::*;
use std::collections::HashSet;

/// Populates the `UtilityAIBuffer` with all candidate entities from the world.
///
/// This function calls various internal helpers to query different entity types
/// (buildings, items, designations) and fill the corresponding vectors in the buffer.
///
/// # Arguments
/// * `world` - Mutable reference to the ECS World.
/// * `buffer` - The buffer to populate.
/// * `context` - World context (time, resources, etc.).
pub fn populate_ai_buffer(world: &mut World, buffer: &mut UtilityAIBuffer, context: &WorldContext) {
    populate_buffer_buildings(world, buffer, context);
    populate_buffer_designations(world, buffer);
    populate_buffer_items_and_misc(world, buffer);
    populate_walls(world, &mut buffer.walls);
    populate_enemies(world, &mut buffer.enemies);
    populate_all_structures(world, &mut buffer.all_structures);
}

fn populate_buffer_buildings(
    world: &mut World,
    buffer: &mut UtilityAIBuffer,
    context: &WorldContext,
) {
    populate_farms(world, &mut buffer.farms, context.cycle);
    populate_housing(world, &mut buffer.housing);
    populate_taverns(world, &mut buffer.taverns);
    populate_libraries(world, &mut buffer.libraries, context.cycle);
    populate_refining(world, &mut buffer.refining, context);
    populate_hospitals(world, &mut buffer.hospitals);
    populate_offices(world, &mut buffer.offices, context.cycle);
    populate_showers(world, &mut buffer.showers);
}

fn populate_farms(
    world: &mut World,
    buffer: &mut Vec<ScorableCandidate>,
    cycle: &crate::layer1::day_night::DayNightCycle,
) {
    buffer.clear();
    let mut farm_query = world.query::<(
        Entity,
        &GridPosition,
        &Farm,
        Option<&ShiftSchedule>,
        Option<&crate::layer1::energy::PowerConsumer>,
    )>();
    for (entity, pos, farm, schedule, power) in farm_query.iter(world) {
        if schedule.is_some_and(|s| !s.is_active(cycle.time_of_day)) {
            continue;
        }
        if power.is_some_and(|p| !p.active) {
            continue;
        }
        if farm.workers.len() >= farm.capacity {
            continue;
        }
        buffer.push(ScorableCandidate::with_capacity(
            entity,
            *pos,
            farm.capacity,
            farm.workers.len(),
        ));
    }
}

fn populate_housing(world: &mut World, buffer: &mut Vec<ScorableCandidate>) {
    buffer.clear();
    let mut housing_query = world.query::<(Entity, &GridPosition, &Housing)>();
    for (entity, pos, housing) in housing_query.iter(world) {
        if housing.residents.len() >= housing.capacity {
            continue;
        }
        buffer.push(ScorableCandidate::with_capacity(
            entity,
            *pos,
            housing.capacity,
            housing.residents.len(),
        ));
    }
}

fn populate_taverns(world: &mut World, buffer: &mut Vec<ScorableCandidate>) {
    buffer.clear();
    let mut tavern_query = world.query::<(Entity, &GridPosition, &Tavern)>();
    for (entity, pos, tavern) in tavern_query.iter(world) {
        if tavern.visitors.len() >= tavern.capacity {
            continue;
        }
        buffer.push(ScorableCandidate::with_capacity(
            entity,
            *pos,
            tavern.capacity,
            tavern.visitors.len(),
        ));
    }
}

fn populate_libraries(
    world: &mut World,
    buffer: &mut Vec<ScorableCandidate>,
    cycle: &crate::layer1::day_night::DayNightCycle,
) {
    buffer.clear();
    let mut library_query =
        world.query::<(Entity, &GridPosition, &Library, Option<&ShiftSchedule>)>();
    for (entity, pos, _library, schedule) in library_query.iter(world) {
        if schedule.is_some_and(|s| !s.is_active(cycle.time_of_day)) {
            continue;
        }
        buffer.push(ScorableCandidate::with_capacity(entity, *pos, 5, 0));
    }
}

fn populate_refining(
    world: &mut World,
    buffer: &mut Vec<ScorableCandidate>,
    context: &WorldContext,
) {
    buffer.clear();
    let mut refine_query = world.query::<(
        Entity,
        &GridPosition,
        &Building,
        &RefiningProgress,
        Option<&ShiftSchedule>,
        Option<&crate::layer1::energy::PowerConsumer>,
    )>();
    for (entity, pos, building, progress, schedule, power) in refine_query.iter(world) {
        if schedule.is_some_and(|s| !s.is_active(context.cycle.time_of_day)) {
            continue;
        }
        if power.is_some_and(|p| !p.active) {
            continue;
        }

        // Check recipe affordability (Global check)
        let (can_afford, _, _, _) = get_refining_recipe(building.building_type, context.resources);
        if !can_afford {
            continue;
        }

        let mut candidate = ScorableCandidate::new(entity, *pos);
        candidate.score_bonus = if progress.current > 0.0 { 0.1 } else { 0.0 };
        buffer.push(candidate);
    }
}

fn populate_hospitals(world: &mut World, buffer: &mut Vec<ScorableCandidate>) {
    buffer.clear();
    let mut hospital_query = world.query::<(
        Entity,
        &GridPosition,
        &Hospital,
        Option<&crate::layer1::energy::PowerConsumer>,
    )>();
    for (entity, pos, _, power) in hospital_query.iter(world) {
        if power.is_some_and(|p| !p.active) {
            continue;
        }
        buffer.push(ScorableCandidate::with_capacity(entity, *pos, 10, 0));
    }
}

fn populate_offices(
    world: &mut World,
    buffer: &mut Vec<ScorableCandidate>,
    cycle: &crate::layer1::day_night::DayNightCycle,
) {
    buffer.clear();
    let mut office_query =
        world.query::<(Entity, &GridPosition, &Office, Option<&ShiftSchedule>)>();
    for (entity, pos, office, schedule) in office_query.iter(world) {
        if schedule.is_some_and(|s| !s.is_active(cycle.time_of_day)) {
            continue;
        }
        if office.workers.len() >= office.capacity {
            continue;
        }
        buffer.push(ScorableCandidate::with_capacity(
            entity,
            *pos,
            office.capacity,
            office.workers.len(),
        ));
    }
}

fn populate_showers(world: &mut World, buffer: &mut Vec<ScorableCandidate>) {
    buffer.clear();
    let mut query = world.query::<(Entity, &GridPosition, &Building)>();
    for (entity, pos, building) in query.iter(world) {
        if building.building_type == BuildingType::Shower {
            buffer.push(ScorableCandidate::with_capacity(entity, *pos, 1, 0));
        }
    }
}

fn populate_buffer_designations(world: &mut World, buffer: &mut UtilityAIBuffer) {
    // Designations (Work, Repair, Tame)
    buffer.work_designations.clear();
    buffer.repair_designations.clear();
    buffer.tame_designations.clear();
    let mut des_query = world.query::<(Entity, &GridPosition, &Designation)>();
    for (entity, pos, des) in des_query.iter(world) {
        match des.designation_type {
            DesignationType::Repair => {
                buffer
                    .repair_designations
                    .push(ScorableCandidate::new(entity, *pos));
            }
            DesignationType::Tame => {
                buffer
                    .tame_designations
                    .push(ScorableCandidate::new(entity, *pos));
            }
            _ => {
                buffer
                    .work_designations
                    .push(ScorableCandidate::new(entity, *pos));
            }
        }
    }
}

fn populate_buffer_items_and_misc(world: &mut World, buffer: &mut UtilityAIBuffer) {
    populate_stockpiles(world, &mut buffer.stockpiles);

    // Create lookup set for stockpiles (Optimization: O(1) lookup instead of O(N))
    let stockpile_positions: HashSet<GridPosition> =
        buffer.stockpiles.iter().map(|s| s.pos).collect();

    populate_items(world, &mut buffer.items);
    populate_generic_items(world, &stockpile_positions, &mut buffer.item_entities);
    populate_anomalies(world, &mut buffer.anomalies);
    populate_corpses(world, &mut buffer.corpses);
    populate_graves(world, &mut buffer.graves);
    populate_repair_structures(world, &mut buffer.repair_structures);
    populate_wanted_criminals(world, &mut buffer.wanted_criminals);
}

fn populate_stockpiles(world: &mut World, buffer: &mut Vec<ScorableCandidate>) {
    buffer.clear();
    let mut stock_query = world.query::<(Entity, &GridPosition, &Stockpile)>();
    for (entity, pos, _) in stock_query.iter(world) {
        buffer.push(ScorableCandidate::new(entity, *pos));
    }
}

fn populate_items(world: &mut World, buffer: &mut Vec<ScorableCandidate>) {
    buffer.clear();
    let mut item_query = world.query::<(Entity, &GridPosition, &ResourceItem)>();
    for (entity, pos, item) in item_query.iter(world) {
        let mut candidate = ScorableCandidate::new(entity, *pos);
        candidate.resource_type = Some(item.resource_type);
        buffer.push(candidate);
    }
}

/// Populates loose items (Tools, Clothing, etc.), filtering out those already in stockpiles.
///
/// # Optimization
/// Uses a `HashSet` for stockpile lookup to avoid O(N*M) complexity where N=Items and M=Stockpiles.
/// This reduces the check to O(1) per item.
fn populate_generic_items(
    world: &mut World,
    stockpiles: &HashSet<GridPosition>,
    buffer: &mut Vec<ScorableCandidate>,
) {
    buffer.clear();
    let mut item_entity_query = world.query::<(Entity, &GridPosition, &Item)>();
    for (entity, pos, item) in item_entity_query.iter(world) {
        // Optimization: Don't haul items that are already at a stockpile
        if stockpiles.contains(pos) {
            continue;
        }

        let mut candidate = ScorableCandidate::new(entity, *pos);
        candidate.item_type = Some(item.item_type.clone());
        buffer.push(candidate);
    }
}

fn populate_anomalies(world: &mut World, buffer: &mut Vec<ScorableCandidate>) {
    buffer.clear();
    let mut anomaly_query = world.query::<(Entity, &GridPosition, &Anomaly)>();
    for (entity, pos, _) in anomaly_query.iter(world) {
        buffer.push(ScorableCandidate::new(entity, *pos));
    }
}

fn populate_corpses(world: &mut World, buffer: &mut Vec<ScorableCandidate>) {
    buffer.clear();
    let mut corpse_query = world.query::<(Entity, &GridPosition, &Corpse)>();
    for (entity, pos, _) in corpse_query.iter(world) {
        buffer.push(ScorableCandidate::new(entity, *pos));
    }
}

fn populate_graves(world: &mut World, buffer: &mut Vec<ScorableCandidate>) {
    buffer.clear();
    let mut grave_query = world.query::<(Entity, &GridPosition, &Grave)>();
    for (entity, pos, grave) in grave_query.iter(world) {
        if !grave.occupied {
            buffer.push(ScorableCandidate::new(entity, *pos));
        }
    }
}

fn populate_repair_structures(world: &mut World, buffer: &mut Vec<ScorableCandidate>) {
    buffer.clear();
    let mut struct_query =
        world.query::<(Entity, &GridPosition, &Structure, Option<&DeferMaintenance>)>();
    for (entity, pos, structure, defer) in struct_query.iter(world) {
        if defer.is_some() {
            continue;
        }
        if (structure.current_hp - structure.max_hp).abs() < f32::EPSILON {
            continue;
        }

        buffer.push(ScorableCandidate::new(entity, *pos));
    }
}

fn populate_wanted_criminals(world: &mut World, buffer: &mut Vec<ScorableCandidate>) {
    buffer.clear();
    let mut wanted_query = world.query::<(Entity, &GridPosition, &Wanted)>();
    for (entity, pos, _) in wanted_query.iter(world) {
        buffer.push(ScorableCandidate::new(entity, *pos));
    }
}

fn populate_walls(world: &mut World, buffer: &mut Vec<ScorableCandidate>) {
    buffer.clear();
    let mut query = world.query::<(Entity, &GridPosition, &Building)>();
    for (entity, pos, building) in query.iter(world) {
        if building.building_type == BuildingType::Wall {
            buffer.push(ScorableCandidate::new(entity, *pos));
        }
    }
}

fn populate_enemies(world: &mut World, buffer: &mut Vec<ScorableCandidate>) {
    buffer.clear();
    let mut fauna_query = world.query::<(Entity, &GridPosition, &Fauna)>();
    for (entity, pos, _) in fauna_query.iter(world) {
        buffer.push(ScorableCandidate::new(entity, *pos));
    }
    let mut flora_query = world.query::<(Entity, &GridPosition, &Flora)>();
    for (entity, pos, _) in flora_query.iter(world) {
        buffer.push(ScorableCandidate::new(entity, *pos));
    }
}

fn populate_all_structures(world: &mut World, buffer: &mut Vec<ScorableCandidate>) {
    buffer.clear();
    let mut query = world.query::<(Entity, &GridPosition, &Structure)>();
    for (entity, pos, _) in query.iter(world) {
        buffer.push(ScorableCandidate::new(entity, *pos));
    }
}

/// Collects data for all Pops that need to be evaluated this tick.
///
/// Filters pops based on `ticks_committed` and `UtilityConfig::evaluation_interval`.
/// Also populates additional data like insulation from equipped clothing.
///
/// # Arguments
/// * `world` - Mutable reference to the ECS World.
/// * `buffer` - The buffer to store `PopEvalData`.
/// * `config` - Utility AI configuration.
pub fn collect_pop_data(world: &mut World, buffer: &mut UtilityAIBuffer, config: &UtilityConfig) {
    buffer.pop_data.clear();
    buffer.pop_data.extend(
        world
            .query_filtered::<PopEvaluationQuery, Without<crate::layer1::cryo::CryoStasis>>()
            .iter(world)
            .filter(|item| {
                item.action.ticks_committed >= config.evaluation_interval
                    && (item.inmate.is_none() || item.penal_labor.is_some())
            })
            .map(PopEvalData::from_query_item),
    );

    // Populate Insulation from Clothing entities
    let mut clothing_query = world.query::<&crate::layer1::items::Clothing>();
    for data in &mut buffer.pop_data {
        if let Some(eq) = data.equipment
            && let Some(body_entity) = eq.body
            && let Ok(clothing) = clothing_query.get(world, body_entity)
        {
            data.insulation = clothing.insulation;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::items::{Item, ItemType};
    use crate::layer1::map::GridPosition;

    #[test]
    fn test_populate_generic_items_filtering() {
        let mut world = World::new();

        // 1. Create Stockpile Positions Set
        let mut stockpiles = std::collections::HashSet::new();
        stockpiles.insert(GridPosition { x: 10, y: 0 });

        // 2. Spawn Item inside stockpile (Should be filtered)
        let _item_in = world
            .spawn((
                Item {
                    item_type: ItemType::Manual,
                },
                GridPosition { x: 10, y: 0 },
            ))
            .id();

        // 3. Spawn Item outside stockpile (Should be included)
        let item_out = world
            .spawn((
                Item {
                    item_type: ItemType::Manual,
                },
                GridPosition { x: 5, y: 0 },
            ))
            .id();

        // 4. Run population
        let mut buffer = Vec::new();
        populate_generic_items(&mut world, &stockpiles, &mut buffer);

        // 5. Verify
        assert_eq!(buffer.len(), 1);
        assert_eq!(buffer[0].entity, item_out);
    }

    #[test]
    fn test_populate_farms_filters_inactive_schedule() {
        use crate::layer1::building::ShiftSchedule;
        use crate::layer1::day_night::{DayNightCycle, TimeOfDay};
        use crate::layer1::farm::Farm;

        let mut world = World::new();
        let cycle = DayNightCycle {
            time_of_day: TimeOfDay::Night, // Currently Night
            ..Default::default()
        };

        // Farm active only during Day
        world.spawn((
            Farm::default(),
            GridPosition { x: 0, y: 0 },
            ShiftSchedule {
                day_shift: true,
                night_shift: false,
            },
        ));

        let mut buffer = Vec::new();
        populate_farms(&mut world, &mut buffer, &cycle);

        assert!(
            buffer.is_empty(),
            "Should filter out farm inactive at night"
        );

        // Change cycle to Day
        let cycle_day = DayNightCycle {
            time_of_day: TimeOfDay::Day,
            ..Default::default()
        };
        populate_farms(&mut world, &mut buffer, &cycle_day);
        assert_eq!(buffer.len(), 1, "Should include farm active during day");
    }

    #[test]
    fn test_populate_farms_filters_unpowered() {
        use crate::layer1::day_night::DayNightCycle;
        use crate::layer1::energy::PowerConsumer;
        use crate::layer1::farm::Farm;

        let mut world = World::new();
        let cycle = DayNightCycle::default(); // Default is Day

        // Unpowered Farm
        world.spawn((
            Farm::default(),
            GridPosition { x: 0, y: 0 },
            PowerConsumer {
                active: false,
                ..Default::default()
            },
        ));

        let mut buffer = Vec::new();
        populate_farms(&mut world, &mut buffer, &cycle);

        assert!(buffer.is_empty(), "Should filter out unpowered farm");

        // Powered Farm
        world.spawn((
            Farm::default(),
            GridPosition { x: 1, y: 0 },
            PowerConsumer {
                active: true,
                ..Default::default()
            },
        ));
        populate_farms(&mut world, &mut buffer, &cycle);
        assert_eq!(buffer.len(), 1, "Should include powered farm");
    }

    #[test]
    fn test_populate_farms_filters_full_capacity() {
        use crate::layer1::day_night::DayNightCycle;
        use crate::layer1::farm::Farm;

        let mut world = World::new();
        let cycle = DayNightCycle::default();

        // Full Farm
        let mut full_farm = Farm::default();
        full_farm.capacity = 1;
        full_farm.workers.push(Entity::from_raw(123)); // Mock worker

        world.spawn((full_farm, GridPosition { x: 0, y: 0 }));

        let mut buffer = Vec::new();
        populate_farms(&mut world, &mut buffer, &cycle);

        assert!(buffer.is_empty(), "Should filter out full farm");

        // Empty Farm
        let empty_farm = Farm {
            capacity: 1,
            ..Default::default()
        };
        world.spawn((empty_farm, GridPosition { x: 1, y: 0 }));
        populate_farms(&mut world, &mut buffer, &cycle);
        assert_eq!(buffer.len(), 1, "Should include empty farm");
    }

    #[test]
    fn test_populate_refining_filters_unaffordable() {
        use crate::layer1::building::{Building, BuildingType};
        use crate::layer1::day_night::DayNightCycle;
        use crate::layer1::resources::{ColonyResources, RefiningProgress};
        use crate::layer1::taboo::TabooState;
        use crate::layer1::utility_eval_types::WorldContext;
        use crate::layer1::zone::ZoneGrid;

        let mut world = World::new();
        // LumberMill requires 1 Wood.
        let mut resources = ColonyResources::default();
        resources.wood = 0.0; // Cannot afford

        let cycle = DayNightCycle::default();
        let taboo = TabooState::default();
        let zone_grid = ZoneGrid::new(10, 10);

        let context = WorldContext {
            resources: &resources,
            cycle: &cycle,
            taboo: &taboo,
            factions: None,
            zone_grid: &zone_grid,
            temperature_grid: None,
        };

        world.spawn((
            Building {
                building_type: BuildingType::LumberMill,
            },
            GridPosition { x: 0, y: 0 },
            RefiningProgress::default(),
        ));

        let mut buffer = Vec::new();
        populate_refining(&mut world, &mut buffer, &context);

        assert!(buffer.is_empty(), "Should filter out unaffordable recipe");

        // Now afford
        resources.wood = 1.0;
        let context_valid = WorldContext {
            resources: &resources,
            cycle: &cycle,
            taboo: &taboo,
            factions: None,
            zone_grid: &zone_grid,
            temperature_grid: None,
        };
        populate_refining(&mut world, &mut buffer, &context_valid);
        assert_eq!(buffer.len(), 1, "Should include affordable recipe");
    }

    #[test]
    fn test_populate_refining_filters_inactive_schedule() {
        use crate::layer1::building::{Building, BuildingType, ShiftSchedule};
        use crate::layer1::day_night::{DayNightCycle, TimeOfDay};
        use crate::layer1::resources::{ColonyResources, RefiningProgress};
        use crate::layer1::taboo::TabooState;
        use crate::layer1::utility_eval_types::WorldContext;
        use crate::layer1::zone::ZoneGrid;

        let mut world = World::new();
        let mut resources = ColonyResources::default();
        resources.wood = 1.0;

        let cycle = DayNightCycle {
            time_of_day: TimeOfDay::Night,
            ..Default::default()
        };
        let taboo = TabooState::default();
        let zone_grid = ZoneGrid::new(10, 10);

        let context = WorldContext {
            resources: &resources,
            cycle: &cycle,
            taboo: &taboo,
            factions: None,
            zone_grid: &zone_grid,
            temperature_grid: None,
        };

        world.spawn((
            Building {
                building_type: BuildingType::LumberMill,
            },
            GridPosition { x: 0, y: 0 },
            RefiningProgress::default(),
            ShiftSchedule {
                day_shift: true,
                night_shift: false,
            },
        ));

        let mut buffer = Vec::new();
        populate_refining(&mut world, &mut buffer, &context);

        assert!(buffer.is_empty(), "Should filter out inactive shift");
    }

    #[test]
    fn test_populate_refining_filters_unpowered() {
        use crate::layer1::building::{Building, BuildingType};
        use crate::layer1::day_night::DayNightCycle;
        use crate::layer1::energy::PowerConsumer;
        use crate::layer1::resources::{ColonyResources, RefiningProgress};
        use crate::layer1::taboo::TabooState;
        use crate::layer1::utility_eval_types::WorldContext;
        use crate::layer1::zone::ZoneGrid;

        let mut world = World::new();
        let mut resources = ColonyResources::default();
        resources.wood = 1.0;

        let cycle = DayNightCycle::default();
        let taboo = TabooState::default();
        let zone_grid = ZoneGrid::new(10, 10);

        let context = WorldContext {
            resources: &resources,
            cycle: &cycle,
            taboo: &taboo,
            factions: None,
            zone_grid: &zone_grid,
            temperature_grid: None,
        };

        world.spawn((
            Building {
                building_type: BuildingType::LumberMill,
            },
            GridPosition { x: 0, y: 0 },
            RefiningProgress::default(),
            PowerConsumer {
                active: false,
                ..Default::default()
            },
        ));

        let mut buffer = Vec::new();
        populate_refining(&mut world, &mut buffer, &context);

        assert!(buffer.is_empty(), "Should filter out unpowered building");
    }

    #[test]
    fn test_populate_housing_filters_full() {
        use crate::layer1::housing::Housing;

        let mut world = World::new();

        // Full Housing
        let mut housing = Housing::default();
        housing.capacity = 1;
        housing.residents.push(Entity::from_raw(123)); // Occupied

        world.spawn((housing, GridPosition { x: 0, y: 0 }));

        let mut buffer = Vec::new();
        populate_housing(&mut world, &mut buffer);

        assert!(buffer.is_empty(), "Should filter out full housing");

        // Available Housing
        let housing_empty = Housing::default();
        world.spawn((housing_empty, GridPosition { x: 1, y: 0 }));
        populate_housing(&mut world, &mut buffer);
        assert_eq!(buffer.len(), 1, "Should include available housing");
    }
}
