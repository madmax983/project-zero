#!/bin/bash
cat << 'INNER_EOF' > /tmp/arrival_patch.txt
<<<<<<< SEARCH
use bevy_ecs::prelude::*;
use ratatui::style::Color;

use crate::layer1::actions::{AssignedTo, AssignmentType};
use crate::layer1::admin::{AdminProvider, Office};
use crate::layer1::execution::components::{AtTarget, MovementTarget};
use crate::layer1::farm::Farm;
use crate::layer1::funeral::{handle_bury_corpse, Corpse, Grave};
use crate::layer1::housing::Housing;
use crate::layer1::items::{Clothing, ClothingType, Equipment, Item, Tool, ToolType, UnequipEvent};
use crate::layer1::map::GridPosition;
use crate::layer1::memory::Memories;
use crate::layer1::pop::Job;
use crate::layer1::resources::ColonyResources;
use crate::layer1::social::{handle_socialize, Tavern};
use crate::layer1::social_stratification::Prestige;
use crate::layer1::utility_types::ActionType;
use crate::shared::log::MessageLog;
use crate::shared::time::SimulationTime;

/// Handles arrival at targets: assigns pops to farms/housing.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn arrival_handler_system(
    mut arrivals: Query<
        (
            Entity,
            &GridPosition,
            &MovementTarget,
            Option<&mut Equipment>,
            Option<&mut crate::layer1::chemical::ChemicalState>,
            Option<&mut crate::layer1::health::Health>,
            Option<&mut crate::layer1::stress::StressTracker>,
        ),
        With<AtTarget>,
    >,
    items: Query<&crate::layer1::items::Item>,
    mut farms: Query<&mut Farm>,
    mut housing_q: Query<&mut Housing>,
    mut taverns: Query<&mut Tavern>,
    mut offices: Query<&mut Office>,
    corpses: Query<&Corpse>,
    mut graves: Query<(Entity, &GridPosition, &mut Grave)>,
    mut memories: Query<&mut Memories>,
    mut resources: ResMut<ColonyResources>,
    mut log: Option<ResMut<MessageLog>>,
    mut graffiti_map: Option<ResMut<crate::layer1::graffiti::GraffitiMap>>,
    mut unequip_events: EventWriter<UnequipEvent>,
    time: Res<SimulationTime>,
    mut commands: Commands,
) {
    for (
        pop_entity,
        pop_pos,
        mt,
        mut equipment_opt,
        mut chem_opt,
        mut health_opt,
        mut stress_opt,
    ) in &mut arrivals
    {
        let should_remove = process_arrival(
            mt.for_action,
            pop_entity,
            mt.target_entity,
            *pop_pos,
            mt.target_position,
            &mut equipment_opt,
            &mut chem_opt,
            &mut health_opt,
            &mut stress_opt,
            &items,
            &mut commands,
            &mut resources,
            log.as_deref_mut(),
            graffiti_map.as_deref_mut(),
            &mut unequip_events,
            &mut farms,
            &mut housing_q,
            &mut taverns,
            &mut offices,
            &corpses,
            &mut graves,
            &mut memories,
            &time,
        );

        if should_remove {
            remove_movement_components(&mut commands, pop_entity);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn process_arrival(
    action: ActionType,
    pop_entity: Entity,
    target_entity: Entity,
    pop_pos: GridPosition,
    target_pos: GridPosition,
    equipment_opt: &mut Option<Mut<Equipment>>,
    chemical_state_opt: &mut Option<Mut<crate::layer1::chemical::ChemicalState>>,
    health_opt: &mut Option<Mut<crate::layer1::health::Health>>,
    stress_opt: &mut Option<Mut<crate::layer1::stress::StressTracker>>,
    items: &Query<&crate::layer1::items::Item>,
    commands: &mut Commands,
    resources: &mut ColonyResources,
    log: Option<&mut MessageLog>,
    graffiti_map: Option<&mut crate::layer1::graffiti::GraffitiMap>,
    unequip_events: &mut EventWriter<UnequipEvent>,
    _farms: &mut Query<&mut Farm>,
    housing_q: &mut Query<&mut Housing>,
    taverns: &mut Query<&mut Tavern>,
    offices: &mut Query<&mut Office>,
    corpses: &Query<&Corpse>,
    graves: &mut Query<(Entity, &GridPosition, &mut Grave)>,
    memories: &mut Query<&mut Memories>,
    time: &Res<SimulationTime>,
) -> bool {
=======
use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use ratatui::style::Color;

use crate::layer1::actions::{AssignedTo, AssignmentType};
use crate::layer1::admin::{AdminProvider, Office};
use crate::layer1::execution::components::{AtTarget, MovementTarget};
use crate::layer1::farm::Farm;
use crate::layer1::funeral::{handle_bury_corpse, Corpse, Grave};
use crate::layer1::housing::Housing;
use crate::layer1::items::{Clothing, ClothingType, Equipment, Item, Tool, ToolType, UnequipEvent};
use crate::layer1::map::GridPosition;
use crate::layer1::memory::Memories;
use crate::layer1::pop::Job;
use crate::layer1::resources::ColonyResources;
use crate::layer1::social::{handle_socialize, Tavern};
use crate::layer1::social_stratification::Prestige;
use crate::layer1::utility_types::ActionType;
use crate::shared::log::MessageLog;
use crate::shared::time::SimulationTime;

#[derive(SystemParam)]
pub struct ArrivalContext<'w, 's> {
    items: Query<'w, 's, &'static crate::layer1::items::Item>,
    farms: Query<'w, 's, &'static mut Farm>,
    housing_q: Query<'w, 's, &'static mut Housing>,
    taverns: Query<'w, 's, &'static mut Tavern>,
    offices: Query<'w, 's, &'static mut Office>,
    corpses: Query<'w, 's, &'static Corpse>,
    graves: Query<'w, 's, (Entity, &'static GridPosition, &'static mut Grave)>,
    memories: Query<'w, 's, &'static mut Memories>,
    resources: ResMut<'w, ColonyResources>,
    log: Option<ResMut<'w, MessageLog>>,
    graffiti_map: Option<ResMut<'w, crate::layer1::graffiti::GraffitiMap>>,
    unequip_events: EventWriter<'w, UnequipEvent>,
    time: Res<'w, SimulationTime>,
}

/// Handles arrival at targets: assigns pops to farms/housing.
#[allow(clippy::type_complexity)]
pub fn arrival_handler_system(
    mut arrivals: Query<
        (
            Entity,
            &GridPosition,
            &MovementTarget,
            Option<&mut Equipment>,
            Option<&mut crate::layer1::chemical::ChemicalState>,
            Option<&mut crate::layer1::health::Health>,
            Option<&mut crate::layer1::stress::StressTracker>,
        ),
        With<AtTarget>,
    >,
    mut ctx: ArrivalContext,
    mut commands: Commands,
) {
    for (
        pop_entity,
        pop_pos,
        mt,
        mut equipment_opt,
        mut chem_opt,
        mut health_opt,
        mut stress_opt,
    ) in &mut arrivals
    {
        let should_remove = process_arrival(
            mt.for_action,
            pop_entity,
            mt.target_entity,
            *pop_pos,
            mt.target_position,
            &mut equipment_opt,
            &mut chem_opt,
            &mut health_opt,
            &mut stress_opt,
            &mut commands,
            &mut ctx,
        );

        if should_remove {
            remove_movement_components(&mut commands, pop_entity);
        }
    }
}

fn process_arrival(
    action: ActionType,
    pop_entity: Entity,
    target_entity: Entity,
    pop_pos: GridPosition,
    target_pos: GridPosition,
    equipment_opt: &mut Option<Mut<Equipment>>,
    chemical_state_opt: &mut Option<Mut<crate::layer1::chemical::ChemicalState>>,
    health_opt: &mut Option<Mut<crate::layer1::health::Health>>,
    stress_opt: &mut Option<Mut<crate::layer1::stress::StressTracker>>,
    commands: &mut Commands,
    ctx: &mut ArrivalContext,
) -> bool {
>>>>>>> REPLACE
<<<<<<< SEARCH
    match action {
        ActionType::ConsumeChemical => {
            handle_consume_chemical_arrival(
                target_entity,
                pop_entity,
                items,
                chemical_state_opt,
                health_opt,
                stress_opt,
                commands,
                time,
            );
            true
        }
        ActionType::ScrawlMemeticSigil => {
            handle_scrawl_memetic_sigil_arrival(target_pos, graffiti_map, log);
            true
        }
        ActionType::Binge => {
            handle_binge_arrival(resources, log);
            true
        }
        ActionType::FetchTool => {
            handle_fetch_tool(commands, resources, pop_entity, equipment_opt);
            true
        }
        ActionType::FetchClothing => {
            handle_fetch_clothing(
                commands,
                resources,
                pop_entity,
                equipment_opt,
                unequip_events,
            );
            true
        }
        ActionType::SatisfyHunger => {
            // SatisfyHunger intentionally assigns no job and no worker slot;
            // the `consume_food_system` globally checks for hungry pops.
            true
        }
        ActionType::SatisfyRest => {
            handle_rest_arrival(pop_entity, target_entity, housing_q, commands);
            true
        }
        ActionType::Socialize => {
            handle_socialize(commands, taverns, target_entity, pop_entity);
            true
        }
        ActionType::SeekMedicalCare => {
            assign_pop(commands, pop_entity, target_entity, AssignmentType::Patient)
        }
        ActionType::Research => assign_pop(
            commands,
            pop_entity,
            target_entity,
            AssignmentType::LibraryWorker,
        ),
        ActionType::Farm => {
            #[allow(clippy::collapsible_if)]
            if let Ok(mut farm) = _farms.get_mut(target_entity) {
                if farm.workers.len() < farm.capacity {
                    farm.workers.push(pop_entity);
                    assign_pop(
                        commands,
                        pop_entity,
                        target_entity,
                        AssignmentType::FarmWorker,
                    );
                }
            }
            true
        }
        ActionType::Admin => {
            if let Ok(mut office) = offices.get_mut(target_entity) {
                if !office.workers.contains(&pop_entity) {
                    office.workers.push(pop_entity);
                }
            }
            assign_pop(
                commands,
                pop_entity,
                target_entity,
                AssignmentType::Administrator,
            )
        }
        ActionType::BuryCorpse => {
            handle_bury_corpse(
                commands,
                corpses,
                graves,
                memories,
                time,
                target_entity,
                pop_entity,
                pop_pos,
            );
            true
        }
        ActionType::Work | ActionType::Repair | ActionType::Haul | ActionType::Tame => {
            // Work/Repair/Haul/Tame is handled by their respective systems
            // Just keep the AtTarget marker for that system
            false
        }
        _ => true,
    }
}
=======
    match action {
        ActionType::ConsumeChemical => {
            handle_consume_chemical_arrival(
                target_entity,
                pop_entity,
                &ctx.items,
                chemical_state_opt,
                health_opt,
                stress_opt,
                commands,
                &ctx.time,
            );
            true
        }
        ActionType::ScrawlMemeticSigil => {
            handle_scrawl_memetic_sigil_arrival(target_pos, ctx.graffiti_map.as_deref_mut(), ctx.log.as_deref_mut());
            true
        }
        ActionType::Binge => {
            handle_binge_arrival(&mut ctx.resources, ctx.log.as_deref_mut());
            true
        }
        ActionType::FetchTool => {
            handle_fetch_tool(commands, &mut ctx.resources, pop_entity, equipment_opt);
            true
        }
        ActionType::FetchClothing => {
            handle_fetch_clothing(
                commands,
                &mut ctx.resources,
                pop_entity,
                equipment_opt,
                &mut ctx.unequip_events,
            );
            true
        }
        ActionType::SatisfyHunger => {
            // SatisfyHunger intentionally assigns no job and no worker slot;
            // the `consume_food_system` globally checks for hungry pops.
            true
        }
        ActionType::SatisfyRest => {
            handle_rest_arrival(pop_entity, target_entity, &mut ctx.housing_q, commands);
            true
        }
        ActionType::Socialize => {
            handle_socialize(commands, &mut ctx.taverns, target_entity, pop_entity);
            true
        }
        ActionType::SeekMedicalCare => {
            assign_pop(commands, pop_entity, target_entity, AssignmentType::Patient)
        }
        ActionType::Research => assign_pop(
            commands,
            pop_entity,
            target_entity,
            AssignmentType::LibraryWorker,
        ),
        ActionType::Farm => {
            #[allow(clippy::collapsible_if)]
            if let Ok(mut farm) = ctx.farms.get_mut(target_entity) {
                if farm.workers.len() < farm.capacity {
                    farm.workers.push(pop_entity);
                    assign_pop(
                        commands,
                        pop_entity,
                        target_entity,
                        AssignmentType::FarmWorker,
                    );
                }
            }
            true
        }
        ActionType::Admin => {
            if let Ok(mut office) = ctx.offices.get_mut(target_entity) {
                if !office.workers.contains(&pop_entity) {
                    office.workers.push(pop_entity);
                }
            }
            assign_pop(
                commands,
                pop_entity,
                target_entity,
                AssignmentType::Administrator,
            )
        }
        ActionType::BuryCorpse => {
            handle_bury_corpse(
                commands,
                &ctx.corpses,
                &mut ctx.graves,
                &mut ctx.memories,
                &ctx.time,
                target_entity,
                pop_entity,
                pop_pos,
            );
            true
        }
        ActionType::Work | ActionType::Repair | ActionType::Haul | ActionType::Tame => {
            // Work/Repair/Haul/Tame is handled by their respective systems
            // Just keep the AtTarget marker for that system
            false
        }
        _ => true,
    }
}
>>>>>>> REPLACE
INNER_EOF
python3 -c '
import sys
with open("src/layer1/execution/arrival.rs", "r") as f: content = f.read()
with open("/tmp/arrival_patch.txt", "r") as f: patch = f.read()

parts = patch.split("<<<<<<< SEARCH\n")
for part in parts[1:]:
    search, rest = part.split("=======\n", 1)
    replace, _ = rest.split(">>>>>>> REPLACE\n", 1)
    content = content.replace(search, replace)

with open("src/layer1/execution/arrival.rs", "w") as f: f.write(content)
'
