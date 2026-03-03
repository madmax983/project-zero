#![deny(unsafe_code)]
//! SCALE native terminal entry point.

use bevy_ecs::system::RunSystemOnce;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use scale::layer1::map::{update_camera_smooth, update_screen_shake_system};
use scale::layer1::GlobalHitStop;
use scale::platform::input::{GameKeyEvent, GameMouseEvent};
use scale::setup::setup_world;
use scale::shared::input::{route_input, route_mouse_input};
use scale::shared::state::GameState;
use scale::shared::time::{SimSpeed, SimulationTime, WallTime};
use scale::simulation::run_simulation_tick;
use scale::ui::map::update_render_cache;
use scale::ui::render;
use std::io;
use std::time::{Duration, Instant};

fn main() -> anyhow::Result<()> {
    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run the app, ensuring cleanup happens afterwards
    let result = run_app(&mut terminal);

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

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> anyhow::Result<()> {
    let mut world = setup_world();

    // Main loop
    let tick_rate = Duration::from_millis(100); // 10 FPS base
    let mut last_tick = Instant::now();
    let mut last_frame = Instant::now();

    loop {
        let now = Instant::now();
        let delta = now.duration_since(last_frame);
        last_frame = now;

        world.resource_mut::<WallTime>().0 += delta.as_secs_f32();

        // Input
        // event::read() must only be called after event::poll() indicates that an event is available.
        // The nested if structure preserves this ordering and cannot be safely collapsed.
        #[allow(clippy::collapsible_if)]
        if event::poll(Duration::from_millis(10))? {
            match event::read()? {
                Event::Key(key) => {
                    if let Ok(game_key) = GameKeyEvent::try_from(key) {
                        route_input(&mut world, game_key);
                    }
                }
                Event::Mouse(mouse) => {
                    if let Ok(game_mouse) = GameMouseEvent::try_from(mouse) {
                        route_mouse_input(&mut world, game_mouse);
                    }
                }
                _ => {}
            }
        }

        // Check quit
        if *world.resource::<GameState>() == GameState::Quitting {
            break;
        }

        // Simulation tick
        if last_tick.elapsed() >= tick_rate {
            // Check Global Hit Stop (Juice Freeze)
            let mut hit_stop_active = false;
            if let Some(mut hs) = world.get_resource_mut::<GlobalHitStop>() {
                if hs.ticks > 0 {
                    hs.ticks -= 1;
                    hit_stop_active = true;
                }
            }

            if !hit_stop_active && *world.resource::<GameState>() == GameState::Running {
                let speed = world.resource::<SimulationTime>().speed;
                if speed != SimSpeed::Paused {
                    run_simulation_tick(&mut world);
                }
            }

            last_tick = Instant::now();
        }

        // Prepare render data
        // We run screen shake here to ensure it updates even during hit stop
        world.run_system_once(update_screen_shake_system).unwrap();
        update_camera_smooth(&mut world);
        update_render_cache(&mut world);

        // Render
        terminal.draw(|frame| render(&world, frame))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use scale::shared::state::GameState;

    #[test]
    fn test_gamestate_derives() {
        // Test that GameState can be cloned, copied, compared
        let state1 = GameState::Running;
        let state2 = state1; // Copy
        assert_eq!(state1, state2);

        let state3 = GameState::default();
        assert_eq!(state3, GameState::MainMenu);

        // Test Debug formatting
        let debug_str = format!("{state1:?}");
        assert!(debug_str.contains("Running"));
    }
}
