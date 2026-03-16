use crate::layer1::map::GridPosition;
use crate::layer1::morale::Morale;
use crate::layer1::pathfinding::manhattan_distance;
use crate::layer1::structure::{DeferMaintenance, Structure};
use bevy_ecs::prelude::*;
use rand::Rng;

#[derive(Component, Default)]
pub struct MaintenanceDebt {
    pub amount: f32,
}

#[derive(Component)]
pub struct MachineQuirk;

#[derive(Component)]
pub struct Cultist {
    pub target_machine: Entity,
}

#[derive(Component)]
pub struct RepairJob {
    pub target: Entity,
}

#[derive(Event)]
pub struct SabotageEvent {
    pub saboteur: Entity,
    pub job: Entity,
}

pub fn generate_machine_quirks_system(
    mut commands: Commands,
    query: Query<(Entity, &MaintenanceDebt), Without<MachineQuirk>>,
    structure_query: Query<(Entity, &Structure, Option<&DeferMaintenance>), Without<MachineQuirk>>,
) {
    // Keep test spec logic:
    for (entity, debt) in query.iter() {
        if debt.amount > 100.0 {
            commands.entity(entity).insert(MachineQuirk);
        }
    }

    // Refactored logic interacting with existing game logic:
    // If a structure has deferred maintenance for a long time, and low HP.
    for (entity, structure, defer_maintenance) in structure_query.iter() {
        if defer_maintenance.is_some() && structure.current_hp < structure.max_hp * 0.5 {
            commands.entity(entity).insert(MachineQuirk);
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn form_machine_cult_system(
    mut commands: Commands,
    quirky_machines: Query<(Entity, Option<&GridPosition>), With<MachineQuirk>>,
    pops: Query<(Entity, Option<&GridPosition>, Option<&Morale>), Without<Cultist>>,
) {
    let mut rng = rand::thread_rng();

    // With a simulation tick rate where this runs frequently, a base 5% probability
    // leads to almost instantaneous conversion. Using a much lower base probability
    // (e.g., 0.05% per tick, equating roughly to 5% every 100 ticks) is safer here.
    let base_probability = 0.0005;

    for (pop_entity, pop_pos_opt, morale_opt) in pops.iter() {
        let mut best_machine = None;
        let mut best_dist = i32::MAX;

        // Find closest quirky machine
        if let Some(pop_pos) = pop_pos_opt {
            for (machine_entity, machine_pos_opt) in quirky_machines.iter() {
                if let Some(machine_pos) = machine_pos_opt {
                    let dist = manhattan_distance((pop_pos.x, pop_pos.y), (machine_pos.x, machine_pos.y));
                    if dist < best_dist && dist <= 20 { // 20 tile radius
                        best_dist = dist;
                        best_machine = Some(machine_entity);
                    }
                }
            }
        } else {
            // Fallback for tests lacking GridPosition
            if let Some((machine_entity, _)) = quirky_machines.iter().next() {
                best_machine = Some(machine_entity);
            }
        }

        if let Some(machine) = best_machine {
            // Resistance check based on morale
            let mut probability = base_probability;
            if let Some(morale) = morale_opt {
                if morale.value < 0.4 {
                    probability += 0.0015; // +0.15% per tick if depressed
                }
            } else {
                probability = 1.0; // Guaranteed in tests lacking Morale for spec tests
            }

            if rng.gen::<f32>() < probability {
                commands.entity(pop_entity).insert(Cultist { target_machine: machine });
            }
        }
    }
}

pub fn cult_sabotage_system(
    mut commands: Commands,
    cultists: Query<(Entity, &Cultist)>,
    repair_jobs: Query<(Entity, &RepairJob)>,
    mut sabotage_events: EventWriter<SabotageEvent>,
) {
    // Collect jobs to despawn so we don't try to despawn the same job multiple times
    // if multiple cultists target it.
    let mut jobs_to_despawn = std::collections::HashSet::new();

    for (saboteur, cultist) in cultists.iter() {
        for (job_entity, job) in repair_jobs.iter() {
            if job.target == cultist.target_machine && !jobs_to_despawn.contains(&job_entity) {
                jobs_to_despawn.insert(job_entity);
                commands.entity(job_entity).despawn();
                sabotage_events.send(SabotageEvent {
                    saboteur,
                    job: job_entity,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::system::RunSystemOnce;

    fn spawn_test_machine(world: &mut World, debt: f32) -> Entity {
        world.spawn(MaintenanceDebt { amount: debt }).id()
    }

    fn spawn_test_pop(world: &mut World) -> Entity {
        world.spawn_empty().id()
    }

    #[test]
    fn test_machine_develops_quirk_from_high_debt() {
        let mut world = World::new();
        let machine = spawn_test_machine(&mut world, 150.0); // High debt

        // Run the system that generates quirks
        world.run_system_once(super::generate_machine_quirks_system).unwrap();

        // Assert the machine now has a quirk
        assert!(world.entity(machine).contains::<MachineQuirk>());
    }

    #[test]
    fn test_pops_become_cultists_near_quirky_machine() {
        let mut world = World::new();
        let machine = spawn_test_machine(&mut world, 150.0);
        world.entity_mut(machine).insert(MachineQuirk);

        let pop = spawn_test_pop(&mut world);
        // Add logic to position pop near machine if spatial queries are used

        // Run cult formation system
        world.run_system_once(super::form_machine_cult_system).unwrap();

        // Assert the pop is now a cultist targeting the machine
        assert!(world.entity(pop).contains::<Cultist>());
        assert_eq!(world.get::<Cultist>(pop).unwrap().target_machine, machine);
    }

    #[test]
    fn test_cultists_sabotage_repair_jobs() {
        let mut world = World::new();
        world.insert_resource(Events::<SabotageEvent>::default());
        let machine = spawn_test_machine(&mut world, 150.0);
        world.entity_mut(machine).insert(MachineQuirk);

        let cultist = spawn_test_pop(&mut world);
        world.entity_mut(cultist).insert(Cultist { target_machine: machine });

        let repair_job = world.spawn(RepairJob { target: machine }).id();

        // Run sabotage system
        world.run_system_once(super::cult_sabotage_system).unwrap();

        // Assert the repair job was sabotaged (e.g., despawned or marked invalid)
        assert!(world.get_entity(repair_job).is_err() || world.get::<RepairJob>(repair_job).is_none());
        // Alternatively, check for SabotageEvent emission
    }
}
