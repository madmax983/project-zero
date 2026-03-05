use bevy_ecs::prelude::*;
use rand::Rng;

use crate::layer1::admin::AdminStats;
use crate::layer1::cybernetics::get_efficiency_bonus;
use crate::layer1::day_night::DayNightCycle;
use crate::layer1::designation::{Designation, DesignationType};
use crate::layer1::economy::{get_wage_for_job, pay_wage};
use crate::layer1::edicts::{get_work_speed_modifier, ColonyPolicies};
use crate::layer1::eureka::check_for_eureka_world;
use crate::layer1::execution::components::{AtTarget, MovementTarget};
use crate::layer1::execution::demolish::{
    execute_cannibalize, execute_demolish, execute_destroy, execute_jury_rig,
};
use crate::layer1::execution::mining::{handle_chopping_work, handle_mining_work};
use crate::layer1::flora::process_flora_clearing;
use crate::layer1::gastronomy::WorkSpeedBuff;
use crate::layer1::hazards::handle_workplace_hazards;
use crate::layer1::heirloom::{Heirloom, ToolHistory};
use crate::layer1::items::{Equipment, Tool, UnequipEvent};
use crate::layer1::language::{calculate_coordination_penalty, Dialect, Linguistics};
use crate::layer1::map::GridPosition;
use crate::layer1::memory::{calculate_effective_morale, Memories};
use crate::layer1::morale::Morale;
use crate::layer1::mother_lode::MotherLode;
use crate::layer1::needs::{get_morale_efficiency, Needs};
use crate::layer1::pop::Job;
use crate::layer1::resources::{ColonyResources, ResourceType};
use crate::layer1::skills::{get_skill_efficiency, SkillType, Skills};
use crate::layer1::social::SocialBuff;
use crate::layer1::tech::hypno_learning::MentalFog;
use crate::layer1::tech::Tech;
use crate::layer1::traits::{get_trait_work_speed_modifier, get_job_efficiency_modifier, Traits};
use crate::layer1::utility_types::{ActionType, PopAction};
use crate::shared::log::MessageLog;

/// Work amount applied per tick when a pop is working.
const WORK_PER_TICK: f32 = 10.0;

/// Durability loss per tick when working.
const TOOL_DURABILITY_LOSS: f32 = 0.1;

struct WorkerData {
    entity: Entity,
    morale: f32,
    action: ActionType,
    equipment: Option<Equipment>,
    speed_modifier: f32,
    job: Option<Job>,
    dialect: Dialect,
    linguistics: Linguistics,
}

/// Executes work at designations when pop is at target with Work action.
#[allow(clippy::too_many_lines, clippy::items_after_statements)]
pub fn work_execution_system(world: &mut World) {
    let policies = world.get_resource::<ColonyPolicies>().cloned();
    let global_work_speed_mod = policies.as_ref().map_or(1.0, get_work_speed_modifier);

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

    // Spec 218: Calculate Improvised Efficiency (Global Fallback)
    let (improvised_efficiency, consumed_resource_type) = world
        .get_resource::<crate::layer1::resources::ColonyResources>()
        .map_or((0.5, None), |res| {
            crate::layer1::execution::efficiency::calculate_work_efficiency(res)
        });

    let workers_by_target =
        collect_workers_by_target(world, policies.as_ref(), &striking_factions, cycle);

    for (target_entity, group) in workers_by_target {
        for worker in &group {
            let coordination_mod = calculate_group_coordination(worker, &group);

            process_single_worker(
                world,
                worker.entity,
                target_entity,
                worker.morale,
                worker.action,
                worker.equipment,
                global_work_speed_mod * worker.speed_modifier * coordination_mod,
                worker.job,
                improvised_efficiency,
                consumed_resource_type,
            );
        }
    }
}

