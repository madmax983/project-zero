//! Utility AI Population Helpers
//!
//! This module contains helper functions for gathering data from the ECS World
//! and populating the `UtilityAIBuffer` with candidates for AI evaluation.
//!
//! It separates the "data gathering" concern from the "decision making" concern
//! in `utility_ai.rs`.

use crate::layer1::admin::Office;
use crate::layer1::building::{Building, BuildingType, ShiftSchedule};
use crate::layer1::day_night::TimeOfDay;
use crate::layer1::designation::{Designation, DesignationType};
use crate::layer1::energy::PowerConsumer;
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

// --- Helpers ---

fn is_active_shift(schedule: Option<&ShiftSchedule>, time: TimeOfDay) -> bool {
    schedule.map(|s| s.is_active(time)).unwrap_or(true)
}

fn is_powered(power: Option<&PowerConsumer>) -> bool {
    power.map(|p| p.active).unwrap_or(true)
}

fn is_at_capacity(current: usize, max: usize) -> bool {
    current >= max
}

fn extend_simple<T: Component>(world: &mut World, buffer: &mut Vec<ScorableCandidate>) {
    buffer.extend(
        world
            .query_filtered::<(Entity, &GridPosition), With<T>>()
            .iter(world)
            .map(|(e, pos)| ScorableCandidate::new(e, *pos)),
    );
}

fn populate_simple<T: Component>(world: &mut World, buffer: &mut Vec<ScorableCandidate>) {
    buffer.clear();
    extend_simple::<T>(world, buffer);
}

fn populate_building_type(
    world: &mut World,
    buffer: &mut Vec<ScorableCandidate>,
    b_type: BuildingType,
) {
    buffer.clear();
    buffer.extend(
        world
            .query::<(Entity, &GridPosition, &Building)>()
            .iter(world)
            .filter(|(_, _, b)| b.building_type == b_type)
            .map(|(e, pos, _)| ScorableCandidate::new(e, *pos)),
    );
}

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
    buffer.extend(
        world
            .query::<(
                Entity,
                &GridPosition,
                &Farm,
                Option<&ShiftSchedule>,
                Option<&PowerConsumer>,
            )>()
            .iter(world)
            .filter(|(_, _, farm, schedule, power)| {
                is_active_shift(*schedule, cycle.time_of_day)
                    && is_powered(*power)
                    && !is_at_capacity(farm.workers.len(), farm.capacity)
            })
            .map(|(entity, pos, farm, _, _)| {
                ScorableCandidate::with_capacity(entity, *pos, farm.capacity, farm.workers.len())
            }),
    );
}

fn populate_housing(world: &mut World, buffer: &mut Vec<ScorableCandidate>) {
    buffer.clear();
    buffer.extend(
        world
            .query::<(Entity, &GridPosition, &Housing)>()
            .iter(world)
            .filter(|(_, _, housing)| !is_at_capacity(housing.residents.len(), housing.capacity))
            .map(|(entity, pos, housing)| {
                ScorableCandidate::with_capacity(
                    entity,
                    *pos,
                    housing.capacity,
                    housing.residents.len(),
                )
            }),
    );
}

fn populate_taverns(world: &mut World, buffer: &mut Vec<ScorableCandidate>) {
    buffer.clear();
    buffer.extend(
        world
            .query::<(Entity, &GridPosition, &Tavern)>()
            .iter(world)
            .filter(|(_, _, tavern)| !is_at_capacity(tavern.visitors.len(), tavern.capacity))
            .map(|(entity, pos, tavern)| {
                ScorableCandidate::with_capacity(
                    entity,
                    *pos,
                    tavern.capacity,
                    tavern.visitors.len(),
                )
            }),
    );
}

fn populate_libraries(
    world: &mut World,
    buffer: &mut Vec<ScorableCandidate>,
    cycle: &crate::layer1::day_night::DayNightCycle,
) {
    buffer.clear();
    buffer.extend(
        world
            .query::<(Entity, &GridPosition, &Library, Option<&ShiftSchedule>)>()
            .iter(world)
            .filter(|(_, _, _, schedule)| is_active_shift(*schedule, cycle.time_of_day))
            .map(|(entity, pos, _, _)| ScorableCandidate::with_capacity(entity, *pos, 5, 0)),
    );
}

fn populate_refining(
    world: &mut World,
    buffer: &mut Vec<ScorableCandidate>,
    context: &WorldContext,
) {
    buffer.clear();
    buffer.extend(
        world
            .query::<(
                Entity,
                &GridPosition,
                &Building,
                &RefiningProgress,
                Option<&ShiftSchedule>,
                Option<&PowerConsumer>,
            )>()
            .iter(world)
            .filter(|(_, _, building, _, schedule, power)| {
                is_active_shift(*schedule, context.cycle.time_of_day)
                    && is_powered(*power)
                    && get_refining_recipe(building.building_type, context.resources).0
            })
            .map(|(entity, pos, _, progress, _, _)| {
                let mut candidate = ScorableCandidate::new(entity, *pos);
                candidate.score_bonus = if progress.current > 0.0 { 0.1 } else { 0.0 };
                candidate
            }),
    );
}

