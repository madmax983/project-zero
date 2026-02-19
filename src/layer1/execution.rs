#![allow(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::collapsible_if,
    clippy::missing_panics_doc,
    clippy::cast_precision_loss
)]
//! Execution layer bridging utility AI decisions to actual pop actions.
//!
//! This module handles:
//! - Movement toward targets
//! - Assigning pops to farms/housing
//! - Executing work at designations (mining, chopping)
//!
//! # System Flow
//!
//! ```text
//! evaluate_actions_system (existing)
//!     ↓ inserts StartPlan
//! cleanup_previous_assignment_system
//!     ↓ removes pop from old farm/housing
//! process_start_plan_system
//!     ↓ converts StartPlan → MovementTarget
//! movement_system
//!     ↓ moves pop 1 tile, inserts AtTarget on arrival
//! arrival_handler_system
//!     ↓ assigns to farm/housing
//! work_execution_system
//!     ↓ calls mine_rock/chop_tree
//! ```

use crate::layer1::access_control::{AccessControl, AccessMode};
use crate::layer1::actions::fetch_clothing::handle_fetch_clothing;
use crate::layer1::actions::fetch_tool::handle_fetch_tool;
use crate::layer1::actions::hunger::handle_arrival as handle_hunger_arrival;
use crate::layer1::actions::rest::handle_arrival as handle_rest_arrival;
use crate::layer1::actions::{AssignedTo, AssignmentType};
use crate::layer1::building::{Building, OccupiedTiles};
use crate::layer1::combat::{HitStop, Weapon};
use crate::layer1::cybernetics::get_efficiency_bonus;
use crate::layer1::day_night::DayNightCycle;
use crate::layer1::defense::Gate;
use crate::layer1::designation::{Designation, DesignationType};
use crate::layer1::edicts::{ColonyPolicies, get_work_speed_modifier};
use crate::layer1::erosion::{ErosionGrid, MOVEMENT_EROSION_AMOUNT};
use crate::layer1::farm::Farm;
use crate::layer1::flora::process_flora_clearing;
use crate::layer1::funeral::{Corpse, Grave, handle_bury_corpse};
use crate::layer1::hazards::handle_workplace_hazards;
use crate::layer1::heirloom::{Heirloom, RetrogradeEngineeringEvent, ToolHistory};
use crate::layer1::housing::Housing;
use crate::layer1::items::{Equipment, Tool};
use crate::layer1::map::{GridPosition, ScreenShake};
use crate::layer1::memory::{Memories, calculate_effective_morale};
use crate::layer1::morale::Morale;
use crate::layer1::needs::{Needs, get_morale_efficiency};
use crate::layer1::particles::spawn_particle;
use crate::layer1::pop::{Job, Role, Speed};
use crate::layer1::resources::{ColonyResources, process_logging, process_mining};
use crate::layer1::ruins::Ruin;
use crate::layer1::skills::{SkillType, Skills, get_skill_efficiency};
use crate::layer1::social::{SocialBuff, Tavern, handle_socialize};
use crate::layer1::social_stratification::Prestige;
use crate::layer1::terrain::{TerrainGrid, TerrainType};
use crate::layer1::traits::{Traits, get_trait_move_speed_modifier, get_trait_work_speed_modifier};
use crate::layer1::utility_types::{ActionType, PopAction, StartPlan};
use crate::shared::log::MessageLog;
use crate::shared::time::SimulationTime;
use bevy_ecs::prelude::*;
use rand::Rng;
use ratatui::style::Color;

/// Executes combat when pop is targeting an enemy.
pub fn combat_execution_system(world: &mut World) {
    // Collect combatants
    let combatants: Vec<(Entity, Entity, Option<Equipment>)> = world
        .query::<(
            Entity,
            &MovementTarget,
            Option<&Equipment>,
            Option<&HitStop>,
        )>()
        .iter(world)
        .filter(|(_, mt, _, hit_stop)| {
            // Ludwig: Check Hit Stop
            if let Some(hs) = hit_stop {
                if hs.ticks_remaining > 0 {
                    return false;
                }
            }
            mt.for_action == ActionType::Fight
        })
        .map(|(e, mt, eq, _)| (e, mt.target_entity, eq.copied()))
        .collect();

    for (pop_entity, target_entity, equipment_opt) in combatants {
        process_single_combatant(world, pop_entity, target_entity, equipment_opt);
    }
}

fn process_single_combatant(
    world: &mut World,
    pop_entity: Entity,
    target_entity: Entity,
    equipment_opt: Option<Equipment>,
) {
    // Find target position (it might have moved)
    let target_pos = if let Some(pos) = world.get::<GridPosition>(target_entity) {
        *pos
    } else {
        // Target despawned?
        cleanup_pop_work_state(world, pop_entity);
        return;
    };

    // Update MovementTarget if needed
    if let Some(mut mt) = world.get_mut::<MovementTarget>(pop_entity) {
        if mt.target_position != target_pos {
            mt.target_position = target_pos;
            // Remove AtTarget to ensure we chase if they moved away
            // But only if we are now out of range?
            // Actually, let's check range first.
        }
    }

    // Check range
    // Safety: Pop must have GridPosition
    let Some(pop_pos) = world.get::<GridPosition>(pop_entity).copied() else {
        return;
    };
    let dist = pop_pos.distance_chebyshev(target_pos) as f32;

    let mut weapon_range = 1.0; // Default melee
    if let Some(ref eq) = equipment_opt {
        if let Some(weapon_entity) = eq.weapon {
            if let Some(weapon) = world.get::<Weapon>(weapon_entity) {
                weapon_range = weapon.properties.range;
            }
        }
    }

    if dist <= weapon_range {
        // In range!
        // Stop movement
        if world.get::<AtTarget>(pop_entity).is_none() {
            world.entity_mut(pop_entity).insert(AtTarget);
        }

        // Attack
        crate::layer1::combat::execute_attack(world, pop_entity, target_entity);
    } else {
        // Out of range
        // Ensure we are moving (remove AtTarget if present)
        world.entity_mut(pop_entity).remove::<AtTarget>();
    }
}

/// Executes vandalism when pop is at target with Vandalize action.
pub fn vandalize_execution_system(world: &mut World) {
    // Find pops at target with Vandalize action
    let vandals: Vec<(Entity, Entity)> = world
        .query_filtered::<(Entity, &MovementTarget), With<AtTarget>>()
        .iter(world)
        .filter(|(_, mt)| mt.for_action == ActionType::Vandalize)
        .map(|(e, mt)| (e, mt.target_entity))
        .collect();

    for (pop_entity, target_entity) in vandals {
        crate::layer1::unrest::perform_vandalize_logic(world, pop_entity, target_entity);

        // If target is destroyed (removed from world), stop vandalizing
        // Since perform_vandalize_logic currently only reduces HP, the target remains.
        // We assume another system handles structure destruction at 0 HP (if exists),
        // or we should handle it here.
        // For MVP Unrest, simple HP reduction is enough to satisfy the test.
    }
}

/// Work amount applied per tick when a pop is working.
const WORK_PER_TICK: f32 = 10.0;

/// Durability loss per tick when working.
const TOOL_DURABILITY_LOSS: f32 = 0.1;

/// Efficiency multiplier when working without tools.
const NO_TOOL_PENALTY: f32 = 0.5;

/// Component indicating a pop is moving toward a target.
#[derive(Component, Debug)]
pub struct MovementTarget {
    /// The entity being targeted (farm, housing, or designation).
    pub target_entity: Entity,
    /// The grid position of the target.
    pub target_position: GridPosition,
    /// The action type this movement is for.
    pub for_action: ActionType,
}

/// Marker component indicating a pop has arrived at its target.
#[derive(Component, Debug)]
pub struct AtTarget;

/// Removes pops from farms/housing when they switch to a different action.
///
/// This system runs before `process_start_plan_system` to ensure pops are
/// properly removed from their previous assignment before moving to a new one.
pub fn cleanup_previous_assignment_system(
    pops_query: Query<(Entity, &AssignedTo), With<StartPlan>>,
    mut farms: Query<&mut Farm>,
    mut housing: Query<&mut Housing>,
    mut taverns: Query<&mut Tavern>,
    mut commands: Commands,
) {
    for (pop_entity, assigned) in &pops_query {
        let assigned_entity = assigned.entity;
        match assigned.assignment_type {
            AssignmentType::FarmWorker => {
                if let Ok(mut farm) = farms.get_mut(assigned_entity) {
                    farm.workers.retain(|&w| w != pop_entity);
                }
            }
            AssignmentType::HousingResident => {
                if let Ok(mut h) = housing.get_mut(assigned_entity) {
                    h.residents.retain(|&r| r != pop_entity);
                }
            }
            AssignmentType::TavernVisitor => {
                if let Ok(mut tavern) = taverns.get_mut(assigned_entity) {
                    tavern.visitors.retain(|&v| v != pop_entity);
                }
            }
            AssignmentType::LibraryWorker
            | AssignmentType::Patient
            | AssignmentType::Funeral
            | AssignmentType::ObservatoryWorker
            | AssignmentType::Miner
            | AssignmentType::Hauler
            | AssignmentType::Builder
            | AssignmentType::Crafter
            | AssignmentType::Guard
            | AssignmentType::Engineer
            | AssignmentType::Doctor
            | AssignmentType::Merchant
            | AssignmentType::Scientist
            | AssignmentType::Artist
            | AssignmentType::Governor
            | AssignmentType::Administrator
            | AssignmentType::Surgery => {}
        }

        commands.entity(pop_entity).remove::<AssignedTo>();
    }
}

/// Consumes `StartPlan` markers and creates `MovementTarget` components.
///
/// This system bridges the utility AI's decision (`StartPlan`) with the
/// movement system by creating `MovementTarget` for each pop.
pub fn process_start_plan_system(
    plans: Query<(Entity, &StartPlan)>,
    positions: Query<&GridPosition>,
    mut commands: Commands,
) {
    for (pop_entity, start_plan) in &plans {
        let action = start_plan.action;
        let target = start_plan.target;

        // Remove the StartPlan marker and any existing movement components
        commands
            .entity(pop_entity)
            .remove::<StartPlan>()
            .remove::<MovementTarget>()
            .remove::<AtTarget>();

        // If there's no target, skip (e.g., Idle action)
        let Some(target_entity) = target else {
            continue;
        };

        // Get the target's position (also validates entity existence)
        let Ok(&target_position) = positions.get(target_entity) else {
            continue;
        };

        // Insert new MovementTarget
        commands.entity(pop_entity).insert(MovementTarget {
            target_entity,
            target_position,
            for_action: action,
        });
    }
}

