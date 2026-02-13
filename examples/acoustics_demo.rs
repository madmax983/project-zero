//! Demo of the Acoustics System (Experimental).
//!
//! Visualizes the `NoiseMap`, `AmbientAudioLevel`, and sound propagation.
//!
//! # Running this example
//!
//! This example requires the `nova` feature to be enabled.
//!
//! ```bash
//! cargo run --example acoustics_demo --features nova
//! ```

#![allow(unused_imports)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]

#[cfg(feature = "nova")]
use {
    bevy_ecs::prelude::*,
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
    ratatui::{
        prelude::*,
        widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    },
    scale::experimental::acoustics::{
        AmbientAudioLevel, apply_ambience_system, weather_ambience_system,
    },
    scale::layer1::acoustic::{NoiseMap, NoiseSource, update_noise_system},
    scale::layer1::map::GridPosition,
    scale::layer1::terrain::{TerrainGrid, TerrainType, generate_terrain},
    scale::layer1::weather::{WeatherState, WeatherType},
    std::{
        io,
        time::{Duration, Instant},
    },
};

#[cfg(not(feature = "nova"))]
fn main() {
    println!("❌ Feature 'nova' is NOT enabled.");
    println!("Please run with: cargo run --example acoustics_demo --features nova");
}

#[cfg(feature = "nova")]
fn main() -> anyhow::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new();

    // Run app
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

#[cfg(feature = "nova")]
struct App {
    world: World,
    schedule: Schedule,
    should_quit: bool,
    tick_count: u64,
    width: usize,
    height: usize,
    cursor: (usize, usize),
}

#[cfg(feature = "nova")]
impl App {
    fn new() -> Self {
        let width = 60;
        let height = 30;

        let mut world = World::new();

        // Initialize Resources
        world.insert_resource(generate_terrain(width, height));
        world.insert_resource(NoiseMap::new(width, height));
        world.insert_resource(WeatherState::default());
        world.insert_resource(AmbientAudioLevel::default());

        // Initialize Schedule
        let mut schedule = Schedule::default();
        schedule.add_systems((
            weather_ambience_system,
            update_noise_system,
            apply_ambience_system.after(update_noise_system),
        ));

        Self {
            world,
            schedule,
            should_quit: false,
            tick_count: 0,
            width,
            height,
            cursor: (width / 2, height / 2),
        }
    }

    fn on_tick(&mut self) {
        self.schedule.run(&mut self.world);
        self.tick_count += 1;
    }

    fn toggle_weather(&mut self) {
        let mut weather = self.world.resource_mut::<WeatherState>();
        weather.current_weather = match weather.current_weather {
            WeatherType::Clear => WeatherType::Rain,
            WeatherType::Rain => WeatherType::Storm,
            WeatherType::Storm => WeatherType::Fog,
            WeatherType::Fog => WeatherType::Heatwave,
            WeatherType::Heatwave => WeatherType::Snow,
            WeatherType::Snow => WeatherType::Clear,
        };
    }

    fn add_source(&mut self) {
        let (x, y) = self.cursor;
        self.world.spawn((
            NoiseSource {
                radius: 8.0,
                intensity: 1.0,
            },
            GridPosition {
                x: x as i32,
                y: y as i32,
            },
        ));
    }

    fn clear_sources(&mut self) {
        let mut query = self.world.query::<(Entity, &NoiseSource)>();
        let entities: Vec<Entity> = query.iter(&self.world).map(|(e, _)| e).collect();
        for e in entities {
            self.world.despawn(e);
        }
    }

    fn move_cursor(&mut self, dx: i32, dy: i32) {
        let nx = (self.cursor.0 as i32 + dx).clamp(0, self.width as i32 - 1);
        let ny = (self.cursor.1 as i32 + dy).clamp(0, self.height as i32 - 1);
        self.cursor = (nx as usize, ny as usize);
    }
}

#[cfg(feature = "nova")]
fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    let tick_rate = Duration::from_millis(100);
    let mut last_tick = Instant::now();

    loop {
        terminal
            .draw(|f| ui(f, app))
            .map_err(|e| io::Error::other(e.to_string()))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
                    KeyCode::Char('w') => app.toggle_weather(),
                    KeyCode::Enter => app.add_source(),
                    KeyCode::Char('c') => app.clear_sources(),
                    KeyCode::Left => app.move_cursor(-1, 0),
                    KeyCode::Right => app.move_cursor(1, 0),
                    KeyCode::Up => app.move_cursor(0, -1),
                    KeyCode::Down => app.move_cursor(0, 1),
                    _ => {}
                },
                _ => {}
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

#[cfg(feature = "nova")]
fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(62),        // Grid (60 + borders)
            Constraint::Percentage(30), // Stats
        ])
        .split(f.area());

    render_grid(f, app, chunks[0]);
    render_stats(f, app, chunks[1]);
}