fn populate_hospitals(world: &mut World, buffer: &mut Vec<ScorableCandidate>) {
    buffer.clear();
    buffer.extend(
        world
            .query::<(
                Entity,
                &GridPosition,
                &Hospital,
                Option<&PowerConsumer>,
            )>()
            .iter(world)
            .filter(|(_, _, _, power)| is_powered(*power))
            .map(|(entity, pos, _, _)| ScorableCandidate::with_capacity(entity, *pos, 10, 0)),
    );
}

fn populate_offices(
    world: &mut World,
    buffer: &mut Vec<ScorableCandidate>,
    cycle: &crate::layer1::day_night::DayNightCycle,
) {
    buffer.clear();
    buffer.extend(
        world
            .query::<(Entity, &GridPosition, &Office, Option<&ShiftSchedule>)>()
            .iter(world)
            .filter(|(_, _, office, schedule)| {
                is_active_shift(*schedule, cycle.time_of_day)
                    && !is_at_capacity(office.workers.len(), office.capacity)
            })
            .map(|(entity, pos, office, _)| {
                ScorableCandidate::with_capacity(
                    entity,
                    *pos,
                    office.capacity,
                    office.workers.len(),
                )
            }),
    );
}

fn populate_showers(world: &mut World, buffer: &mut Vec<ScorableCandidate>) {
    populate_building_type(world, buffer, BuildingType::Shower);
}

fn populate_buffer_designations(world: &mut World, buffer: &mut UtilityAIBuffer) {
    // Designations (Work, Repair, Tame)
    buffer.work_designations.clear();
    buffer.repair_designations.clear();
    buffer.tame_designations.clear();
    let mut des_query = world.query::<(Entity, &GridPosition, &Designation)>();
    for (entity, pos, des) in des_query.iter(world) {
        let candidate = ScorableCandidate::new(entity, *pos);
        match des.designation_type {
            DesignationType::Repair => buffer.repair_designations.push(candidate),
            DesignationType::Tame => buffer.tame_designations.push(candidate),
            _ => buffer.work_designations.push(candidate),
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
    populate_simple::<Stockpile>(world, buffer);
}

fn populate_items(world: &mut World, buffer: &mut Vec<ScorableCandidate>) {
    buffer.clear();
    buffer.extend(
        world
            .query::<(Entity, &GridPosition, &ResourceItem)>()
            .iter(world)
            .map(|(entity, pos, item)| {
                let mut c = ScorableCandidate::new(entity, *pos);
                c.resource_type = Some(item.resource_type);
                c
            }),
    );
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
    buffer.extend(
        world
            .query::<(Entity, &GridPosition, &Item)>()
            .iter(world)
            .filter(|(_, pos, _)| !stockpiles.contains(pos))
            .map(|(entity, pos, item)| {
                let mut c = ScorableCandidate::new(entity, *pos);
                c.item_type = Some(item.item_type.clone());
                c
            }),
    );
}

fn populate_anomalies(world: &mut World, buffer: &mut Vec<ScorableCandidate>) {
    populate_simple::<Anomaly>(world, buffer);
}

fn populate_corpses(world: &mut World, buffer: &mut Vec<ScorableCandidate>) {
    populate_simple::<Corpse>(world, buffer);
}

fn populate_graves(world: &mut World, buffer: &mut Vec<ScorableCandidate>) {
    buffer.clear();
    buffer.extend(
        world
            .query::<(Entity, &GridPosition, &Grave)>()
            .iter(world)
            .filter(|(_, _, grave)| !grave.occupied)
            .map(|(entity, pos, _)| ScorableCandidate::new(entity, *pos)),
    );
}

fn populate_repair_structures(world: &mut World, buffer: &mut Vec<ScorableCandidate>) {
    buffer.clear();
    buffer.extend(
        world
            .query::<(
                Entity,
                &GridPosition,
                &Structure,
                Option<&DeferMaintenance>,
            )>()
            .iter(world)
            .filter(|(_, _, structure, defer)| {
                defer.is_none()
                    && (structure.current_hp - structure.max_hp).abs() >= f32::EPSILON
            })
            .map(|(entity, pos, _, _)| ScorableCandidate::new(entity, *pos)),
    );
}

fn populate_wanted_criminals(world: &mut World, buffer: &mut Vec<ScorableCandidate>) {
    populate_simple::<Wanted>(world, buffer);
}

fn populate_walls(world: &mut World, buffer: &mut Vec<ScorableCandidate>) {
    populate_building_type(world, buffer, BuildingType::Wall);
}

fn populate_enemies(world: &mut World, buffer: &mut Vec<ScorableCandidate>) {
    buffer.clear();
    extend_simple::<Fauna>(world, buffer);
    extend_simple::<Flora>(world, buffer);
}

fn populate_all_structures(world: &mut World, buffer: &mut Vec<ScorableCandidate>) {
    populate_simple::<Structure>(world, buffer);
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
        let full_farm = Farm {
            capacity: 1,
            workers: vec![Entity::from_raw(123)],
            ..Default::default()
        };

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
        let resources = ColonyResources {
            wood: 1.0,
            ..Default::default()
        };
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
        let resources = ColonyResources {
            wood: 1.0,
            ..Default::default()
        };

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
        let resources = ColonyResources {
            wood: 1.0,
            ..Default::default()
        };

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
        let housing = Housing {
            capacity: 1,
            residents: vec![Entity::from_raw(123)],
            ..Default::default()
        };

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
