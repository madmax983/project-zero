use bevy_ecs::prelude::*;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, BorderType},
};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::time::{Duration, Instant};
use std::io;

mod layer1;
use layer1::{generate_terrain, TerrainGrid, Viewport, render_terrain};

#[derive(Resource, Default, PartialEq, Eq)]
pub enum GameState {
    #[default]
    Running,
    Paused,
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

    let mut schedule = Schedule::default();
    // Systems will be added here by other specs

    // Main loop
    let tick_rate = Duration::from_millis(100); // 10 FPS base
    let mut last_tick = Instant::now();

    loop {
        // Input
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
        KeyCode::Char('w')
        | KeyCode::Up
        | KeyCode::Char('s')
        | KeyCode::Down
        | KeyCode::Char('a')
        | KeyCode::Left
        | KeyCode::Char('d')
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
    let area = frame.area();

    let block = Block::default()
        .title(" SCALE ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    let inner_area = block.inner(area);

    let terrain = world.resource::<TerrainGrid>();
    let viewport = world.resource::<Viewport>();
    render_terrain(frame, inner_area, terrain, viewport);

    frame.render_widget(block, area);
}
