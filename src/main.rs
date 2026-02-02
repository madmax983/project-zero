//! SCALE executable entry point.
//!
//! # Test Coverage Note
//!
//! This module achieves ~57% line coverage. The untested portions are primarily:
//! - Terminal setup/teardown (requires actual terminal)
//! - Main event loop (requires Terminal mock)
//! - All rendering functions (ratatui widgets, hard to unit test)
//!
//! All testable business logic (input handling, state transitions) has >95% coverage.
//! Combined with 100% coverage in `shared/time`, overall project coverage is ~76%.

use bevy_ecs::prelude::*;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, Paragraph},
};
use std::collections::HashSet;
use std::io;
use std::time::{Duration, Instant};

use scale::layer1::{
    GridPosition, Pop, TerrainGrid, Viewport, generate_terrain, render_terrain_and_pops,
    spawn_initial_pops,
};
use scale::shared::time::{SimSpeed, SimulationTime};

/// Represents the high-level state of the game loop.
#[derive(Resource, Default, PartialEq, Eq, Clone, Copy, Debug)]
pub enum GameState {
    /// The simulation is running normally.
    #[default]
    Running,
    /// The simulation is paused, but input is still handled.
    Paused,
    /// The game is in the process of shutting down.
    Quitting,
}

fn main() -> anyhow::Result<()> {
    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run the app, ensuring cleanup happens afterwards
    let result = run_app(&mut terminal);

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> anyhow::Result<()> {
    // ECS setup
    let mut world = World::new();
    world.insert_resource(GameState::Running);
    world.insert_resource(generate_terrain(80, 50));
    world.insert_resource(Viewport::default());
    world.insert_resource(SimulationTime::default());

    spawn_initial_pops(&mut world);

    let mut schedule = Schedule::default();
    // Systems will be added here by other specs

    // Main loop
    let tick_rate = Duration::from_millis(100); // 10 FPS base
    let mut last_tick = Instant::now();

    loop {
        // Input
        // event::read() must only be called after event::poll() indicates that an event is available.
        // The nested if structure preserves this ordering and cannot be safely collapsed.
        #[allow(clippy::collapsible_if)]
        if event::poll(Duration::from_millis(10))? {
            if let Event::Key(key) = event::read()? {
                handle_input(&mut world, key);
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
                    world.resource_mut::<SimulationTime>().tick += 1;
                }
            }

            last_tick = Instant::now();
        }

        // Render
        terminal.draw(|frame| render(&world, frame))?;
    }

    Ok(())
}

fn handle_input(world: &mut World, key: crossterm::event::KeyEvent) {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            *world.resource_mut::<GameState>() = GameState::Quitting;
        }
        KeyCode::Char(' ') => {
            let mut state = world.resource_mut::<GameState>();
            *state = match *state {
                GameState::Running => GameState::Paused,
                GameState::Paused => GameState::Running,
                GameState::Quitting => GameState::Quitting,
            };
        }
        KeyCode::Char('1') => {
            world.resource_mut::<SimulationTime>().speed = SimSpeed::Normal;
        }
        KeyCode::Char('2') => {
            world.resource_mut::<SimulationTime>().speed = SimSpeed::Fast;
        }
        KeyCode::Char('3') => {
            world.resource_mut::<SimulationTime>().speed = SimSpeed::Faster;
        }
        KeyCode::Char('w' | 's' | 'a' | 'd')
        | KeyCode::Up
        | KeyCode::Down
        | KeyCode::Left
        | KeyCode::Right => {
            let mut viewport = world.resource_mut::<Viewport>();
            match key.code {
                KeyCode::Char('w') | KeyCode::Up => {
                    viewport.y -= 1;
                }
                KeyCode::Char('s') | KeyCode::Down => {
                    viewport.y += 1;
                }
                KeyCode::Char('a') | KeyCode::Left => {
                    viewport.x -= 1;
                }
                KeyCode::Char('d') | KeyCode::Right => {
                    viewport.x += 1;
                }
                _ => {}
            }
        }
        _ => {}
    }
}

fn render(world: &World, frame: &mut Frame) {
    // Main vertical split: content + status bar
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),   // Content area
            Constraint::Length(1), // Status bar
        ])
        .split(frame.area());

    // Horizontal split: map + info panel
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(20),    // Map area
            Constraint::Length(20), // Info panel
        ])
        .split(main_chunks[0]);

    let map_area = content_chunks[0];
    let info_area = content_chunks[1];
    let status_area = main_chunks[1];

    // Render map
    render_map(frame, map_area, world);

    // Render info panel
    render_info_panel(frame, info_area, world);

    // Render status bar
    render_status_bar(frame, status_area, world);
}