fn collect_workers_by_target(
    world: &mut World,
    policies: Option<&ColonyPolicies>,
    striking_factions: &std::collections::HashSet<crate::layer1::factions::FactionId>,
    cycle: Option<crate::layer1::day_night::TimeOfDay>,
) -> std::collections::HashMap<Entity, Vec<WorkerData>> {
    let mut workers_by_target: std::collections::HashMap<Entity, Vec<WorkerData>> =
        std::collections::HashMap::new();

    let query_results: Vec<_> = world
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
            Option<&WorkSpeedBuff>,
            Option<&Job>,
            Option<&Dialect>,
            Option<&Linguistics>,
            Option<&MentalFog>,
        ), With<AtTarget>>()
        .iter(world)
        .filter(|(_, mt, _, _, _, _, _, _, faction_member, _, _, _, _, _)| {
            let is_work = mt.for_action == ActionType::Work || mt.for_action == ActionType::Repair;
            if !is_work {
                return false;
            }

            let is_striking = faction_member
                .and_then(|m| m.faction_id)
                .is_some_and(|fid| striking_factions.contains(&fid));

            !is_striking
        })
        .map(
            |(
                e,
                mt,
                needs,
                memories,
                social_buff,
                eq,
                traits,
                morale_comp,
                _,
                buff,
                job,
                dialect,
                ling,
                fog,
            )| {
                let morale = needs.map_or(0.5, |n| {
                    calculate_effective_morale(
                        n,
                        memories,
                        social_buff,
                        policies,
                        traits,
                        cycle,
                        morale_comp,
                    )
                });
                let trait_work_mod = traits.map_or(1.0, get_trait_work_speed_modifier);
                let spec_mod = if let (Some(j), Some(t)) = (job, traits) {
                    get_job_efficiency_modifier(t, j.job_type)
                } else {
                    1.0
                };
                let buff_mod = buff.map_or(1.0, |b| b.multiplier);
                let fog_mod = fog.map_or(1.0, |f| f.work_speed_penalty);

                (
                    mt.target_entity,
                    WorkerData {
                        entity: e,
                        morale,
                        action: mt.for_action,
                        equipment: eq.copied(),
                        speed_modifier: trait_work_mod * spec_mod * buff_mod * fog_mod,
                        job: job.copied(),
                        dialect: dialect.copied().unwrap_or_default(),
                        linguistics: ling.cloned().unwrap_or_default(),
                    },
                )
            },
        )
        .collect();

    for (target, worker) in query_results {
        workers_by_target.entry(target).or_default().push(worker);
    }

    workers_by_target
}

fn calculate_group_coordination(worker: &WorkerData, group: &[WorkerData]) -> f32 {
    let mut coordination_mod = 1.0;
    for other in group {
        if worker.entity == other.entity {
            continue;
        }
        let p =
            calculate_coordination_penalty(&worker.dialect, &worker.linguistics, &other.dialect);
        if p < coordination_mod {
            coordination_mod = p;
        }
    }
    coordination_mod
}

fn get_designation_type(
    world: &World,
    entity: Entity,
    action: ActionType,
) -> Option<DesignationType> {
    if let Some(des) = world.get::<Designation>(entity) {
        return Some(des.designation_type);
    }
    if action == ActionType::Repair
        && world
            .get::<crate::layer1::structure::Structure>(entity)
            .is_some()
    {
        return Some(DesignationType::Repair);
    }
    None
}

#[allow(clippy::too_many_arguments)]
fn process_single_worker(
    world: &mut World,
    pop_entity: Entity,
    designation_entity: Entity,
    morale: f32,
    action_type: ActionType,
    equipment_opt: Option<Equipment>,
    work_speed_mod: f32,
    job_opt: Option<Job>,
    improvised_efficiency: f32,
    consumed_resource_type: Option<ResourceType>,
) {
    // Check if designation/target still exists (early exit)
    if world.get_entity(designation_entity).is_err() {
        cleanup_pop_work_state(world, pop_entity);
        return;
    }

    // Get designation type
    let Some(designation_type) = get_designation_type(world, designation_entity, action_type)
    else {
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
        improvised_efficiency,
    );

    // Execute Work
    let worked = execute_work_on_designation(
        world,
        pop_entity,
        designation_entity,
        designation_type,
        work_amount,
    );

    if check_work_completion(world, designation_entity, designation_type) {
        cleanup_pop_work_state(world, pop_entity);

        // Pay Wage
        let wage = job_opt.map_or(1.0, |job| get_wage_for_job(job.job_type));
        pay_wage(world, pop_entity, wage);
    }

    // Post-work effects (XP, Hazards, Durability, Improvised Tool Consumption)
    if worked {
        handle_post_work_effects(
            world,
            pop_entity,
            designation_entity,
            designation_type,
            action_type,
            tool_entity_opt,
        );

        handle_eureka_moment(world, pop_entity, designation_type, action_type);

        // Spec 218: Consume improvised materials if no tool was used
        if tool_entity_opt.is_none() {
            if let Some(res_type) = consumed_resource_type {
                // Probabilistic consumption
                let break_chance = match res_type {
                    ResourceType::Tools => 0.01,
                    _ => 0.05,
                };
                if rand::thread_rng().gen_bool(break_chance) {
                    if let Some(mut resources) = world.get_resource_mut::<ColonyResources>() {
                        resources.consume(res_type, 1.0);
                    }
                }
            }
        }
    }
}

