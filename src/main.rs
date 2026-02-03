//! SCALE executable entry point.

use bevy_ecs::prelude::*;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::prelude::*;
use scale::shared::log::MessageLog;
use std::io;
use std::time::{Duration, Instant};

use scale::layer1::{
    BuildMode, BuildingTracker, Chronicle, ChronicleUiState, ColonyResources, DesignationMode,
    OccupiedTiles, Viewport, check_milestones_system, clean_dead_residents_system,
    clean_dead_workers_system, consume_food_system, decay_needs_system, generate_terrain,
    initial_chronicle_event, kill_starving_entities_system, produce_food_system,
    restore_rest_in_housing_system, spawn_initial_pops,
};
use scale::shared::input::{InputContextStack, InputRouter};
use scale::shared::selection::Selection;
use scale::shared::state::GameState;
use scale::shared::time::{SimSpeed, SimulationTime};
use scale::ui::render::render_app;

fn main() -> anyhow::Result<()> {
    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run the app, ensuring cleanup happens afterwards
    let result = run_app_loop(&mut terminal);

    // Cleanup
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

fn run_app_loop(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> anyhow::Result<()> {
    // ECS setup
    let mut world = World::new();
    world.insert_resource(GameState::Running);
    world.insert_resource(generate_terrain(80, 50));
    world.insert_resource(Viewport::default());
    world.insert_resource(SimulationTime::default());
    world.insert_resource(BuildMode::default());
    world.insert_resource(DesignationMode::default());
    world.insert_resource(OccupiedTiles::default());
    world.insert_resource(ColonyResources::default());
    world.insert_resource(InputContextStack::default());
    world.insert_resource(MessageLog::default());
    world.insert_resource(Chronicle::default());
    world.insert_resource(ChronicleUiState::default());
    world.insert_resource(BuildingTracker::default());
    world.insert_resource(Selection::default());

    spawn_initial_pops(&mut world);
    initial_chronicle_event(&mut world);

    let mut schedule = Schedule::default();
    let mut input_router = InputRouter::new();
    // Systems will be added here by other specs

    // Main loop
    let tick_rate = Duration::from_millis(100); // 10 FPS base
    let mut last_tick = Instant::now();

    loop {
        // Input
        #[allow(clippy::collapsible_if)]
        if event::poll(Duration::from_millis(10))? {
            match event::read()? {
                Event::Key(key) => input_router.route(&mut world, key),
                Event::Mouse(mouse) => input_router.route_mouse(&mut world, mouse),
                _ => {}
            }
        }

        // Check quit
        if *world.resource::<GameState>() == GameState::Quitting {
            break;
        }

        // Simulation tick
        if last_tick.elapsed() >= tick_rate {
            schedule.run(&mut world);

            // Update tick count
            if *world.resource::<GameState>() == GameState::Running {
                let speed = world.resource::<SimulationTime>().speed;
                if speed != SimSpeed::Paused {
                    produce_food_system(&mut world);
                    restore_rest_in_housing_system(&mut world);
                    consume_food_system(&mut world);
                    decay_needs_system(&mut world);
                    kill_starving_entities_system(&mut world);
                    clean_dead_residents_system(&mut world);
                    clean_dead_workers_system(&mut world);
                    check_milestones_system(&mut world);
                    world.resource_mut::<SimulationTime>().tick += 1;
                }
            }

            last_tick = Instant::now();
        }

        // Render
        render_app(terminal, &world)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gamestate_derives() {
        let state1 = GameState::Running;
        let state2 = state1;
        assert_eq!(state1, state2);
    }
}