fn render_map(frame: &mut Frame, area: Rect, world: &World) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" Colony ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Render terrain inside
    let terrain = world.resource::<TerrainGrid>();
    let viewport = world.resource::<Viewport>();

    let pop_positions = get_pop_positions(world);

    render_terrain_and_pops(frame, inner, terrain, viewport, &pop_positions);
}

fn get_pop_positions(world: &World) -> HashSet<(i32, i32)> {
    world
        .iter_entities()
        .filter(bevy_ecs::world::EntityRef::contains::<Pop>)
        .filter_map(|e| e.get::<GridPosition>())
        .map(|pos| (pos.x, pos.y))
        .collect()
}

fn render_info_panel(frame: &mut Frame, area: Rect, _world: &World) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" Info ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Placeholder text
    let text = Paragraph::new("Select something\nto see info here");
    frame.render_widget(text, inner);
}

fn render_status_bar(frame: &mut Frame, area: Rect, world: &World) {
    let sim_time = world.resource::<SimulationTime>();
    let game_state = world.resource::<GameState>();

    // NOTE: Dual pause state check. GameState::Paused is controlled by spacebar,
    // SimSpeed::Paused exists but is currently not used (no key binds to it).
    // This allows for future distinction between "paused but still simulating at 0x"
    // vs "completely frozen". Current behavior: only GameState::Paused matters (main.rs:86).
    let paused = *game_state == GameState::Paused || sim_time.speed == SimSpeed::Paused;

    let status = get_status_string(sim_time.tick, sim_time.speed, paused);

    let bar = Paragraph::new(status).style(Style::default().bg(Color::DarkGray).fg(Color::White));
    frame.render_widget(bar, area);
}