fn check_work_completion(
    world: &World,
    target_entity: Entity,
    designation_type: DesignationType,
) -> bool {
    if world.get_entity(target_entity).is_err() {
        return true;
    }
    if designation_type == DesignationType::Repair {
        if let Some(s) = world.get::<crate::layer1::structure::Structure>(target_entity) {
            return (s.current_hp - s.max_hp).abs() < f32::EPSILON;
        }
    }
    false
}

fn handle_eureka_moment(
    world: &mut World,
    pop_entity: Entity,
    designation_type: DesignationType,
    action_type: ActionType,
) {
    let related_tech = match designation_type {
        DesignationType::Mine => Some(Tech::Masonry),
        // Add other mappings as appropriate
        _ => None,
    };

    // Fetch traits for the pop
    let traits = world.get::<Traits>(pop_entity).cloned();
    check_for_eureka_world(world, action_type, related_tech, traits);
}

/// Returns the skill type associated with a designation type.
pub(crate) const fn get_skill_for_designation(
    designation_type: DesignationType,
) -> Option<SkillType> {
    match designation_type {
        DesignationType::Mine => Some(SkillType::Mining),
        DesignationType::Chop => Some(SkillType::Forestry),
        DesignationType::Repair
        | DesignationType::Demolish
        | DesignationType::JuryRig
        | DesignationType::Cannibalize
        | DesignationType::Destroy => Some(SkillType::Construction),
        DesignationType::ClearFlora | DesignationType::CollectSample => Some(SkillType::Farming),
        DesignationType::SetZone(_) | DesignationType::Tame => None,
    }
}

/// Calculates the amount of work a pop can perform in a tick.
pub fn calculate_work_amount(
    world: &World,
    pop_entity: Entity,
    designation_type: DesignationType,
    tool_entity: Option<Entity>,
    morale: f32,
    work_speed_mod: f32,
    improvised_efficiency: f32,
) -> f32 {
    let mut tool_efficiency = if tool_entity.is_some() {
        1.0
    } else {
        improvised_efficiency
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

fn execute_work_on_designation(
    world: &mut World,
    pop_entity: Entity, // Added pop_entity for XP attribution
    designation_entity: Entity,
    designation_type: DesignationType,
    work_amount: f32,
) -> bool {
    // Ludwig: Get position for juice effects
    let pos = world.get::<GridPosition>(designation_entity).copied();

    match designation_type {
        DesignationType::Mine => {
            handle_mining_work(world, designation_entity, pop_entity, work_amount, pos)
        }
        DesignationType::Chop => {
            handle_chopping_work(world, designation_entity, pop_entity, work_amount, pos)
        }
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
        DesignationType::Destroy => execute_destroy(world, designation_entity),
        DesignationType::CollectSample => {
            if let Some(pos) = world.get::<GridPosition>(designation_entity).copied() {
                // Actor unused in current implementation
                let success =
                    crate::layer1::gene_bank::collect_sample_action(world, designation_entity, pos);
                if success {
                    world.despawn(designation_entity);
                }
                success
            } else {
                false
            }
        }
        DesignationType::SetZone(_) | DesignationType::Tame => false,
    }
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

    // Fetch MotherLode hazard if present
    let hazard_modifier = if let Some(lode) = world.get::<MotherLode>(designation_entity) {
        f64::from(lode.current_hazard)
    } else {
        1.0
    };

    handle_workplace_hazards(
        world,
        pop_entity,
        action_type,
        structure_opt.as_ref(),
        &skills,
        hazard_modifier,
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
        // Emit UnequipEvent
        world.send_event(UnequipEvent {
            actor: pop_entity,
            item: tool_entity,
            slot: "tool".to_string(),
        });

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
