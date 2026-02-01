//! SCALE executable entry point.

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
use std::io;
use std::time::{Duration, Instant};

use scale::layer1::{TerrainGrid, Viewport, generate_terrain, render_terrain};
use scale::shared::time::{SimSpeed, SimulationTime};

/// Represents the high-level state of the game loop.
#[derive(Resource, Default, PartialEq, Eq, Clone, Copy)]
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
    render_terrain(frame, inner, terrain, viewport);
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

    let paused = *game_state == GameState::Paused || sim_time.speed == SimSpeed::Paused;

    let status = format!(
        " {} │ Tick: {} │ {} │ WASD:Move  Space:Pause  1-3:Speed  q:Quit ",
        if paused { "⏸" } else { "▶" },
        sim_time.tick,
        sim_time.speed.label(),
    );

    let bar = Paragraph::new(status).style(Style::default().bg(Color::DarkGray).fg(Color::White));
    frame.render_widget(bar, area);
}