#[cfg(feature = "nova")]
fn render_grid(f: &mut Frame, app: &mut App, area: Rect) {
    let grid_block = Block::default()
        .borders(Borders::ALL)
        .title(" Acoustic Simulation ")
        .border_type(BorderType::Rounded);

    let grid_inner = grid_block.inner(area);
    f.render_widget(grid_block, area);

    // Query sources first to avoid borrow conflicts
    let sources_pos: Vec<(i32, i32)> = app
        .world
        .query::<(&NoiseSource, &GridPosition)>()
        .iter(&app.world)
        .map(|(_, p)| (p.x, p.y))
        .collect();

    let noise_map = app.world.resource::<NoiseMap>();
    let terrain = app.world.resource::<TerrainGrid>();

    let mut text = Vec::new();
    for y in 0..app.height {
        let mut spans = Vec::new();
        for x in 0..app.width {
            let val = noise_map.get(x as i32, y as i32);
            let tile = terrain.get(x, y).unwrap_or(TerrainType::Grass);

            // Determine base char/color from terrain
            let (mut char, mut color) = match tile {
                TerrainType::Rock => ('#', Color::DarkGray),
                TerrainType::Tree => ('T', Color::Green),
                TerrainType::Water => ('~', Color::Blue),
                _ => ('·', Color::DarkGray), // Grass/Dirt
            };

            // Overlay noise intensity
            if val > 0.8 {
                char = '█';
                color = Color::Red;
            } else if val > 0.6 {
                char = '▓';
                color = Color::LightRed;
            } else if val > 0.4 {
                char = '▒';
                color = Color::Yellow;
            } else if val > 0.2 && tile == TerrainType::Grass {
                char = '░';
                color = Color::LightGreen;
            }

            // Draw cursor
            if x == app.cursor.0 && y == app.cursor.1 {
                color = Color::White;
                char = '+';
            }

            // Draw Source
            if sources_pos.contains(&(x as i32, y as i32)) {
                char = 'S';
                color = Color::White;
            }

            let bg = if x == app.cursor.0 && y == app.cursor.1 {
                Color::Blue
            } else if sources_pos.contains(&(x as i32, y as i32)) {
                Color::Red
            } else {
                Color::Reset
            };

            spans.push(Span::styled(
                char.to_string(),
                Style::default().fg(color).bg(bg),
            ));
        }
        text.push(Line::from(spans));
    }

    let grid_paragraph = Paragraph::new(text);
    f.render_widget(grid_paragraph, grid_inner);
}

#[cfg(feature = "nova")]
fn render_stats(f: &mut Frame, app: &mut App, area: Rect) {
    let source_count = app.world.query::<&NoiseSource>().iter(&app.world).count();

    let weather = app.world.resource::<WeatherState>();
    let ambience = app.world.resource::<AmbientAudioLevel>();

    let stats_block = Block::default()
        .borders(Borders::ALL)
        .title(" Controls & Stats ");

    let stats_text = vec![
        Line::from(vec![
            Span::raw("Weather: "),
            Span::styled(
                weather.current_weather.name(),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(vec![
            Span::raw("Ambient Level: "),
            Span::styled(
                format!("{:.2}", ambience.level),
                Style::default().fg(Color::Yellow),
            ),
        ]),
        Line::from(format!("Sources: {source_count}")),
        Line::from(""),
        Line::from("Controls:"),
        Line::from(" [Arrows] Move Cursor"),
        Line::from(" [Enter]  Add Source"),
        Line::from(" [w]      Cycle Weather"),
        Line::from(" [c]      Clear Sources"),
        Line::from(" [q]      Quit"),
        Line::from(""),
        Line::from("Legend:"),
        Line::from(vec![
            Span::raw(" · "),
            Span::styled("Quiet (Grass)", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::raw(" ░ "),
            Span::styled("Low Noise", Style::default().fg(Color::LightGreen)),
        ]),
        Line::from(vec![
            Span::raw(" ▒ "),
            Span::styled("Med Noise", Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::raw(" ▓ "),
            Span::styled("High Noise", Style::default().fg(Color::LightRed)),
        ]),
        Line::from(vec![
            Span::raw(" █ "),
            Span::styled("Deafening", Style::default().fg(Color::Red)),
        ]),
        Line::from(vec![
            Span::raw(" # "),
            Span::styled("Rock (Damps)", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::raw(" T "),
            Span::styled("Tree (Damps)", Style::default().fg(Color::Green)),
        ]),
    ];

    let stats_paragraph = Paragraph::new(stats_text)
        .block(stats_block)
        .wrap(Wrap { trim: true });

    f.render_widget(stats_paragraph, area);
}
