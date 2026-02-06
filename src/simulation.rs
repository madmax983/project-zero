//! Shared simulation tick logic used by all entry points.

use bevy_ecs::prelude::*;

use crate::experimental::biography::biography_monitor_system;
use crate::experimental::dreams::dream_system;
use crate::layer1::{
    advance_season_system, arrival_handler_system, check_milestones_system,
    clean_dead_residents_system, clean_dead_workers_system, cleanup_previous_assignment_system,
    consume_food_system, decay_needs_system, evaluate_actions_system,
    kill_starving_entities_system, movement_system, process_refining_system,
    process_research_system, process_start_plan_system, produce_food_system,
    restore_leisure_system, restore_rest_in_housing_system, track_plan_outcomes_system,
    update_action_timer_system, update_resource_caps_system, work_execution_system,
};
use crate::shared::time::SimulationTime;

/// Run one simulation tick: all game systems in order, then increment tick counter.
pub fn run_simulation_tick(world: &mut World) {
    evaluate_actions_system(world);
    update_action_timer_system(world);

    // Execution layer: bridge AI decisions to actual actions
    cleanup_previous_assignment_system(world);
    process_start_plan_system(world);
    movement_system(world);
    arrival_handler_system(world);
    work_execution_system(world);

    update_resource_caps_system(world);
    advance_season_system(world);
    produce_food_system(world);
    process_refining_system(world);
    process_research_system(world);
    restore_rest_in_housing_system(world);
    restore_leisure_system(world);
    consume_food_system(world);
    decay_needs_system(world);
    kill_starving_entities_system(world);
    clean_dead_residents_system(world);
    clean_dead_workers_system(world);

    track_plan_outcomes_system(world);
    biography_monitor_system(world);
    dream_system(world);
    check_milestones_system(world);

    world.resource_mut::<SimulationTime>().tick += 1;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::setup::setup_world;
    use crate::shared::state::GameState;

    #[test]
    fn test_run_simulation_tick_increments() {
        let mut world = setup_world();
        *world.resource_mut::<GameState>() = GameState::Running;

        let tick_before = world.resource::<SimulationTime>().tick;
        run_simulation_tick(&mut world);
        let tick_after = world.resource::<SimulationTime>().tick;

        assert_eq!(tick_after, tick_before + 1);
    }

    #[test]
    fn test_run_multiple_ticks() {
        let mut world = setup_world();
        *world.resource_mut::<GameState>() = GameState::Running;

        for _ in 0..10 {
            run_simulation_tick(&mut world);
        }

        assert_eq!(world.resource::<SimulationTime>().tick, 10);
    }
}
