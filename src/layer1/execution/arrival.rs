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
    commands: &mut Commands,
    ctx: &mut ArrivalContext,
) -> bool {
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
            handle_scrawl_memetic_sigil_arrival(
                target_pos,
                ctx.graffiti_map.as_deref_mut(),
                ctx.log.as_deref_mut(),
            );
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

fn assign_pop(
    commands: &mut Commands,
    pop_entity: Entity,
    target_entity: Entity,
    assignment_type: AssignmentType,
) -> bool {
    let mut entity_cmds = commands.entity(pop_entity);
    entity_cmds.insert(AssignedTo {
        entity: target_entity,
        assignment_type,
    });

    // If this assignment counts as a Job (persistent employment), update the Job component.
    match assignment_type {
        AssignmentType::FarmWorker
        | AssignmentType::LibraryWorker
        | AssignmentType::ObservatoryWorker => {
            entity_cmds.insert((
                Job {
                    workplace: target_entity,
                    job_type: assignment_type,
                },
                Prestige::from_job(assignment_type),
            ));
        }
        AssignmentType::Administrator => {
            entity_cmds.insert((
                Job {
                    workplace: target_entity,
                    job_type: assignment_type,
                },
                Prestige::from_job(assignment_type),
                AdminProvider { amount: 5.0 },
            ));
        }
        AssignmentType::HousingResident
        | AssignmentType::TavernVisitor
        | AssignmentType::Patient
        | AssignmentType::Funeral
        | AssignmentType::Surgery
        | AssignmentType::Sheriff => {
            // These are not jobs, so we don't update Job component.
            // The pop keeps their previous job (if any).
        }
    }

    true
}

fn remove_movement_components(commands: &mut Commands, pop_entity: Entity) {
    commands
        .entity(pop_entity)
        .remove::<MovementTarget>()
        .remove::<AtTarget>();
}

fn handle_binge_arrival(resources: &mut ColonyResources, log: Option<&mut MessageLog>) {
    let amount_needed = 5.0;
    if resources.food >= amount_needed {
        resources.food -= amount_needed;
    } else {
        let taken = resources.food;
        resources.food = 0.0;
        let remaining = amount_needed - taken;
        if remaining > 0.0 {
            // Subtract remaining from rations
            resources.rations = (resources.rations - remaining).max(0.0);
        }
    }

    if let Some(log) = log {
        log.add("Pop is binge eating!");
    }
}

// --- Consolidated Handler Functions ---

pub fn handle_fetch_tool(
    commands: &mut Commands,
    resources: &mut ColonyResources,
    pop_entity: Entity,
    equipment_opt: &mut Option<Mut<Equipment>>,
) {
    if resources.tools >= 1.0 {
        resources.tools -= 1.0;
        let tool_history = crate::layer1::heirloom::ToolHistory::default();
        let tool_entity = commands
            .spawn((
                Item::default(),
                Tool {
                    tool_type: ToolType::Pickaxe, // Generic for now
                    durability: 100.0,
                    max_durability: 100.0,
                },
                tool_history,
            ))
            .id();

        if let Some(eq) = equipment_opt {
            eq.tool = Some(tool_entity);
        } else {
            commands.entity(pop_entity).insert(Equipment {
                tool: Some(tool_entity),
                ..Default::default()
            });
        }
    }
}

pub fn handle_fetch_clothing(
    commands: &mut Commands,
    resources: &mut ColonyResources,
    pop_entity: Entity,
    equipment_opt: &mut Option<Mut<Equipment>>,
    unequip_events: &mut EventWriter<UnequipEvent>,
) {
    if resources.clothing >= 1.0 {
        resources.clothing -= 1.0;

        let mut is_upgrade = false;
        if let Some(eq) = equipment_opt {
            if eq.body.is_some() {
                is_upgrade = true;
                if let Some(old_entity) = eq.body {
                    unequip_events.send(UnequipEvent {
                        actor: pop_entity,
                        item: old_entity,
                        slot: "body".to_string(),
                    });
                    commands.entity(old_entity).despawn();
                }
            }
        }
        let (clothing_type, insulation) = if is_upgrade {
            (ClothingType::Parka, 2.0)
        } else {
            (ClothingType::Tunic, 1.0)
        };

        let clothing_entity = commands
            .spawn((
                Item::default(),
                Clothing {
                    clothing_type,
                    insulation,
                    durability: 100.0,
                    max_durability: 100.0,
                },
            ))
            .id();

        if let Some(eq) = equipment_opt {
            eq.body = Some(clothing_entity);
        } else {
            commands.entity(pop_entity).insert(Equipment {
                body: Some(clothing_entity),
                ..Default::default()
            });
        }
    }
}

fn handle_rest_arrival(
    pop_entity: Entity,
    target_entity: Entity,
    housing: &mut Query<&mut Housing>,
    commands: &mut Commands,
) {
    if let Ok(mut house) = housing.get_mut(target_entity) {
        if house.residents.len() < house.capacity {
            house.residents.push(pop_entity);
            commands.entity(pop_entity).insert(AssignedTo {
                entity: target_entity,
                assignment_type: AssignmentType::HousingResident,
            });
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_consume_chemical_arrival(
    target_entity: Entity,
    pop_entity: Entity,
    items: &Query<&crate::layer1::items::Item>,
    chemical_state_opt: &mut Option<Mut<crate::layer1::chemical::ChemicalState>>,
    health_opt: &mut Option<Mut<crate::layer1::health::Health>>,
    stress_opt: &mut Option<Mut<crate::layer1::stress::StressTracker>>,
    commands: &mut Commands,
    time: &Res<SimulationTime>,
) {
    if let Ok(item) = items.get(target_entity) {
        let chem_type = match item.item_type {
            crate::layer1::items::ItemType::Stim => {
                Some(crate::layer1::chemical::ChemicalType::Stim)
            }
            crate::layer1::items::ItemType::Sedative => {
                Some(crate::layer1::chemical::ChemicalType::Sedative)
            }
            _ => None,
        };

        if let Some(ct) = chem_type {
            let tick = time.tick;
            if let Some(state) = chemical_state_opt {
                crate::layer1::chemical::consume_chemical_logic(
                    state,
                    ct,
                    tick,
                    health_opt.as_deref_mut(),
                    stress_opt.as_deref_mut(),
                );
            } else {
                let mut state = crate::layer1::chemical::ChemicalState::default();
                crate::layer1::chemical::consume_chemical_logic(
                    &mut state,
                    ct,
                    tick,
                    health_opt.as_deref_mut(),
                    stress_opt.as_deref_mut(),
                );
                commands.entity(pop_entity).insert(state);
            }
            commands.entity(target_entity).despawn();
        }
    }
}

fn handle_scrawl_memetic_sigil_arrival(
    target_pos: GridPosition,
    graffiti_map: Option<&mut crate::layer1::graffiti::GraffitiMap>,
    log: Option<&mut MessageLog>,
) {
    if let Some(map) = graffiti_map {
        use crate::layer1::graffiti::{Graffiti, GraffitiType};
        map.markings.insert(
            (target_pos.x, target_pos.y),
            Graffiti {
                graffiti_type: GraffitiType::MemeticSigil,
                decay: 500.0,
                modifier: -0.2, // Strong debuff
            },
        );
        if let Some(l) = log {
            l.add_colored("A Memetic Sigil has been scrawled on a wall!", Color::Red);
        }
    }
}
