//! Demo of the Miasma Grid (Experimental).
//!
//! # Running this example
//!
//! This example requires the `nova` feature to be enabled.
//!
//! ```bash
//! cargo run --example miasma_test --features nova
//! ```

#![allow(unused_imports)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]

#[cfg(feature = "nova")]
use {
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
        execute,
        terminal::{
            disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
        },
    },
    ratatui::{
        prelude::*,
        widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    },
    scale::experimental::miasma::MiasmaGrid,
    std::{
        io,
        time::{Duration, Instant},
    },
};

#[cfg(not(feature = "nova"))]
fn main() {
    println!("❌ Feature 'nova' is NOT enabled.");
    println!("Please run with: cargo run --example miasma_test --features nova");
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
    grid: MiasmaGrid,
    sources: Vec<(usize, usize)>, // (x, y) of constant sources
    paused: bool,
    should_quit: bool,
    tick_count: u64,
}

#[cfg(feature = "nova")]
impl App {
    fn new() -> Self {
        let width = 40;
        let height = 20;
        Self {
            grid: MiasmaGrid::new(width, height),
            sources: Vec::new(),
            paused: false,
            should_quit: false,
            tick_count: 0,
        }
    }

    fn on_tick(&mut self) {
        if !self.paused {
            // Add from sources
            for &(x, y) in &self.sources {
                self.grid.add(x as i32, y as i32, 0.2); // Add significant miasma
            }

            // Diffuse
            self.grid.diffuse();
            self.tick_count += 1;
        }
    }

    fn add_random_source(&mut self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(0..self.grid.width);
        let y = rng.gen_range(0..self.grid.height);
        self.sources.push((x, y));
    }

    fn clear(&mut self) {
        self.grid = MiasmaGrid::new(self.grid.width, self.grid.height);
        self.sources.clear();
        self.tick_count = 0;
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
                    KeyCode::Char(' ') => app.paused = !app.paused,
                    KeyCode::Enter => app.add_random_source(),
                    KeyCode::Char('c') => app.clear(),
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
fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(42),        // Grid (40 + borders)
            Constraint::Percentage(30), // Stats
        ])
        .split(f.area());

    // --- Grid Render ---
    let grid_block = Block::default()
        .borders(Borders::ALL)
        .title(" Miasma Grid ")
        .border_type(BorderType::Rounded);

    let grid_inner = grid_block.inner(chunks[0]);
    f.render_widget(grid_block, chunks[0]);

    // We render the grid character by character or line by line
    let mut text = Vec::new();
    for y in 0..app.grid.height {
        let mut spans = Vec::new();
        for x in 0..app.grid.width {
            let val = app.grid.get(x as i32, y as i32);
            let (char, color) = if val < 0.05 {
                ('·', Color::DarkGray)
            } else if val < 0.3 {
                (':', Color::Green)
            } else if val < 0.6 {
                ('+', Color::Yellow)
            } else if val < 0.9 {
                ('%', Color::Red)
            } else {
                ('#', Color::Magenta)
            };

            // Check if source
            let is_source = app.sources.contains(&(x, y));
            let final_char = if is_source { '@' } else { char };
            let final_color = if is_source { Color::White } else { color };
            let bg = if is_source { Color::Red } else { Color::Reset };

            spans.push(Span::styled(
                final_char.to_string(),
                Style::default().fg(final_color).bg(bg),
            ));
        }
        text.push(Line::from(spans));
    }

    let grid_paragraph = Paragraph::new(text);
    f.render_widget(grid_paragraph, grid_inner);

    // --- Stats Panel ---
    let stats_block = Block::default()
        .borders(Borders::ALL)
        .title(" Controls & Stats ");

    let stats_text = vec![
        Line::from(vec![
            Span::raw("Status: "),
            if app.paused {
                Span::styled("PAUSED", Style::default().fg(Color::Red))
            } else {
                Span::styled("RUNNING", Style::default().fg(Color::Green))
            },
        ]),
        Line::from(format!("Ticks: {}", app.tick_count)),
        Line::from(""),
        Line::from(vec![
            Span::styled("Sources: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{}", app.sources.len())),
        ]),
        Line::from(""),
        Line::from("Controls:"),
        Line::from(" [Space] Pause/Resume"),
        Line::from(" [Enter] Add Source"),
        Line::from(" [c]     Clear Grid"),
        Line::from(" [q]     Quit"),
        Line::from(""),
        Line::from("Legend:"),
        Line::from(vec![
            Span::raw(" · "),
            Span::styled("Safe", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::raw(" : "),
            Span::styled("Low", Style::default().fg(Color::Green)),
        ]),
        Line::from(vec![
            Span::raw(" + "),
            Span::styled("Med", Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::raw(" % "),
            Span::styled("High", Style::default().fg(Color::Red)),
        ]),
        Line::from(vec![
            Span::raw(" # "),
            Span::styled("Toxic", Style::default().fg(Color::Magenta)),
        ]),
        Line::from(vec![
            Span::styled(" @ ", Style::default().bg(Color::Red).fg(Color::White)),
            Span::raw(" Source"),
        ]),
    ];

    let stats_paragraph = Paragraph::new(stats_text)
        .block(stats_block)
        .wrap(Wrap { trim: true });

    f.render_widget(stats_paragraph, chunks[1]);
}
