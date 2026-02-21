#![allow(clippy::too_many_arguments, clippy::type_complexity, clippy::cast_precision_loss)]

use bevy_ecs::prelude::*;
use rand::Rng;
use ratatui::style::Color;

use crate::layer1::map::{GridPosition, ScreenShake};
use crate::layer1::needs::{Needs, get_morale_efficiency};
use crate::layer1::memory::{Memories, calculate_effective_morale};
use crate::layer1::social::SocialBuff;
use crate::layer1::items::{Equipment, Tool};
use crate::layer1::traits::{Traits, get_trait_work_speed_modifier};
use crate::layer1::morale::Morale;
use crate::layer1::factions::{Factions, FactionMember, FactionState, FactionId};
use crate::layer1::edicts::{ColonyPolicies, get_work_speed_modifier};
use crate::layer1::day_night::DayNightCycle;
use crate::layer1::utility_types::ActionType;
use crate::layer1::designation::{Designation, DesignationType};
use crate::layer1::structure::Structure;
use crate::layer1::heirloom::{Heirloom, RetrogradeEngineeringEvent, ToolHistory};
use crate::layer1::skills::{Skills, SkillType, get_skill_efficiency};
use crate::layer1::hazards::handle_workplace_hazards;
use crate::layer1::cybernetics::get_efficiency_bonus;
use crate::layer1::admin::AdminStats;
use crate::layer1::resources::{ColonyResources, process_mining, process_logging, ResourceItem, ResourceType};
use crate::layer1::particles::spawn_particle;
use crate::layer1::ruins::Ruin;
use crate::layer1::building::{Building, BuildingType, OccupiedTiles};
use crate::layer1::defense::Gate;
use crate::layer1::access_control::AccessControl;
use crate::shared::log::MessageLog;
use crate::layer1::flora::process_flora_clearing;

use super::types::{MovementTarget, AtTarget};
use super::utils::cleanup_pop_work_state;

/// Work amount applied per tick when a pop is working.
pub const WORK_PER_TICK: f32 = 10.0;

/// Durability loss per tick when working.
pub const TOOL_DURABILITY_LOSS: f32 = 0.1;

/// Efficiency multiplier when working without tools.
pub const NO_TOOL_PENALTY: f32 = 0.5;

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

/// Executes work at designations when pop is at target with Work action.
pub fn work_execution_system(world: &mut World) {
    let policies = world.get_resource::<ColonyPolicies>().cloned();
    let work_speed_mod = policies.as_ref().map_or(1.0, get_work_speed_modifier);

    // Fetch DayNightCycle
    let cycle = world.get_resource::<DayNightCycle>().map(|c| c.time_of_day);

    // Fetch Factions for strike check
    // We collect striking factions into a set to avoid borrowing conflicts with world
    let striking_factions: std::collections::HashSet<FactionId> = world
        .get_resource::<Factions>()
        .map(|f| {
            f.map
                .iter()
                .filter(|(_, d)| d.state == FactionState::Striking)
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
            Option<&FactionMember>,
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

pub fn process_single_worker(
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
        .get::<Structure>(designation_entity)
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
            .get::<Structure>(designation_entity)
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

pub fn calculate_work_amount(
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

    let admin_efficiency = world
        .get_resource::<AdminStats>()
        .map_or(1.0, |stats| stats.efficiency);

    let amount = WORK_PER_TICK
        * tool_efficiency
        * morale_efficiency
        * skill_efficiency
        * work_speed_mod
        * admin_efficiency
        * (1.0 + augmentation_bonus)
        * organic_factor;

    // Cap work amount to prevent logic bugs / economy exploits
    amount.min(1000.0)
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

const fn calculate_knowledge_reward(
    building_type: Option<BuildingType>,
) -> f32 {
    match building_type {
        Some(BuildingType::AncientReactor) => 500.0,
        Some(BuildingType::AncientFabricator) => 300.0,
        _ => 100.0,
    }
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
        .query::<(Entity, &GridPosition, &Building)>()
        .iter(world)
        .find(|(_, pos, b)| {
            pos.x == designation_pos.x
                && pos.y == designation_pos.y
                && b.building_type == BuildingType::Lander
        })
        .map(|(e, _, _)| e);

    if let Some(entity) = building_entity {
        // Spawn Resources
        spawn_resource_pile(
            world,
            designation_pos,
            ResourceType::Metal,
            100.0,
        );
        spawn_resource_pile(
            world,
            designation_pos,
            ResourceType::Fuel,
            50.0,
        );
        spawn_resource_pile(
            world,
            designation_pos,
            ResourceType::Rations,
            50.0,
        );

        if let Some(mut log) = world.get_resource_mut::<MessageLog>() {
            log.add_colored(
                "Lander cannibalized! Massive resources gained.",
                Color::Yellow,
            );
        }

        // VFX
        spawn_particle(
            world,
            designation_pos,
            'X',
            Color::Red,
            20,
        );
        if let Some(mut shake) = world.get_resource_mut::<ScreenShake>() {
            shake.trigger(0.8);
        }

        // Cleanup
        world.despawn(entity);
        if let Some(mut occupied) =
            world.get_resource_mut::<OccupiedTiles>()
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

pub fn spawn_resource_pile(
    world: &mut World,
    pos: GridPosition,
    res_type: ResourceType,
    amount: f32,
) {
    world.spawn((
        ResourceItem {
            resource_type: res_type,
            amount,
        },
        pos,
    ));
}

pub fn execute_work_on_designation(
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

pub fn handle_mining_work(
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

pub fn handle_chopping_work(
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

pub fn trigger_shake(world: &mut World, intensity: f32) {
    if let Some(mut shake) = world.get_resource_mut::<ScreenShake>() {
        shake.trigger(intensity);
    }
}

pub fn execute_jury_rig(world: &mut World, designation_entity: Entity) -> bool {
    // Find designation position
    world
        .get::<GridPosition>(designation_entity)
        .copied()
        .is_some_and(|designation_pos| {
            // Find structure at this position
            let structure_entity = world
                .query::<(Entity, &GridPosition, &Structure)>()
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

pub fn handle_post_work_effects(
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
        .get::<Structure>(designation_entity)
        .is_some()
    {
        world
            .get::<Structure>(designation_entity)
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

pub fn handle_tool_durability(world: &mut World, pop_entity: Entity, tool_entity: Entity) {
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