fn get_status_string(tick: u64, speed: SimSpeed, paused: bool) -> String {
    format!(
        " {} │ Tick: {} │ {} │ WASD:Move  Space:Pause  1-3:Speed  q:Quit ",
        if paused { "⏸" } else { "▶" },
        tick,
        speed.label(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn create_test_world() -> World {
        let mut world = World::new();
        world.insert_resource(GameState::Running);
        world.insert_resource(SimulationTime::default());
        world.insert_resource(Viewport::default());
        world
    }

    fn key_event(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    #[test]
    fn test_quit_key() {
        let mut world = create_test_world();
        handle_input(&mut world, key_event(KeyCode::Char('q')));
        assert_eq!(*world.resource::<GameState>(), GameState::Quitting);
    }

    #[test]
    fn test_escape_key() {
        let mut world = create_test_world();
        handle_input(&mut world, key_event(KeyCode::Esc));
        assert_eq!(*world.resource::<GameState>(), GameState::Quitting);
    }

    #[test]
    fn test_pause_toggle() {
        let mut world = create_test_world();
        assert_eq!(*world.resource::<GameState>(), GameState::Running);

        // Pause
        handle_input(&mut world, key_event(KeyCode::Char(' ')));
        assert_eq!(*world.resource::<GameState>(), GameState::Paused);

        // Unpause
        handle_input(&mut world, key_event(KeyCode::Char(' ')));
        assert_eq!(*world.resource::<GameState>(), GameState::Running);
    }

    #[test]
    fn test_speed_key_1() {
        let mut world = create_test_world();
        world.resource_mut::<SimulationTime>().speed = SimSpeed::Fast;

        handle_input(&mut world, key_event(KeyCode::Char('1')));
        assert_eq!(world.resource::<SimulationTime>().speed, SimSpeed::Normal);
    }

    #[test]
    fn test_speed_key_2() {
        let mut world = create_test_world();
        handle_input(&mut world, key_event(KeyCode::Char('2')));
        assert_eq!(world.resource::<SimulationTime>().speed, SimSpeed::Fast);
    }

    #[test]
    fn test_speed_key_3() {
        let mut world = create_test_world();
        handle_input(&mut world, key_event(KeyCode::Char('3')));
        assert_eq!(world.resource::<SimulationTime>().speed, SimSpeed::Faster);
    }

    #[test]
    fn test_viewport_movement_wasd() {
        let mut world = create_test_world();
        let initial_x = world.resource::<Viewport>().x;
        let initial_y = world.resource::<Viewport>().y;

        // Move up (w)
        handle_input(&mut world, key_event(KeyCode::Char('w')));
        assert_eq!(world.resource::<Viewport>().y, initial_y - 1);

        // Move down (s)
        handle_input(&mut world, key_event(KeyCode::Char('s')));
        assert_eq!(world.resource::<Viewport>().y, initial_y);

        // Move left (a)
        handle_input(&mut world, key_event(KeyCode::Char('a')));
        assert_eq!(world.resource::<Viewport>().x, initial_x - 1);

        // Move right (d)
        handle_input(&mut world, key_event(KeyCode::Char('d')));
        assert_eq!(world.resource::<Viewport>().x, initial_x);
    }

    #[test]
    fn test_viewport_movement_arrows() {
        let mut world = create_test_world();
        let initial_x = world.resource::<Viewport>().x;
        let initial_y = world.resource::<Viewport>().y;

        handle_input(&mut world, key_event(KeyCode::Up));
        assert_eq!(world.resource::<Viewport>().y, initial_y - 1);

        handle_input(&mut world, key_event(KeyCode::Down));
        assert_eq!(world.resource::<Viewport>().y, initial_y);

        handle_input(&mut world, key_event(KeyCode::Left));
        assert_eq!(world.resource::<Viewport>().x, initial_x - 1);

        handle_input(&mut world, key_event(KeyCode::Right));
        assert_eq!(world.resource::<Viewport>().x, initial_x);
    }

    #[test]
    fn test_speed_persists_across_pause() {
        let mut world = create_test_world();

        // Set speed to Fast
        handle_input(&mut world, key_event(KeyCode::Char('2')));
        assert_eq!(world.resource::<SimulationTime>().speed, SimSpeed::Fast);

        // Pause
        handle_input(&mut world, key_event(KeyCode::Char(' ')));
        assert_eq!(*world.resource::<GameState>(), GameState::Paused);
        assert_eq!(world.resource::<SimulationTime>().speed, SimSpeed::Fast);

        // Unpause
        handle_input(&mut world, key_event(KeyCode::Char(' ')));
        assert_eq!(*world.resource::<GameState>(), GameState::Running);
        assert_eq!(world.resource::<SimulationTime>().speed, SimSpeed::Fast);
    }

    #[test]
    fn test_pause_doesnt_quit() {
        let mut world = create_test_world();
        *world.resource_mut::<GameState>() = GameState::Paused;

        // Space when paused should unpause, not stay paused
        handle_input(&mut world, key_event(KeyCode::Char(' ')));
        assert_eq!(*world.resource::<GameState>(), GameState::Running);
    }

    #[test]
    fn test_quitting_state_stays_quitting() {
        let mut world = create_test_world();
        *world.resource_mut::<GameState>() = GameState::Quitting;

        // Space when quitting should stay quitting
        handle_input(&mut world, key_event(KeyCode::Char(' ')));
        assert_eq!(*world.resource::<GameState>(), GameState::Quitting);
    }

    #[test]
    fn test_gamestate_derives() {
        // Test that GameState can be cloned, copied, compared
        let state1 = GameState::Running;
        let state2 = state1; // Copy
        assert_eq!(state1, state2);

        let state3 = GameState::default();
        assert_eq!(state3, GameState::Running);

        // Test Debug formatting
        let debug_str = format!("{state1:?}");
        assert!(debug_str.contains("Running"));
    }

    #[test]
    fn test_get_pop_positions() {
        let mut world = create_test_world();
        world.spawn((Pop, GridPosition { x: 10, y: 20 }));
        world.spawn((Pop, GridPosition { x: 5, y: 5 }));
        // Entity without Pop component
        world.spawn(GridPosition { x: 99, y: 99 });

        let positions = get_pop_positions(&world);

        assert_eq!(positions.len(), 2);
        assert!(positions.contains(&(10, 20)));
        assert!(positions.contains(&(5, 5)));
        assert!(!positions.contains(&(99, 99)));
    }

    #[test]
    fn test_get_status_string() {
        let s = get_status_string(100, SimSpeed::Normal, false);
        assert!(s.contains("Tick: 100"));
        assert!(s.contains("▶"));
        assert!(s.contains("1x"));

        let s_paused = get_status_string(50, SimSpeed::Fast, true);
        assert!(s_paused.contains("Tick: 50"));
        assert!(s_paused.contains("⏸"));
        assert!(s_paused.contains("3x"));
    }
}