/// Moves pops 1 tile per tick toward their target (Manhattan-style).
///
/// When a pop arrives at its target position (or adjacent for work), this system
/// marks it with `AtTarget`.
#[allow(clippy::type_complexity)]
pub fn movement_system(
    mut pops: Query<
        (
            Entity,
            &mut GridPosition,
            &MovementTarget,
            Option<&mut Speed>,
            Option<&Traits>,
            Option<&HitStop>,
            Option<&Role>,
        ),
        (Without<AtTarget>, Without<Building>),
    >,
    mut erosion: ResMut<ErosionGrid>,
    terrain: Res<TerrainGrid>,
    occupied_tiles: Option<Res<OccupiedTiles>>,
    buildings: Query<(
        &GridPosition,
        &Building,
        Option<&Gate>,
        Option<&AccessControl>,
    )>,
    mut commands: Commands,
) {
    for (pop_entity, mut current_pos, mt, mut speed_opt, traits, hit_stop, role) in &mut pops {
        // Ludwig: Check Hit Stop
        if let Some(hs) = hit_stop {
            if hs.ticks_remaining > 0 {
                continue;
            }
        }

        let trait_mod = traits.map_or(1.0, get_trait_move_speed_modifier);

        // Accumulate speed
        if let Some(ref mut speed) = speed_opt {
            speed.accumulator += speed.current * trait_mod;
        }

        let target_pos = mt.target_position;
        let action = mt.for_action;

        // Check pre-move adjacency for work
        if check_work_adjacency(
            *current_pos,
            target_pos,
            action,
            &terrain,
            occupied_tiles.as_deref(),
            &buildings,
            pop_entity,
            role.copied(),
        ) {
            commands.entity(pop_entity).insert(AtTarget);
            continue;
        }

        // Check if already at target (for non-work actions requiring exact position)
        if *current_pos == target_pos {
            commands.entity(pop_entity).insert(AtTarget);
            continue;
        }

        let (primary, secondary) = calculate_next_positions(*current_pos, target_pos);

        // Selection Phase: Find first walkable candidate
        let chosen_pos = try_get_walkable_pos(
            primary,
            &terrain,
            occupied_tiles.as_deref(),
            &buildings,
            pop_entity,
            role.copied(),
        )
        .or_else(|| {
            try_get_walkable_pos(
                secondary,
                &terrain,
                occupied_tiles.as_deref(),
                &buildings,
                pop_entity,
                role.copied(),
            )
        });

        let Some(new_pos) = chosen_pos else {
            continue;
        };

        // Determine movement cost
        let movement_cost =
            if let (Ok(x), Ok(y)) = (usize::try_from(new_pos.x), usize::try_from(new_pos.y)) {
                terrain
                    .get(x, y)
                    .map_or(1.0, crate::layer1::terrain::TerrainType::movement_cost)
            } else {
                1.0
            };

        // Check if we can move
        let can_move = if let Some(ref mut speed) = speed_opt {
            // Ludwig: "Coyote Speed" - Allow moving if we are *almost* there.
            // This prevents the feeling of "just missing the bus" by 0.01 speed.
            // Increased to 0.15 for better flow.
            const COYOTE_THRESHOLD: f32 = 0.15;
            if speed.accumulator >= (movement_cost - COYOTE_THRESHOLD) {
                speed.accumulator -= movement_cost;
                true
            } else {
                false
            }
        } else {
            true
        };

        if !can_move {
            continue;
        }

        current_pos.x = new_pos.x;
        current_pos.y = new_pos.y;

        // Apply Erosion
        if let (Ok(x), Ok(y)) = (usize::try_from(new_pos.x), usize::try_from(new_pos.y)) {
            erosion.add_erosion(x, y, MOVEMENT_EROSION_AMOUNT);
        }

        if new_pos == target_pos {
            commands.entity(pop_entity).insert(AtTarget);
        }

        // Check post-move adjacency for work
        if check_work_adjacency(
            new_pos,
            target_pos,
            action,
            &terrain,
            occupied_tiles.as_deref(),
            &buildings,
            pop_entity,
            role.copied(),
        ) {
            commands.entity(pop_entity).insert(AtTarget);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn check_work_adjacency(
    current_pos: GridPosition,
    target_pos: GridPosition,
    action: ActionType,
    terrain: &TerrainGrid,
    occupied: Option<&OccupiedTiles>,
    buildings: &Query<(
        &GridPosition,
        &Building,
        Option<&Gate>,
        Option<&AccessControl>,
    )>,
    pop_entity: Entity,
    pop_role: Option<Role>,
) -> bool {
    if action != ActionType::Work && action != ActionType::Repair {
        return false;
    }

    if is_walkable(
        terrain,
        occupied,
        buildings,
        target_pos.x,
        target_pos.y,
        pop_entity,
        pop_role,
    ) {
        return false;
    }

    let distance = (current_pos.x - target_pos.x).abs() + (current_pos.y - target_pos.y).abs();
    distance == 1
}

/// Handles arrival at targets: assigns pops to farms/housing.
pub fn arrival_handler_system(
    mut arrivals: Query<
        (
            Entity,
            &GridPosition,
            &MovementTarget,
            Option<&mut Equipment>,
        ),
        With<AtTarget>,
    >,
    mut farms: Query<&mut Farm>,
    mut housing_q: Query<&mut Housing>,
    mut taverns: Query<&mut Tavern>,
    corpses: Query<&Corpse>,
    mut graves: Query<(Entity, &GridPosition, &mut Grave)>,
    mut memories: Query<&mut Memories>,
    mut resources: ResMut<ColonyResources>,
    mut log: Option<ResMut<MessageLog>>,
    time: Res<SimulationTime>,
    mut commands: Commands,
) {
    for (pop_entity, pop_pos, mt, mut equipment_opt) in &mut arrivals {
        let should_remove = process_arrival(
            mt.for_action,
            pop_entity,
            mt.target_entity,
            *pop_pos,
            &mut equipment_opt,
            &mut commands,
            &mut resources,
            log.as_deref_mut(),
            &mut farms,
            &mut housing_q,
            &mut taverns,
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

fn process_arrival(
    action: ActionType,
    pop_entity: Entity,
    target_entity: Entity,
    pop_pos: GridPosition,
    equipment_opt: &mut Option<Mut<Equipment>>,
    commands: &mut Commands,
    resources: &mut ColonyResources,
    log: Option<&mut MessageLog>,
    farms: &mut Query<&mut Farm>,
    housing_q: &mut Query<&mut Housing>,
    taverns: &mut Query<&mut Tavern>,
    corpses: &Query<&Corpse>,
    graves: &mut Query<(Entity, &GridPosition, &mut Grave)>,
    memories: &mut Query<&mut Memories>,
    time: &Res<SimulationTime>,
) -> bool {
    match action {
        ActionType::Binge => {
            handle_binge_arrival(resources, log);
            true
        }
        ActionType::FetchTool => {
            handle_fetch_tool(commands, resources, pop_entity, equipment_opt);
            true
        }
        ActionType::FetchClothing => {
            handle_fetch_clothing(commands, resources, pop_entity, equipment_opt);
            true
        }
        ActionType::SatisfyHunger => {
            handle_hunger_arrival(pop_entity, target_entity, farms, commands);
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
        | AssignmentType::ObservatoryWorker
        | AssignmentType::Miner
        | AssignmentType::Hauler
        | AssignmentType::Builder
        | AssignmentType::Crafter
        | AssignmentType::Guard
        | AssignmentType::Engineer
        | AssignmentType::Doctor
        | AssignmentType::Merchant
        | AssignmentType::Scientist
        | AssignmentType::Artist
        | AssignmentType::Governor
        | AssignmentType::Administrator => {
            entity_cmds.insert((
                Job {
                    workplace: target_entity,
                    job_type: assignment_type,
                },
                Prestige::from_job(assignment_type),
            ));
        }
        AssignmentType::HousingResident
        | AssignmentType::TavernVisitor
        | AssignmentType::Patient
        | AssignmentType::Funeral
        | AssignmentType::Surgery => {
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

fn try_get_walkable_pos(
    pos: Option<GridPosition>,
    terrain: &TerrainGrid,
    occupied_tiles: Option<&OccupiedTiles>,
    buildings: &Query<(
        &GridPosition,
        &Building,
        Option<&Gate>,
        Option<&AccessControl>,
    )>,
    pop_entity: Entity,
    pop_role: Option<Role>,
) -> Option<GridPosition> {
    let p = pos?;
    if is_walkable(
        terrain,
        occupied_tiles,
        buildings,
        p.x,
        p.y,
        pop_entity,
        pop_role,
    ) {
        Some(p)
    } else {
        None
    }
}

fn is_walkable(
    terrain: &TerrainGrid,
    occupied: Option<&OccupiedTiles>,
    buildings: &Query<(
        &GridPosition,
        &Building,
        Option<&Gate>,
        Option<&AccessControl>,
    )>,
    x: i32,
    y: i32,
    pop_entity: Entity,
    pop_role: Option<Role>,
) -> bool {
    // Check Terrain bounds and type
    let Ok(x_idx) = usize::try_from(x) else {
        return false;
    };
    let Ok(y_idx) = usize::try_from(y) else {
        return false;
    };

    if !terrain
        .get(x_idx, y_idx)
        .is_some_and(TerrainType::is_walkable)
    {
        return false;
    }

    // Check Buildings
    let Some(occupied_tiles) = occupied else {
        return true;
    };
    if !occupied_tiles.0.contains(&(x, y)) {
        return true;
    }

    for (pos, building, _gate, access_opt) in buildings.iter() {
        if pos.x == x && pos.y == y {
            // Priority: AccessControl
            if let Some(access) = access_opt {
                return match access.mode {
                    AccessMode::Public => true,
                    AccessMode::Lockdown => false,
                    AccessMode::Restricted => {
                        access.allowed_pops.contains(&pop_entity)
                            || pop_role.is_some_and(|r| access.allowed_roles.contains(&r))
                    }
                };
            }

            // Fallback: Gate (Legacy) - deprecated but kept for safety if AccessControl missing
            // If we decide to fully remove logic, we can just skip this.
            // But spec said "Replace Gate.is_locked".
            // Since we are adding AccessControl to Gates, we should rely on AccessControl.
            // If AccessControl is missing, we check building.is_obstacle().
            // Gate is an obstacle.
            if building.building_type.is_obstacle() {
                return false;
            }
            return true;
        }
    }
    true
}

#[allow(clippy::missing_const_for_fn, clippy::unnecessary_wraps)]
fn calculate_next_positions(
    current: GridPosition,
    target: GridPosition,
) -> (Option<GridPosition>, Option<GridPosition>) {
    // Calculate movement direction (Manhattan)
    let dx = (target.x - current.x).signum();
    let dy = (target.y - current.y).signum();

    let move_x = if dx != 0 {
        Some(GridPosition {
            x: current.x + dx,
            y: current.y,
        })
    } else {
        None
    };

    let move_y = if dy != 0 {
        Some(GridPosition {
            x: current.x,
            y: current.y + dy,
        })
    } else {
        None
    };

    // Prefer horizontal movement
    if dx != 0 {
        (move_x, move_y)
    } else {
        (move_y, None)
    }
}

/// Executes the demolition of a building at the designation's location.
///
/// If the building is an Ancient Structure, this triggers "Retrograde Engineering",
/// awarding Knowledge instead of resources/debris.
pub fn execute_demolish(world: &mut World, designation_entity: Entity) -> bool {
    // Find designation position
    world
        .get::<GridPosition>(designation_entity)
        .copied()
        .is_some_and(|designation_pos| {
            // 1. Check for Ruin first (Scavenging)
            let ruin_entity = world
                .query::<(Entity, &GridPosition, &Ruin)>()
                .iter(world)
                .find(|(_, pos, _)| **pos == designation_pos)
                .map(|(e, _, _)| e);

            if let Some(ruin) = ruin_entity {
                let yielded = crate::layer1::ruins::process_scavenge(world, ruin);
                if !yielded.is_empty() {
                    if let Some(mut log) = world.get_resource_mut::<MessageLog>() {
                        log.add_colored("Scavenged resources from Ruin.", Color::Green);
                    }
                }
                // Despawn the designation itself
                world.despawn(designation_entity);
                return true;
            }

            // 2. Check for Building (Existing logic)
            // We collect to avoid borrow issues if we need to mutate world later
            let building_entity = world
                .query::<(Entity, &GridPosition, &Building)>()
                .iter(world)
                .find(|(_, pos, _)| pos.x == designation_pos.x && pos.y == designation_pos.y)
                .map(|(e, _, _)| e);

            if let Some(entity) = building_entity {
                // Check for AncientStructure before despawn
                let is_ancient = world
                    .get::<crate::layer1::heirloom::AncientStructure>(entity)
                    .is_some();
                let building_type = world.get::<Building>(entity).map(|b| b.building_type);

                if is_ancient {
                    // Retrograde Engineering: Award Knowledge
                    let amount = calculate_knowledge_reward(building_type);

                    if let Some(mut res) = world.get_resource_mut::<ColonyResources>() {
                        res.add_knowledge(amount);
                    }

                    let label = building_type.map_or_else(|| "Ancient Structure".to_string(), |b| b.label().to_string());

                    // Fire integration event
                    world.send_event(RetrogradeEngineeringEvent {
                        building_label: label.clone(),
                        knowledge_gained: amount,
                    });

                    if let Some(mut log) = world.get_resource_mut::<MessageLog>() {
                        log.add_colored(
                            format!(
                                "Retrograde Engineering: Deconstructed {label} for {amount} Knowledge."
                            ),
                            Color::Cyan,
                        );
                    }

                    // Cyan 'data' sparks
                    spawn_particle(world, designation_pos, '?', Color::Cyan, 15);
                } else {
                    // Normal Debris
                    spawn_particle(world, designation_pos, 'X', Color::Red, 10);
                }

                world.despawn(entity);

                // Trigger Screen Shake (Ludwig: "Juice")
                if let Some(mut shake) = world.get_resource_mut::<ScreenShake>() {
                    shake.trigger(0.5);
                }

                // Remove from OccupiedTiles
                if let Some(mut occupied) = world.get_resource_mut::<OccupiedTiles>() {
                    occupied.0.remove(&(designation_pos.x, designation_pos.y));
                }
            }

            // Despawn the designation itself
            world.despawn(designation_entity);
            true
        })
}

/// Executes work at designations when pop is at target with Work action.
pub fn work_execution_system(world: &mut World) {
    let policies = world.get_resource::<ColonyPolicies>().cloned();
    let work_speed_mod = policies.as_ref().map_or(1.0, get_work_speed_modifier);

    // Fetch DayNightCycle
    let cycle = world.get_resource::<DayNightCycle>().map(|c| c.time_of_day);

    // Fetch Factions for strike check
    // We collect striking factions into a set to avoid borrowing conflicts with world
    let striking_factions: std::collections::HashSet<crate::layer1::factions::FactionId> = world
        .get_resource::<crate::layer1::factions::Factions>()
        .map(|f| {
            f.map
                .iter()
                .filter(|(_, d)| d.state == crate::layer1::factions::FactionState::Striking)
                .map(|(id, _)| *id)
                .collect()
        })
        .unwrap_or_default();

    // Find pops at their work target and capture their morale
    // Since we need to access Needs which is a component, and we need &mut World later,
    // we should collect Needs data first.
    let workers_data: Vec<(Entity, Entity, f32, ActionType, Option<Equipment>, f32)> = world
        .query_filtered::<(
            Entity,
            &MovementTarget,
            Option<&Needs>,
            Option<&Memories>,
            Option<&SocialBuff>,
            Option<&Equipment>,
            Option<&Traits>,
            Option<&Morale>,
            Option<&crate::layer1::factions::FactionMember>,
        ), With<AtTarget>>()
        .iter(world)
        .filter(|(_, mt, _, _, _, _, _, _, faction_member)| {
            let is_work = mt.for_action == ActionType::Work || mt.for_action == ActionType::Repair;
            if !is_work {
                return false;
            }

            // Check strike
            let is_striking = faction_member
                .and_then(|m| m.faction_id)
                .is_some_and(|fid| striking_factions.contains(&fid));

            !is_striking
        })
        .map(
            |(e, mt, needs, memories, social_buff, eq, traits, morale_comp, _)| {
                let morale = needs.map_or(0.5, |n| {
                    calculate_effective_morale(
                        n,
                        memories,
                        social_buff,
                        policies.as_ref(),
                        traits,
                        cycle,
                        morale_comp,
                    )
                });
                let trait_work_mod = traits.map_or(1.0, get_trait_work_speed_modifier);
                (
                    e,
                    mt.target_entity,
                    morale,
                    mt.for_action,
                    eq.copied(),
                    trait_work_mod,
                )
            },
        )
        .collect();

    for (pop_entity, designation_entity, morale, action_type, equipment_opt, trait_work_mod) in
        workers_data
    {
        process_single_worker(
            world,
            pop_entity,
            designation_entity,
            morale,
            action_type,
            equipment_opt,
            work_speed_mod * trait_work_mod,
        );
    }
}

fn process_single_worker(
    world: &mut World,
    pop_entity: Entity,
    designation_entity: Entity,
    morale: f32,
    action_type: ActionType,
    equipment_opt: Option<Equipment>,
    work_speed_mod: f32,
) {
    // Check if designation/target still exists (early exit)
    // We need to check existence first because we need the component later
    if world.get_entity(designation_entity).is_err() {
        cleanup_pop_work_state(world, pop_entity);
        return;
    }

    // Get designation type or infer from target
    let designation_type = if let Some(des) = world.get::<Designation>(designation_entity) {
        des.designation_type
    } else if world
        .get::<crate::layer1::structure::Structure>(designation_entity)
        .is_some()
        && action_type == ActionType::Repair
    {
        // Implicit repair designation for structures
        DesignationType::Repair
    } else {
        // Invalid target type
        cleanup_pop_work_state(world, pop_entity);
        return;
    };

    // Check per-pop tool availability
    let tool_entity_opt = equipment_opt.as_ref().and_then(|e| e.tool);

    // Update Tool History
    if let Some(tool_entity) = tool_entity_opt {
        if let Some(mut history) = world.get_mut::<ToolHistory>(tool_entity) {
            history.ticks_used += 1;
        }
    }

    // Calculate Work Amount
    let work_amount = calculate_work_amount(
        world,
        pop_entity,
        designation_type,
        tool_entity_opt,
        morale,
        work_speed_mod,
    );

    // Execute Work
    let worked =
        execute_work_on_designation(world, designation_entity, designation_type, work_amount);

    // After work: Check if target is "done"
    let target_gone = world.get_entity(designation_entity).is_err();
    let structure_full = if !target_gone && designation_type == DesignationType::Repair {
        world
            .get::<crate::layer1::structure::Structure>(designation_entity)
            .is_some_and(|s| (s.current_hp - s.max_hp).abs() < f32::EPSILON)
    } else {
        false
    };

    if target_gone || structure_full {
        cleanup_pop_work_state(world, pop_entity);
    }

    // Post-work effects (XP, Hazards, Durability)
    if worked {
        handle_post_work_effects(
            world,
            pop_entity,
            designation_entity,
            designation_type,
            action_type,
            tool_entity_opt,
        );
    }
}

const fn get_skill_for_designation(designation_type: DesignationType) -> Option<SkillType> {
    match designation_type {
        DesignationType::Mine => Some(SkillType::Mining),
        DesignationType::Chop => Some(SkillType::Forestry),
        DesignationType::Repair
        | DesignationType::Demolish
        | DesignationType::JuryRig
        | DesignationType::Cannibalize => Some(SkillType::Construction),
        DesignationType::ClearFlora => Some(SkillType::Farming),
        DesignationType::SetZone(_) | DesignationType::Tame => None,
    }
}

fn calculate_work_amount(
    world: &World,
    pop_entity: Entity,
    designation_type: DesignationType,
    tool_entity: Option<Entity>,
    morale: f32,
    work_speed_mod: f32,
) -> f32 {
    let mut tool_efficiency = if tool_entity.is_some() {
        1.0
    } else {
        NO_TOOL_PENALTY
    };

    // Apply Heirloom bonus
    if let Some(entity) = tool_entity {
        if let Some(heirloom) = world.get::<Heirloom>(entity) {
            tool_efficiency *= 1.0 + heirloom.efficiency_bonus;
        }
    }

    let skill_type = get_skill_for_designation(designation_type);

    let skill_efficiency = {
        let skills = world.get::<Skills>(pop_entity);
        skill_type.map_or(1.0, |st| get_skill_efficiency(skills, st))
    };

    let morale_efficiency = get_morale_efficiency(morale);

    // Ludwig: Add organic variation to work speed (0.9 - 1.1) so pops don't feel robotic
    let mut rng = rand::thread_rng();
    let organic_factor = rng.gen_range(0.9..1.1);

    let augmentation_bonus = get_efficiency_bonus(world, pop_entity);

    WORK_PER_TICK
        * tool_efficiency
        * morale_efficiency
        * skill_efficiency
        * work_speed_mod
        * (1.0 + augmentation_bonus)
        * organic_factor
}

/// Executes the cannibalization of the Lander.
///
/// This destroys the Lander and spawns a large amount of resources.
pub fn execute_cannibalize(world: &mut World, designation_entity: Entity) -> bool {
    let Some(designation_pos) = world.get::<GridPosition>(designation_entity).copied() else {
        return false;
    };

    // Find building at this position
    let building_entity = world
        .query::<(Entity, &GridPosition, &crate::layer1::building::Building)>()
        .iter(world)
        .find(|(_, pos, b)| {
            pos.x == designation_pos.x
                && pos.y == designation_pos.y
                && b.building_type == crate::layer1::building::BuildingType::Lander
        })
        .map(|(e, _, _)| e);

    if let Some(entity) = building_entity {
        // Spawn Resources
        spawn_resource_pile(
            world,
            designation_pos,
            crate::layer1::resources::ResourceType::Metal,
            100.0,
        );
        spawn_resource_pile(
            world,
            designation_pos,
            crate::layer1::resources::ResourceType::Fuel,
            50.0,
        );
        spawn_resource_pile(
            world,
            designation_pos,
            crate::layer1::resources::ResourceType::Rations,
            50.0,
        );

        if let Some(mut log) = world.get_resource_mut::<crate::shared::log::MessageLog>() {
            log.add_colored(
                "Lander cannibalized! Massive resources gained.",
                ratatui::style::Color::Yellow,
            );
        }

        // VFX
        crate::layer1::particles::spawn_particle(
            world,
            designation_pos,
            'X',
            ratatui::style::Color::Red,
            20,
        );
        if let Some(mut shake) = world.get_resource_mut::<crate::layer1::map::ScreenShake>() {
            shake.trigger(0.8);
        }

        // Cleanup
        world.despawn(entity);
        if let Some(mut occupied) =
            world.get_resource_mut::<crate::layer1::building::OccupiedTiles>()
        {
            occupied.0.remove(&(designation_pos.x, designation_pos.y));
        }

        world.despawn(designation_entity);
        return true;
    }

    // If we are here, we didn't find a Lander (maybe destroyed already)
    // Clean up designation anyway
    world.despawn(designation_entity);
    false
}

fn spawn_resource_pile(
    world: &mut World,
    pos: GridPosition,
    res_type: crate::layer1::resources::ResourceType,
    amount: f32,
) {
    world.spawn((
        crate::layer1::resources::ResourceItem {
            resource_type: res_type,
            amount,
        },
        pos,
    ));
}

fn execute_work_on_designation(
    world: &mut World,
    designation_entity: Entity,
    designation_type: DesignationType,
    work_amount: f32,
) -> bool {
    // Ludwig: Get position for juice effects
    let pos = world.get::<GridPosition>(designation_entity).copied();

    match designation_type {
        DesignationType::Mine => handle_mining_work(world, designation_entity, work_amount, pos),
        DesignationType::Chop => handle_chopping_work(world, designation_entity, work_amount, pos),
        DesignationType::Demolish => execute_demolish(world, designation_entity),
        DesignationType::Repair => {
            crate::layer1::structure::process_repair(world, designation_entity, work_amount);
            true
        }
        DesignationType::ClearFlora => {
            process_flora_clearing(world, designation_entity, work_amount);
            true
        }
        DesignationType::JuryRig => execute_jury_rig(world, designation_entity),
        DesignationType::Cannibalize => execute_cannibalize(world, designation_entity),
        DesignationType::SetZone(_) | DesignationType::Tame => false,
    }
}

fn handle_mining_work(
    world: &mut World,
    entity: Entity,
    work_amount: f32,
    pos: Option<GridPosition>,
) -> bool {
    let mut rng = rand::thread_rng();
    let is_crit = rng.gen_bool(0.05);

    let mut effective_work = work_amount;
    if is_crit {
        effective_work *= 5.0;
        if let Some(p) = pos {
            spawn_particle(world, p, '*', Color::Yellow, 10);
            trigger_shake(world, 0.3);
        }
        if let Some(mut log) = world.get_resource_mut::<MessageLog>() {
            log.add_colored("Critical Mine!", Color::Yellow);
        }
    }

    if let Some(p) = pos {
        crate::layer1::geology::add_seismic_stress(world, p, 1.0);
    }

    process_mining(world, entity, effective_work);

    if let Some(p) = pos {
        if world.get_entity(entity).is_err() {
            // Finished: Big shake + Debris
            trigger_shake(world, 0.5);
            spawn_particle(world, p, '*', Color::White, 10);
        } else {
            // Working: Dynamic shake + Dust
            if !is_crit {
                let intensity = world
                    .get::<crate::layer1::resources::MiningProgress>(entity)
                    .map_or(0.05, |prog| (prog.current / prog.max).mul_add(0.15, 0.05));

                trigger_shake(world, intensity);
                spawn_particle(world, p, '.', Color::DarkGray, 3);
            }
        }
    }
    true
}

fn handle_chopping_work(
    world: &mut World,
    entity: Entity,
    work_amount: f32,
    pos: Option<GridPosition>,
) -> bool {
    let mut rng = rand::thread_rng();
    let is_crit = rng.gen_bool(0.05);

    let mut effective_work = work_amount;
    if is_crit {
        effective_work *= 5.0;
        if let Some(p) = pos {
            spawn_particle(world, p, '^', Color::LightGreen, 10);
            trigger_shake(world, 0.3);
        }
        if let Some(mut log) = world.get_resource_mut::<MessageLog>() {
            log.add_colored("Critical Chop!", Color::LightGreen);
        }
    }

    process_logging(world, entity, effective_work);

    if let Some(p) = pos {
        if world.get_entity(entity).is_err() {
            // Finished
            trigger_shake(world, 0.3);
            spawn_particle(world, p, '^', Color::Green, 10);
        } else {
            // Working
            if !is_crit {
                let intensity = world
                    .get::<crate::layer1::resources::ForestryProgress>(entity)
                    .map_or(0.02, |prog| (prog.current / prog.max).mul_add(0.1, 0.02));

                trigger_shake(world, intensity);
                spawn_particle(world, p, '\'', Color::Rgb(139, 69, 19), 3);
            }
        }
    }
    true
}

fn trigger_shake(world: &mut World, intensity: f32) {
    if let Some(mut shake) = world.get_resource_mut::<ScreenShake>() {
        shake.trigger(intensity);
    }
}

fn execute_jury_rig(world: &mut World, designation_entity: Entity) -> bool {
    // Find designation position
    world
        .get::<GridPosition>(designation_entity)
        .copied()
        .is_some_and(|designation_pos| {
            // Find structure at this position
            let structure_entity = world
                .query::<(Entity, &GridPosition, &crate::layer1::structure::Structure)>()
                .iter(world)
                .find(|(_, pos, _)| **pos == designation_pos)
                .map(|(e, _, _)| e);

            if let Some(entity) = structure_entity {
                crate::layer1::structure::process_jury_rig(world, entity);
            }

            // Despawn the designation itself (Jury-Rig is one-shot)
            world.despawn(designation_entity);
            true
        })
}

fn handle_post_work_effects(
    world: &mut World,
    pop_entity: Entity,
    designation_entity: Entity,
    designation_type: DesignationType,
    action_type: ActionType,
    tool_entity_opt: Option<Entity>,
) {
    let skill_type = get_skill_for_designation(designation_type);

    // Add XP
    if let Some(st) = skill_type {
        if let Some(mut skills) = world.get_mut::<Skills>(pop_entity) {
            skills.add_xp(st, 1.0);
        }
    }

    // Fetch structure if designation targets one
    let structure_opt = if world
        .get::<crate::layer1::structure::Structure>(designation_entity)
        .is_some()
    {
        world
            .get::<crate::layer1::structure::Structure>(designation_entity)
            .copied() // Copy to avoid borrow issues
    } else {
        None
    };

    // Fetch skills
    let skills = world.get::<Skills>(pop_entity).cloned().unwrap_or_default();

    handle_workplace_hazards(
        world,
        pop_entity,
        action_type,
        structure_opt.as_ref(),
        &skills,
    );

    // Handle tool durability
    if let Some(tool_entity) = tool_entity_opt {
        handle_tool_durability(world, pop_entity, tool_entity);
    }
}

fn handle_tool_durability(world: &mut World, pop_entity: Entity, tool_entity: Entity) {
    let mut broke = false;
    if let Some(mut tool) = world.get_mut::<Tool>(tool_entity) {
        tool.durability -= TOOL_DURABILITY_LOSS;
        if tool.durability <= 0.0 {
            broke = true;
        }
    }

    if broke {
        // Despawn tool
        world.despawn(tool_entity);

        // Clear equipment
        if let Some(mut eq) = world.get_mut::<Equipment>(pop_entity) {
            eq.tool = None;
        }

        // Log breakage
        if let Some(mut log) = world.get_resource_mut::<MessageLog>() {
            log.add("CRACK! A tool has broken.");
        }
    }
}

fn cleanup_pop_work_state(world: &mut World, pop_entity: Entity) {
    world
        .entity_mut(pop_entity)
        .remove::<MovementTarget>()
        .remove::<AtTarget>();
    if let Some(mut action) = world.get_mut::<PopAction>(pop_entity) {
        action.current = ActionType::Idle;
        action.current_utility = 0.0;
        action.ticks_committed = 1;
    }
}

const fn calculate_knowledge_reward(
    building_type: Option<crate::layer1::building::BuildingType>,
) -> f32 {
    use crate::layer1::building::BuildingType;
    match building_type {
        Some(BuildingType::AncientReactor) => 500.0,
        Some(BuildingType::AncientFabricator) => 300.0,
        _ => 100.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer1::building::{Building, BuildingType};
    use crate::layer1::items::{Item, ToolType};
    use crate::layer1::needs::Needs;
    use crate::layer1::pop::Pop;
    use crate::layer1::resources::{ForestryProgress, MiningProgress};
    use crate::layer1::terrain::TerrainType;
    use crate::layer1::utility_types::{PopAction, UtilityWeights};
    use bevy_ecs::system::RunSystemOnce;

    fn setup_world() -> World {
        crate::setup::init_task_pools();
        let mut world = World::new();
        let tiles = vec![TerrainType::Grass; 100];
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(crate::layer1::erosion::ErosionGrid::new(10, 10));
        world.insert_resource(crate::layer1::resources::ColonyResources::default());
        world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(10, 10));
        world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(10, 10));
        world.insert_resource(crate::shared::time::SimulationTime::default());
        world.insert_resource(crate::layer1::day_night::DayNightCycle::default());
        world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(10, 10));
        world.insert_resource(OccupiedTiles::default());
        world.insert_resource(crate::layer1::taboo::TabooState::default());
        world
    }

    // =========================================================================
    // process_start_plan_system tests
    // =========================================================================

    #[test]
    fn test_process_start_plan_creates_movement_target() {
        let mut world = setup_world();

        // Create a farm as target
        let farm = world
            .spawn((
                Building {
                    building_type: BuildingType::Farm,
                },
                GridPosition { x: 5, y: 5 },
                Farm::default(),
            ))
            .id();

        // Create a pop with StartPlan
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Needs::default(),
                PopAction::default(),
                UtilityWeights::default(),
                StartPlan {
                    action: ActionType::SatisfyHunger,
                    target: Some(farm),
                },
            ))
            .id();

        world.run_system_once(process_start_plan_system).unwrap();

        // Should have MovementTarget
        let mt = world.get::<MovementTarget>(pop);
        assert!(mt.is_some(), "Pop should have MovementTarget");
        let mt = mt.unwrap();
        assert_eq!(mt.target_entity, farm);
        assert_eq!(mt.target_position, GridPosition { x: 5, y: 5 });
        assert_eq!(mt.for_action, ActionType::SatisfyHunger);
    }

    #[test]
    fn test_process_start_plan_removes_itself() {
        let mut world = setup_world();

        let farm = world
            .spawn((
                Building {
                    building_type: BuildingType::Farm,
                },
                GridPosition { x: 5, y: 5 },
                Farm::default(),
            ))
            .id();

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                StartPlan {
                    action: ActionType::SatisfyHunger,
                    target: Some(farm),
                },
            ))
            .id();

        world.run_system_once(process_start_plan_system).unwrap();

        // StartPlan should be removed
        assert!(
            world.get::<StartPlan>(pop).is_none(),
            "StartPlan should be removed"
        );
    }

    #[test]
    fn test_process_start_plan_handles_despawned_target() {
        let mut world = setup_world();

        // Create a pop with StartPlan pointing to a despawned entity
        let fake_entity = Entity::from_raw(9999);
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                StartPlan {
                    action: ActionType::SatisfyHunger,
                    target: Some(fake_entity),
                },
            ))
            .id();

        world.run_system_once(process_start_plan_system).unwrap();

        // Should not have MovementTarget
        assert!(
            world.get::<MovementTarget>(pop).is_none(),
            "Should not create MovementTarget for despawned target"
        );
    }

    // =========================================================================
    // movement_system tests
    // =========================================================================

    #[test]
    fn test_movement_system_moves_toward_target() {
        let mut world = setup_world();

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                MovementTarget {
                    target_entity: Entity::from_raw(1),
                    target_position: GridPosition { x: 5, y: 5 },
                    for_action: ActionType::Work,
                },
            ))
            .id();

        world.run_system_once(movement_system).unwrap();

        let pos = world.get::<GridPosition>(pop).unwrap();
        // Should have moved 1 tile toward target (horizontal first)
        assert_eq!(pos.x, 1);
        assert_eq!(pos.y, 0);
    }

    #[test]
    fn test_movement_system_marks_arrival() {
        let mut world = setup_world();

        // Pop already at target position
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                MovementTarget {
                    target_entity: Entity::from_raw(1),
                    target_position: GridPosition { x: 5, y: 5 },
                    for_action: ActionType::Work,
                },
            ))
            .id();

        world.run_system_once(movement_system).unwrap();

        assert!(
            world.get::<AtTarget>(pop).is_some(),
            "Pop should be marked as AtTarget"
        );
    }

    #[test]
    fn test_movement_system_marks_arrival_on_last_step() {
        let mut world = setup_world();

        // Pop 1 tile away from target
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 4, y: 5 },
                MovementTarget {
                    target_entity: Entity::from_raw(1),
                    target_position: GridPosition { x: 5, y: 5 },
                    for_action: ActionType::Work,
                },
            ))
            .id();

        world.run_system_once(movement_system).unwrap();

        let pos = world.get::<GridPosition>(pop).unwrap();
        assert_eq!(pos.x, 5);
        assert_eq!(pos.y, 5);
        assert!(
            world.get::<AtTarget>(pop).is_some(),
            "Pop should be marked as AtTarget"
        );
    }

    #[test]
    fn test_movement_blocked_by_rock() {
        let mut world = World::new();
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[1] = TerrainType::Rock; // Block position (1, 0)
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(crate::layer1::erosion::ErosionGrid::new(10, 10));

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                MovementTarget {
                    target_entity: Entity::from_raw(1),
                    target_position: GridPosition { x: 5, y: 0 },
                    for_action: ActionType::Work,
                },
            ))
            .id();

        world.run_system_once(movement_system).unwrap();

        // Pop should not have moved (blocked)
        let pos = world.get::<GridPosition>(pop).unwrap();
        assert_eq!(pos.x, 0);
        assert_eq!(pos.y, 0);
    }

    #[test]
    fn test_movement_blocked_by_water() {
        let mut world = World::new();
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[1] = TerrainType::Water; // Block position (1, 0)
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(crate::layer1::erosion::ErosionGrid::new(10, 10));

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                MovementTarget {
                    target_entity: Entity::from_raw(1),
                    target_position: GridPosition { x: 5, y: 0 },
                    for_action: ActionType::Work,
                },
            ))
            .id();

        world.run_system_once(movement_system).unwrap();

        let pos = world.get::<GridPosition>(pop).unwrap();
        assert_eq!(pos.x, 0);
        assert_eq!(pos.y, 0);
    }

    // =========================================================================
    // arrival_handler_system tests
    // =========================================================================

    #[test]
    fn test_arrival_assigns_to_farm() {
        let mut world = setup_world();

        let farm = world
            .spawn((
                Building {
                    building_type: BuildingType::Farm,
                },
                GridPosition { x: 5, y: 5 },
                Farm::default(),
            ))
            .id();

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                Needs::default(),
                MovementTarget {
                    target_entity: farm,
                    target_position: GridPosition { x: 5, y: 5 },
                    for_action: ActionType::SatisfyHunger,
                },
                AtTarget,
            ))
            .id();

        world.run_system_once(arrival_handler_system).unwrap();

        // Pop should be in farm workers list
        let farm_comp = world.get::<Farm>(farm).unwrap();
        assert!(farm_comp.workers.contains(&pop));

        // Pop should have AssignedTo
        let assigned = world.get::<AssignedTo>(pop);
        assert!(assigned.is_some());
        assert_eq!(
            assigned.unwrap().assignment_type,
            AssignmentType::FarmWorker
        );

        // Pop should have Job
        let job = world.get::<Job>(pop);
        assert!(job.is_some(), "FarmWorker assignment should create Job");
        let job = job.unwrap();
        assert_eq!(job.workplace, farm);
        assert_eq!(job.job_type, AssignmentType::FarmWorker);
    }

    #[test]
    fn test_arrival_assigns_to_housing() {
        let mut world = setup_world();

        let housing = world
            .spawn((
                Building {
                    building_type: BuildingType::Housing,
                },
                GridPosition { x: 5, y: 5 },
                Housing::default(),
            ))
            .id();

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                Needs::default(),
                MovementTarget {
                    target_entity: housing,
                    target_position: GridPosition { x: 5, y: 5 },
                    for_action: ActionType::SatisfyRest,
                },
                AtTarget,
            ))
            .id();

        world.run_system_once(arrival_handler_system).unwrap();

        // Pop should be in housing residents list
        let housing_comp = world.get::<Housing>(housing).unwrap();
        assert!(housing_comp.residents.contains(&pop));

        // Pop should have AssignedTo
        let assigned = world.get::<AssignedTo>(pop);
        assert!(assigned.is_some());
        assert_eq!(
            assigned.unwrap().assignment_type,
            AssignmentType::HousingResident
        );

        // Housing is not a job
        assert!(world.get::<Job>(pop).is_none());
    }

    #[test]
    fn test_arrival_farm_at_capacity() {
        let mut world = setup_world();

        let other_pop = world.spawn(Pop).id();
        let farm = world
            .spawn((
                Building {
                    building_type: BuildingType::Farm,
                },
                GridPosition { x: 5, y: 5 },
                Farm {
                    capacity: 1,
                    workers: vec![other_pop], // Already full
                },
            ))
            .id();

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                MovementTarget {
                    target_entity: farm,
                    target_position: GridPosition { x: 5, y: 5 },
                    for_action: ActionType::SatisfyHunger,
                },
                AtTarget,
            ))
            .id();

        world.run_system_once(arrival_handler_system).unwrap();

        // Pop should NOT be in farm workers
        let farm_comp = world.get::<Farm>(farm).unwrap();
        assert!(!farm_comp.workers.contains(&pop));

        // MovementTarget should be removed
        assert!(world.get::<MovementTarget>(pop).is_none());
    }

    // =========================================================================
    // work_execution_system tests
    // =========================================================================

    #[test]
    fn test_work_execution_calls_mine_rock() {
        let mut world = World::new();
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Rock; // (5, 5)
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(crate::layer1::resources::ColonyResources::default());
        world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(10, 10));
        world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(10, 10));
        world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(10, 10));
        world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(10, 10));
        world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(10, 10));
        world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(10, 10));
        world.insert_resource(crate::layer1::day_night::DayNightCycle::default());

        let designation = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Mine,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                MovementTarget {
                    target_entity: designation,
                    target_position: GridPosition { x: 5, y: 5 },
                    for_action: ActionType::Work,
                },
                AtTarget,
            ))
            .id();

        work_execution_system(&mut world);

        // MiningProgress should be added and incremented
        let progress = world.get::<MiningProgress>(designation);
        assert!(progress.is_some(), "MiningProgress should be added");
        assert!(progress.unwrap().current > 0.0, "Progress should increase");

        // Pop should still exist
        assert!(world.get_entity(pop).is_ok());
    }

    #[test]
    fn test_work_execution_calls_chop_tree() {
        let mut world = World::new();
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Tree; // (5, 5)
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(crate::layer1::resources::ColonyResources::default());
        world.insert_resource(crate::layer1::day_night::DayNightCycle::default());

        let designation = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Chop,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        let tool = world
            .spawn((
                Item,
                Tool {
                    tool_type: ToolType::Pickaxe,
                    durability: 100.0,
                    max_durability: 100.0,
                },
            ))
            .id();

        let _pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                Equipment {
                    tool: Some(tool),
                    ..Default::default()
                },
                MovementTarget {
                    target_entity: designation,
                    target_position: GridPosition { x: 5, y: 5 },
                    for_action: ActionType::Work,
                },
                AtTarget,
            ))
            .id();

        work_execution_system(&mut world);

        // ForestryProgress should be added and incremented
        let progress = world.get::<ForestryProgress>(designation);
        assert!(progress.is_some(), "ForestryProgress should be added");
        assert!(progress.unwrap().current > 0.0, "Progress should increase");
    }

    #[test]
    fn test_work_execution_completes_mining() {
        let mut world = World::new();
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Rock;
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(10, 10));
        world.insert_resource(crate::layer1::resources::ColonyResources::default());
        world.insert_resource(crate::layer1::day_night::DayNightCycle::default());

        let designation = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Mine,
                },
                GridPosition { x: 5, y: 5 },
                MiningProgress {
                    current: 95.0,
                    max: 100.0,
                },
            ))
            .id();

        let tool = world
            .spawn((
                Item,
                Tool {
                    tool_type: ToolType::Pickaxe,
                    durability: 100.0,
                    max_durability: 100.0,
                },
            ))
            .id();

        let _pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                Equipment {
                    tool: Some(tool),
                    ..Default::default()
                },
                MovementTarget {
                    target_entity: designation,
                    target_position: GridPosition { x: 5, y: 5 },
                    for_action: ActionType::Work,
                },
                AtTarget,
            ))
            .id();

        work_execution_system(&mut world);

        // Designation should be despawned
        assert!(
            world.get_entity(designation).is_err(),
            "Designation should be despawned"
        );

        // Terrain should be Dirt
        let terrain = world.resource::<TerrainGrid>();
        assert_eq!(terrain.get(5, 5), Some(TerrainType::Dirt));

        // ResourceItem should be spawned
        let items: Vec<_> = world
            .query::<&crate::layer1::resources::ResourceItem>()
            .iter(&world)
            .collect();
        assert!(!items.is_empty(), "ResourceItem should be spawned");
        assert_eq!(
            items[0].resource_type,
            crate::layer1::resources::ResourceType::Stone
        );
    }

    #[test]
    fn test_work_execution_resets_pop_on_completion() {
        let mut world = World::new();
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Rock;
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(10, 10));
        world.insert_resource(crate::layer1::resources::ColonyResources::default());
        world.insert_resource(crate::layer1::day_night::DayNightCycle::default());

        let designation = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Mine,
                },
                GridPosition { x: 5, y: 5 },
                MiningProgress {
                    current: 95.0,
                    max: 100.0,
                },
            ))
            .id();

        let tool = world
            .spawn((
                Item,
                Tool {
                    tool_type: ToolType::Pickaxe,
                    durability: 100.0,
                    max_durability: 100.0,
                },
            ))
            .id();

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                Equipment {
                    tool: Some(tool),
                    ..Default::default()
                },
                MovementTarget {
                    target_entity: designation,
                    target_position: GridPosition { x: 5, y: 5 },
                    for_action: ActionType::Work,
                },
                AtTarget,
                PopAction {
                    current: ActionType::Work,
                    current_utility: 0.8,
                    ticks_committed: 5,
                },
            ))
            .id();

        work_execution_system(&mut world);

        // Designation should be despawned after mining completes
        assert!(
            world.get_entity(designation).is_err(),
            "Designation should be despawned"
        );

        // Pop should have MovementTarget and AtTarget removed
        assert!(
            world.get::<MovementTarget>(pop).is_none(),
            "MovementTarget should be removed after work completes"
        );
        assert!(
            world.get::<AtTarget>(pop).is_none(),
            "AtTarget should be removed after work completes"
        );

        // PopAction should be reset to Idle with zero utility
        let action = world.get::<PopAction>(pop).unwrap();
        assert_eq!(action.current, ActionType::Idle);
        assert!(
            (action.current_utility - 0.0).abs() < f32::EPSILON,
            "Utility should be reset to 0.0"
        );
    }

    #[test]
    fn test_work_execution_cleans_up_stale_target() {
        let mut world = World::new();
        let tiles = vec![TerrainType::Grass; 100];
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(crate::layer1::resources::ColonyResources::default());
        world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(10, 10));
        world.insert_resource(crate::layer1::day_night::DayNightCycle::default());

        // Pop targeting a non-existent designation entity
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                MovementTarget {
                    target_entity: Entity::from_raw(9999),
                    target_position: GridPosition { x: 5, y: 5 },
                    for_action: ActionType::Work,
                },
                AtTarget,
                PopAction {
                    current: ActionType::Work,
                    current_utility: 0.8,
                    ticks_committed: 5,
                },
            ))
            .id();

        work_execution_system(&mut world);

        // Pop should have stale references cleaned up
        assert!(
            world.get::<MovementTarget>(pop).is_none(),
            "MovementTarget should be removed for stale target"
        );
        assert!(
            world.get::<AtTarget>(pop).is_none(),
            "AtTarget should be removed for stale target"
        );

        // PopAction should be reset to Idle
        let action = world.get::<PopAction>(pop).unwrap();
        assert_eq!(action.current, ActionType::Idle);
    }

    // =========================================================================
    // cleanup_previous_assignment_system tests
    // =========================================================================

    #[test]
    fn test_cleanup_removes_from_farm() {
        let mut world = setup_world();

        let pop = world.spawn(Pop).id();
        let farm = world
            .spawn((
                Building {
                    building_type: BuildingType::Farm,
                },
                GridPosition { x: 5, y: 5 },
                Farm {
                    capacity: 2,
                    workers: vec![pop],
                },
            ))
            .id();

        // Pop is assigned and wants to switch
        world.entity_mut(pop).insert((
            AssignedTo {
                entity: farm,
                assignment_type: AssignmentType::FarmWorker,
            },
            StartPlan {
                action: ActionType::SatisfyRest,
                target: None,
            },
        ));

        world
            .run_system_once(cleanup_previous_assignment_system)
            .unwrap();

        // Pop should be removed from farm workers
        let farm_comp = world.get::<Farm>(farm).unwrap();
        assert!(!farm_comp.workers.contains(&pop));

        // AssignedTo should be removed
        assert!(world.get::<AssignedTo>(pop).is_none());
    }

    #[test]
    fn test_cleanup_removes_from_housing() {
        let mut world = setup_world();

        let pop = world.spawn(Pop).id();
        let housing = world
            .spawn((
                Building {
                    building_type: BuildingType::Housing,
                },
                GridPosition { x: 5, y: 5 },
                Housing {
                    capacity: 2,
                    residents: vec![pop],
                },
            ))
            .id();

        world.entity_mut(pop).insert((
            AssignedTo {
                entity: housing,
                assignment_type: AssignmentType::HousingResident,
            },
            StartPlan {
                action: ActionType::SatisfyHunger,
                target: None,
            },
        ));

        world
            .run_system_once(cleanup_previous_assignment_system)
            .unwrap();

        // Pop should be removed from housing residents
        let housing_comp = world.get::<Housing>(housing).unwrap();
        assert!(!housing_comp.residents.contains(&pop));

        // AssignedTo should be removed
        assert!(world.get::<AssignedTo>(pop).is_none());
    }

    #[test]
    fn test_cleanup_handles_despawned_building() {
        let mut world = setup_world();

        let fake_entity = Entity::from_raw(9999);
        let pop = world
            .spawn((
                Pop,
                AssignedTo {
                    entity: fake_entity, // Doesn't exist
                    assignment_type: AssignmentType::FarmWorker,
                },
                StartPlan {
                    action: ActionType::SatisfyRest,
                    target: None,
                },
            ))
            .id();

        // Should not panic
        world
            .run_system_once(cleanup_previous_assignment_system)
            .unwrap();

        // AssignedTo should still be removed
        assert!(world.get::<AssignedTo>(pop).is_none());
    }

    // =========================================================================
    // Integration tests
    // =========================================================================

    #[test]
    fn test_full_execution_flow_farm() {
        let mut world = setup_world();

        let farm = world
            .spawn((
                Building {
                    building_type: BuildingType::Farm,
                },
                GridPosition { x: 2, y: 0 },
                Farm::default(),
            ))
            .id();

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Needs::default(),
                PopAction::default(),
                UtilityWeights::default(),
                StartPlan {
                    action: ActionType::SatisfyHunger,
                    target: Some(farm),
                },
            ))
            .id();

        // Process start plan
        world.run_system_once(process_start_plan_system).unwrap();
        assert!(world.get::<MovementTarget>(pop).is_some());

        // Move toward farm (2 ticks)
        world.run_system_once(movement_system).unwrap();
        let pos = world.get::<GridPosition>(pop).unwrap();
        assert_eq!(pos.x, 1);

        world.run_system_once(movement_system).unwrap();
        assert!(world.get::<AtTarget>(pop).is_some());

        // Handle arrival
        world.run_system_once(arrival_handler_system).unwrap();
        let farm_comp = world.get::<Farm>(farm).unwrap();
        assert!(farm_comp.workers.contains(&pop));
    }

    #[test]
    fn test_pop_moves_multiple_tiles_to_designation() {
        let mut world = setup_world();

        // Create a designation 5 tiles away
        let designation = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Mine,
                },
                GridPosition { x: 5, y: 0 },
            ))
            .id();

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                MovementTarget {
                    target_entity: designation,
                    target_position: GridPosition { x: 5, y: 0 },
                    for_action: ActionType::Work,
                },
            ))
            .id();

        // Move 5 times - pop should reach destination
        for tick in 1..=5 {
            world.run_system_once(movement_system).unwrap();
            let pos = world.get::<GridPosition>(pop).unwrap();
            assert_eq!(pos.x, tick, "Pop should be at x={tick} after {tick} ticks",);
        }

        // Should be at target now
        assert!(world.get::<AtTarget>(pop).is_some());
        let pos = world.get::<GridPosition>(pop).unwrap();
        assert_eq!(pos.x, 5);
        assert_eq!(pos.y, 0);
    }

    #[test]
    fn test_movement_persists_across_evaluation_cycles() {
        use crate::layer1::utility_ai::{
            UtilityConfig, evaluate_actions_system, update_action_timer_system,
        };
        use crate::shared::time::SimulationTime;

        let mut world = setup_world();
        world.insert_resource(UtilityConfig::default());
        world.insert_resource(SimulationTime::default());

        // Put a rock at (5, 0) for mining
        {
            let mut terrain = world.resource_mut::<TerrainGrid>();
            terrain.tiles[5] = TerrainType::Rock;
        }

        // Create a mining designation
        let _designation = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Mine,
                },
                GridPosition { x: 5, y: 0 },
            ))
            .id();

        // Create a pop that will want to work
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Needs::default(),
                PopAction::default(),
                UtilityWeights::default(),
            ))
            .id();

        // Tick 1: Pop should decide to work
        world.run_system_once(update_action_timer_system).unwrap();
        evaluate_actions_system(&mut world);

        // Should have StartPlan for work
        assert!(
            world.get::<StartPlan>(pop).is_some(),
            "Pop should have StartPlan after first evaluation"
        );

        // Process and start moving
        world
            .run_system_once(cleanup_previous_assignment_system)
            .unwrap();
        world.run_system_once(process_start_plan_system).unwrap();
        world.run_system_once(movement_system).unwrap();
        world.run_system_once(arrival_handler_system).unwrap();
        work_execution_system(&mut world);

        let pos1 = world.get::<GridPosition>(pop).unwrap();
        assert_eq!(pos1.x, 1, "Pop should have moved to x=1");

        // Tick 2-4: Continue moving toward rock (stop adjacent at x=4)
        // Pop can't stand ON the rock, so they work from adjacent tile
        for tick in 2..=4 {
            world.run_system_once(update_action_timer_system).unwrap();
            evaluate_actions_system(&mut world);
            world
                .run_system_once(cleanup_previous_assignment_system)
                .unwrap();
            world.run_system_once(process_start_plan_system).unwrap();
            world.run_system_once(movement_system).unwrap();
            world.run_system_once(arrival_handler_system).unwrap();
            work_execution_system(&mut world);

            let pos = *world.get::<GridPosition>(pop).unwrap();
            assert_eq!(pos.x, tick, "Tick {tick}: Pop should be at x={tick}");
        }

        // After tick 4, pop should be adjacent to rock (at x=4) and marked AtTarget
        assert!(
            world.get::<AtTarget>(pop).is_some(),
            "Pop should be AtTarget when adjacent to work designation"
        );
        let final_pos = *world.get::<GridPosition>(pop).unwrap();
        assert_eq!(final_pos.x, 4, "Pop should stop adjacent to rock at x=4");

        // Pop should now be at the designation
        assert!(world.get::<AtTarget>(pop).is_some());
    }

    #[test]
    fn test_work_execution_efficiency_low_morale() {
        let mut world = World::new();
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Rock;
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(crate::layer1::resources::ColonyResources::default());
        world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(10, 10));
        world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(10, 10));
        world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(10, 10));
        world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(10, 10));
        world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(10, 10));
        world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(10, 10));
        world.insert_resource(crate::layer1::day_night::DayNightCycle::default());

        let designation = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Mine,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        let tool = world
            .spawn((
                Item,
                Tool {
                    tool_type: ToolType::Pickaxe,
                    durability: 100.0,
                    max_durability: 100.0,
                },
            ))
            .id();

        // Spawn a pop with low morale (hunger=0.1, rest=0.1, leisure=0.1 -> morale=0.1)
        // Expected efficiency: 0.5 (penalty)
        let _pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                Needs {
                    hunger: 0.1,
                    rest: 0.1,
                    leisure: 0.1,
                },
                Equipment {
                    tool: Some(tool),
                    ..Default::default()
                },
                MovementTarget {
                    target_entity: designation,
                    target_position: GridPosition { x: 5, y: 5 },
                    for_action: ActionType::Work,
                },
                AtTarget,
            ))
            .id();

        work_execution_system(&mut world);

        let progress = world.get::<MiningProgress>(designation).unwrap();
        // Base work = 10.0
        // Tool efficiency = 1.0 (default resources has 2 tools)
        // Morale efficiency = 0.5 (low morale)
        // Expected = 10.0 * 1.0 * 0.5 = 5.0
        // Ludwig: Organic factor (0.9-1.1) implies range 4.5 - 5.5
        // Note: 5% Critical hit chance multiplies by 5.0 -> ~25.0
        let is_crit_range = progress.current >= 22.5 && progress.current <= 27.5;
        let is_normal_range = progress.current >= 4.5 && progress.current <= 5.5;

        assert!(
            is_normal_range || is_crit_range,
            "Expected ~5.0 (or ~25.0 crit) progress, got {}",
            progress.current
        );
    }

    #[test]
    fn test_work_execution_efficiency_high_morale() {
        let mut world = World::new();
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Rock;
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(crate::layer1::resources::ColonyResources::default());
        world.insert_resource(crate::layer1::day_night::DayNightCycle::default());

        let designation = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Mine,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        let tool = world
            .spawn((
                Item,
                Tool {
                    tool_type: ToolType::Pickaxe,
                    durability: 100.0,
                    max_durability: 100.0,
                },
            ))
            .id();

        // Spawn a pop with high morale (all 1.0 -> morale=1.0)
        // Expected efficiency: 1.2 (bonus)
        let _pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                Needs {
                    hunger: 1.0,
                    rest: 1.0,
                    leisure: 1.0,
                },
                Equipment {
                    tool: Some(tool),
                    ..Default::default()
                },
                MovementTarget {
                    target_entity: designation,
                    target_position: GridPosition { x: 5, y: 5 },
                    for_action: ActionType::Work,
                },
                AtTarget,
            ))
            .id();

        work_execution_system(&mut world);

        let progress = world.get::<MiningProgress>(designation).unwrap();
        // Base work = 10.0
        // Tool efficiency = 1.0 (default resources has 2 tools)
        // Morale efficiency = 1.2 (high morale)
        // Expected = 10.0 * 1.0 * 1.2 = 12.0
        // Ludwig: Organic factor (0.9-1.1) implies range 10.8 - 13.2
        assert!(
            progress.current >= 10.8 && progress.current <= 13.2,
            "Expected ~12.0 progress, got {}",
            progress.current
        );
    }

    #[test]
    fn test_work_execution_skills_mining_efficiency() {
        let mut world = World::new();
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Rock;
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(crate::layer1::resources::ColonyResources::default());
        world.insert_resource(crate::layer1::day_night::DayNightCycle::default());

        let designation = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Mine,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        // Pop with Mining skill level 1 (100 XP) -> Efficiency 1.1
        let mut skills = Skills::default();
        skills.add_xp(SkillType::Mining, 100.0);

        let tool = world
            .spawn((
                Item,
                Tool {
                    tool_type: ToolType::Pickaxe,
                    durability: 100.0,
                    max_durability: 100.0,
                },
            ))
            .id();

        let _pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                skills,
                Equipment {
                    tool: Some(tool),
                    ..Default::default()
                },
                MovementTarget {
                    target_entity: designation,
                    target_position: GridPosition { x: 5, y: 5 },
                    for_action: ActionType::Work,
                },
                AtTarget,
            ))
            .id();

        work_execution_system(&mut world);

        let progress = world.get::<MiningProgress>(designation).unwrap();
        // Base work = 10.0
        // Tool efficiency = 1.0
        // Morale efficiency = 1.0 (0.5 morale is neutral)
        // Skill efficiency = 1.1
        // Expected = 10.0 * 1.0 * 1.0 * 1.1 = 11.0
        // Ludwig: Organic factor (0.9-1.1) implies range 9.9 - 12.1
        // Crit (5%) -> Range 49.5 - 60.5
        let is_normal = progress.current >= 9.9 && progress.current <= 12.1;
        let is_crit = progress.current >= 49.5 && progress.current <= 60.5;
        assert!(
            is_normal || is_crit,
            "Expected ~11.0 (or ~55.0 crit) progress, got {}",
            progress.current
        );
    }

    #[test]
    fn test_work_execution_gains_xp() {
        let mut world = World::new();
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Rock;
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(crate::layer1::resources::ColonyResources::default());
        world.insert_resource(crate::layer1::day_night::DayNightCycle::default());

        let designation = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Mine,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                Skills::default(),
                MovementTarget {
                    target_entity: designation,
                    target_position: GridPosition { x: 5, y: 5 },
                    for_action: ActionType::Work,
                },
                AtTarget,
            ))
            .id();

        work_execution_system(&mut world);

        let skills = world.get::<Skills>(pop).unwrap();
        assert!((skills.get_xp(SkillType::Mining) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_movement_with_speed_penalty() {
        let mut world = setup_world();

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                MovementTarget {
                    target_entity: Entity::from_raw(1),
                    target_position: GridPosition { x: 5, y: 0 },
                    for_action: ActionType::Work,
                },
                Speed {
                    base: 1.0,
                    current: 0.5, // Move every 2 ticks
                    accumulator: 0.0,
                },
            ))
            .id();

        // Tick 1: Accumulator 0.0 + 0.5 = 0.5 (< 1.0) -> No Move
        world.run_system_once(movement_system).unwrap();
        let pos = world.get::<GridPosition>(pop).unwrap();
        assert_eq!(pos.x, 0);

        // Tick 2: Accumulator 0.5 + 0.5 = 1.0 (>= 1.0) -> Move
        world.run_system_once(movement_system).unwrap();
        let pos = world.get::<GridPosition>(pop).unwrap();
        assert_eq!(pos.x, 1);
    }

    #[test]
    fn test_combat_execution_system_attacks_in_range() {
        use crate::layer1::combat::{AttackProperties, Weapon};
        use crate::layer1::health::Health;

        let mut world = setup_world();

        // Create Enemy
        let enemy = world
            .spawn((
                GridPosition { x: 1, y: 0 },
                Health {
                    current: 100.0,
                    max: 100.0,
                },
            ))
            .id();

        // Create Weapon
        let weapon = world
            .spawn(Weapon {
                properties: AttackProperties {
                    damage: 10.0,
                    range: 1.0,
                    cooldown: 0,
                    accuracy: 1.0,
                },
            })
            .id();

        // Create Pop targeting enemy
        world.spawn((
            Pop,
            GridPosition { x: 0, y: 0 }, // Adjacent (dist 1)
            Equipment {
                weapon: Some(weapon),
                ..Default::default()
            },
            MovementTarget {
                target_entity: enemy,
                target_position: GridPosition { x: 1, y: 0 },
                for_action: ActionType::Fight,
            },
        ));

        combat_execution_system(&mut world);

        // Enemy should take damage
        let health = world.get::<Health>(enemy).unwrap();
        assert!((health.current - 90.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_combat_execution_system_chases_out_of_range() {
        use crate::layer1::combat::{AttackProperties, Weapon};
        use crate::layer1::health::Health;

        let mut world = setup_world();

        // Create Enemy far away
        let enemy = world
            .spawn((
                GridPosition { x: 5, y: 0 },
                Health {
                    current: 100.0,
                    max: 100.0,
                },
            ))
            .id();

        let weapon = world
            .spawn(Weapon {
                properties: AttackProperties {
                    damage: 10.0,
                    range: 1.0, // Short range
                    cooldown: 0,
                    accuracy: 1.0,
                },
            })
            .id();

        // Create Pop targeting enemy
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                Equipment {
                    weapon: Some(weapon),
                    ..Default::default()
                },
                MovementTarget {
                    target_entity: enemy,
                    target_position: GridPosition { x: 5, y: 0 },
                    for_action: ActionType::Fight,
                },
                AtTarget, // Simulate arrived at previous target position?
                          // Or simply ensure AtTarget is removed if present
            ))
            .id();

        combat_execution_system(&mut world);

        // Enemy should NOT take damage
        let health = world.get::<Health>(enemy).unwrap();
        assert!((health.current - 100.0).abs() < f32::EPSILON);

        // AtTarget should be removed (to allow movement)
        assert!(world.get::<AtTarget>(pop).is_none());
    }

    #[test]
    fn test_arrival_assigns_to_hospital() {
        let mut world = setup_world();

        // Hospital component might need to be imported or fully qualified
        // It is fully qualified in the test body I prepared.
        let hospital = world
            .spawn((
                Building {
                    building_type: BuildingType::Hospital,
                },
                GridPosition { x: 5, y: 5 },
                crate::layer1::medical::Hospital::default(),
            ))
            .id();

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                Needs::default(),
                MovementTarget {
                    target_entity: hospital,
                    target_position: GridPosition { x: 5, y: 5 },
                    for_action: ActionType::SeekMedicalCare,
                },
                AtTarget,
            ))
            .id();

        world.run_system_once(arrival_handler_system).unwrap();

        let assigned = world.get::<AssignedTo>(pop);
        assert!(assigned.is_some(), "Pop should be assigned to hospital");
        assert_eq!(assigned.unwrap().assignment_type, AssignmentType::Patient);

        // Hospital is not a job
        assert!(world.get::<Job>(pop).is_none());

        // MovementTarget should be removed
        assert!(world.get::<MovementTarget>(pop).is_none());
    }

    #[test]
    fn test_arrival_assigns_to_library() {
        let mut world = setup_world();

        let library = world
            .spawn((
                Building {
                    building_type: BuildingType::Library,
                },
                GridPosition { x: 5, y: 5 },
                crate::layer1::tech::Library,
            ))
            .id();

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 5, y: 5 },
                Needs::default(),
                MovementTarget {
                    target_entity: library,
                    target_position: GridPosition { x: 5, y: 5 },
                    for_action: ActionType::Research,
                },
                AtTarget,
            ))
            .id();

        world.run_system_once(arrival_handler_system).unwrap();

        let assigned = world.get::<AssignedTo>(pop);
        assert!(assigned.is_some(), "Pop should be assigned to library");
        assert_eq!(
            assigned.unwrap().assignment_type,
            AssignmentType::LibraryWorker
        );

        // Library is a job
        let job = world.get::<Job>(pop);
        assert!(job.is_some(), "LibraryWorker assignment should create Job");
        let job = job.unwrap();
        assert_eq!(job.workplace, library);
        assert_eq!(job.job_type, AssignmentType::LibraryWorker);

        // MovementTarget should be removed
        assert!(world.get::<MovementTarget>(pop).is_none());
    }

    #[test]
    fn test_movement_system_fast_walker() {
        use crate::layer1::traits::{Trait, Traits};
        use std::collections::HashSet;

        let mut world = setup_world();

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                MovementTarget {
                    target_entity: Entity::from_raw(1),
                    target_position: GridPosition { x: 5, y: 0 },
                    for_action: ActionType::Work,
                },
                Speed {
                    base: 1.0,
                    current: 1.0,
                    accumulator: 0.0,
                },
                Traits(HashSet::from([Trait::FastWalker])), // +10% speed
            ))
            .id();

        // Tick 1: Acc = 0.0 + (1.0 * 1.1) = 1.1 -> Move -> Acc = 0.1
        world.run_system_once(movement_system).unwrap();
        let pos = world.get::<GridPosition>(pop).unwrap();
        assert_eq!(pos.x, 1);
        let speed = world.get::<Speed>(pop).unwrap();
        assert!((speed.accumulator - 0.1).abs() < 0.001);
    }

    #[test]
    fn test_work_execution_hard_worker() {
        use crate::layer1::traits::{Trait, Traits};
        use std::collections::HashSet;

        let mut world = World::new();
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Rock;
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(crate::layer1::resources::ColonyResources::default());
        world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(10, 10));
        world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(10, 10));
        world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(10, 10));
        world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(10, 10));
        world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(10, 10));
        world.insert_resource(crate::layer1::structural_integrity::RoofGrid::new(10, 10));
        world.insert_resource(crate::layer1::day_night::DayNightCycle::default());

        let designation = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Mine,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        let tool = world
            .spawn((
                Item,
                Tool {
                    tool_type: ToolType::Pickaxe,
                    durability: 100.0,
                    max_durability: 100.0,
                },
            ))
            .id();

        world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            Equipment {
                tool: Some(tool),
                ..Default::default()
            },
            MovementTarget {
                target_entity: designation,
                target_position: GridPosition { x: 5, y: 5 },
                for_action: ActionType::Work,
            },
            AtTarget,
            Traits(HashSet::from([Trait::HardWorker])), // +20% Work Speed
        ));

        work_execution_system(&mut world);

        let progress = world.get::<MiningProgress>(designation).unwrap();
        // Base 10.0 * 1.2 = 12.0.
        // Organic factor 0.9-1.1 -> Range 10.8 - 13.2
        // Crit (5%) -> Range 54.0 - 66.0
        let is_normal = progress.current >= 10.8 && progress.current <= 13.2;
        let is_crit = progress.current >= 54.0 && progress.current <= 66.0;
        assert!(is_normal || is_crit, "Got {}", progress.current);
    }

    #[test]
    fn test_coyote_speed_movement() {
        let mut world = setup_world();

        // 0.96 accumulator, 1.0 cost.
        // Without coyote: 0.96 < 1.0 -> No move.
        // With coyote (0.05): 0.96 >= 0.95 -> Move.
        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                MovementTarget {
                    target_entity: Entity::from_raw(1),
                    target_position: GridPosition { x: 5, y: 0 },
                    for_action: ActionType::Work,
                },
                Speed {
                    base: 1.0,
                    current: 0.0, // Don't add more speed this tick to isolate accumulator check
                    accumulator: 0.96,
                },
            ))
            .id();

        world.run_system_once(movement_system).unwrap();
        let pos = world.get::<GridPosition>(pop).unwrap();
        assert_eq!(pos.x, 1, "Should move due to Coyote Speed");

        let speed = world.get::<Speed>(pop).unwrap();
        // 0.96 - 1.0 = -0.04
        assert!((speed.accumulator - (-0.04)).abs() < 0.001);
    }
    #[test]
    fn test_movement_system_stuck_in_greedy_corner() {
        let mut world = setup_world();

        // Map setup
        // P # .
        // . . T
        // Pop at (0,0). Target at (2,1). Wall at (1,0).

        {
            let mut terrain = world.resource_mut::<TerrainGrid>();
            terrain.tiles[1] = TerrainType::Rock; // (1,0)
        }

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                MovementTarget {
                    target_entity: Entity::from_raw(1),
                    target_position: GridPosition { x: 2, y: 1 },
                    for_action: ActionType::Work,
                },
            ))
            .id();

        // Run movement
        world.run_system_once(movement_system).unwrap();

        // Expectation: Pop moves to (0,1) because (1,0) is blocked
        let pos = world.get::<GridPosition>(pop).unwrap();
        assert_eq!(pos.x, 0);
        assert_eq!(pos.y, 1, "Pop should detour to Y if X is blocked");
    }

    #[test]
    fn test_movement_system_blocked_by_hit_stop() {
        let mut world = setup_world();

        let pop = world
            .spawn((
                Pop,
                GridPosition { x: 0, y: 0 },
                MovementTarget {
                    target_entity: Entity::from_raw(1),
                    target_position: GridPosition { x: 5, y: 0 },
                    for_action: ActionType::Work,
                },
                HitStop { ticks_remaining: 1 },
            ))
            .id();

        // Run movement
        world.run_system_once(movement_system).unwrap();

        // Should NOT move
        let pos = world.get::<GridPosition>(pop).unwrap();
        assert_eq!(pos.x, 0, "Pop should be frozen by HitStop");

        // Decrement HitStop (manually or via system)
        world.get_mut::<HitStop>(pop).unwrap().ticks_remaining = 0;

        // Run movement again
        world.run_system_once(movement_system).unwrap();

        // Should move now (if HitStop is 0, we still check ticks_remaining)
        // Wait, if ticks_remaining is 0, we treat it as no hit stop?
        // My implementation: if hit_stop.ticks_remaining > 0 { continue }
        // So 0 is fine.
        let pos = world.get::<GridPosition>(pop).unwrap();
        assert_eq!(pos.x, 1, "Pop should move when HitStop expires");
    }

    #[test]
    fn test_combat_execution_blocked_by_hit_stop() {
        use crate::layer1::combat::{AttackProperties, Weapon};
        use crate::layer1::health::Health;

        let mut world = setup_world();

        let enemy = world
            .spawn((
                GridPosition { x: 1, y: 0 },
                Health {
                    current: 100.0,
                    max: 100.0,
                },
            ))
            .id();

        let weapon = world
            .spawn(Weapon {
                properties: AttackProperties {
                    damage: 10.0,
                    range: 1.0,
                    cooldown: 0,
                    accuracy: 1.0,
                },
            })
            .id();

        world.spawn((
            Pop,
            GridPosition { x: 0, y: 0 },
            Equipment {
                weapon: Some(weapon),
                ..Default::default()
            },
            MovementTarget {
                target_entity: enemy,
                target_position: GridPosition { x: 1, y: 0 },
                for_action: ActionType::Fight,
            },
            HitStop { ticks_remaining: 1 },
        ));

        combat_execution_system(&mut world);

        // Enemy should NOT take damage
        let health = world.get::<Health>(enemy).unwrap();
        assert!(
            (health.current - 100.0).abs() < f32::EPSILON,
            "HitStop should prevent attack"
        );
    }

    #[test]
    fn test_work_execution_augmentation_bonus() {
        use crate::layer1::cybernetics::{Augmentations, Prosthetic, ProstheticType};

        let mut world = setup_world();
        let mut tiles = vec![TerrainType::Grass; 100];
        tiles[55] = TerrainType::Rock;
        world.insert_resource(TerrainGrid {
            width: 10,
            height: 10,
            tiles,
        });
        world.insert_resource(crate::layer1::resources::ColonyResources::default());
        world.insert_resource(crate::layer1::day_night::DayNightCycle::default());

        let designation = world
            .spawn((
                Designation {
                    designation_type: DesignationType::Mine,
                },
                GridPosition { x: 5, y: 5 },
            ))
            .id();

        let tool = world
            .spawn((
                Item,
                Tool {
                    tool_type: ToolType::Pickaxe,
                    durability: 100.0,
                    max_durability: 100.0,
                },
            ))
            .id();

        // Prosthetic with +50% efficiency
        let prosthetic = world
            .spawn(Prosthetic {
                prosthetic_type: ProstheticType::BionicArm,
                efficiency_bonus: 0.5,
                social_penalty: 0.0,
                power_consumption: 0.0,
            })
            .id();

        world.spawn((
            Pop,
            GridPosition { x: 5, y: 5 },
            Equipment {
                tool: Some(tool),
                ..Default::default()
            },
            MovementTarget {
                target_entity: designation,
                target_position: GridPosition { x: 5, y: 5 },
                for_action: ActionType::Work,
            },
            AtTarget,
            Augmentations {
                installed: vec![prosthetic],
            },
        ));

        work_execution_system(&mut world);

        let progress = world.get::<MiningProgress>(designation).unwrap();
        // Base 10.0. Tool 1.0. Morale 1.0 (neutral 0.5).
        // Augmentation +0.5 -> Multiplier 1.5.
        // Expected: 10.0 * 1.5 = 15.0.
        // Organic: 13.5 - 16.5.
        // Crit (5%): ~75.0.

        let is_normal = progress.current >= 13.5 && progress.current <= 16.5;
        let is_crit = progress.current >= 67.5 && progress.current <= 82.5;

        assert!(
            is_normal || is_crit,
            "Expected ~15.0 (or crit), got {}",
            progress.current
        );
    }
}
