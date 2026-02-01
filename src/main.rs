use anyhow::Result;
use bevy_ecs::prelude::*;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::io;
use std::time::{Duration, Instant};

// ECS Components
#[derive(Component)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Component)]
struct Renderable {
    glyph: char,
    color: Color,
}

#[derive(Component)]
struct Player;

// Resources
#[derive(Resource)]
struct GameState {
    running: bool,
    tick: u64,
}

fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Setup ECS World
    let mut world = World::new();
    world.insert_resource(GameState {
        running: true,
        tick: 0,
    });

    // Spawn player entity
    world.spawn((
        Player,
        Position { x: 40, y: 12 },
        Renderable {
            glyph: '@',
            color: Color::Yellow,
        },
    ));

    // Game loop
    let result = game_loop(&mut terminal, &mut world);

    // Cleanup terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn game_loop(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, world: &mut World) -> Result<()> {
    let tick_rate = Duration::from_millis(100);
    let mut last_tick = Instant::now();

    loop {
        // Handle input
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                handle_input(world, key);
            }
        }

        // Update game state
        if last_tick.elapsed() >= tick_rate {
            update_systems(world);
            last_tick = Instant::now();
        }

        // Render
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0), Constraint::Length(3)])
                .split(f.area());

            // Main game area
            let game_area = Block::default()
                .borders(Borders::ALL)
                .title("Project Zero - Dwarf Fortress Style");

            f.render_widget(game_area, chunks[0]);

            // Render entities
            render_entities(world, f, chunks[0]);

            // Status bar
            let game_state = world.resource::<GameState>();
            let status = Paragraph::new(format!("Tick: {} | Press 'q' to quit | WASD to move", game_state.tick))
                .style(Style::default().fg(Color::White))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL).title("Status"));

            f.render_widget(status, chunks[1]);
        })?;

        // Check if we should exit
        let game_state = world.resource::<GameState>();
        if !game_state.running {
            break;
        }
    }

    Ok(())
}

fn handle_input(world: &mut World, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') => {
            let mut state = world.resource_mut::<GameState>();
            state.running = false;
        }
        KeyCode::Char('w') | KeyCode::Up => move_player(world, 0, -1),
        KeyCode::Char('s') | KeyCode::Down => move_player(world, 0, 1),
        KeyCode::Char('a') | KeyCode::Left => move_player(world, -1, 0),
        KeyCode::Char('d') | KeyCode::Right => move_player(world, 1, 0),
        _ => {}
    }
}

fn move_player(world: &mut World, dx: i32, dy: i32) {
    let mut query = world.query_filtered::<&mut Position, With<Player>>();
    for mut pos in query.iter_mut(world) {
        pos.x += dx;
        pos.y += dy;
        // Simple bounds checking
        pos.x = pos.x.clamp(1, 78);
        pos.y = pos.y.clamp(1, 22);
    }
}

fn update_systems(world: &mut World) {
    // Increment tick counter
    let mut state = world.resource_mut::<GameState>();
    state.tick += 1;

    // Add more game systems here
}

fn render_entities(world: &mut World, f: &mut ratatui::Frame, area: ratatui::layout::Rect) {
    let mut query = world.query::<(&Position, &Renderable)>();

    for (pos, renderable) in query.iter(world) {
        // Adjust for border offset
        let screen_x = (pos.x + area.x as i32 + 1) as u16;
        let screen_y = (pos.y + area.y as i32 + 1) as u16;

        if screen_x < area.right() && screen_y < area.bottom() {
            let text = Text::styled(
                renderable.glyph.to_string(),
                Style::default().fg(renderable.color),
            );
            let para = Paragraph::new(text);

            let cell = ratatui::layout::Rect {
                x: screen_x,
                y: screen_y,
                width: 1,
                height: 1,
            };

            f.render_widget(para, cell);
        }
    }
}
