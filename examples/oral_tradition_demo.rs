//! Demo of the Oral Tradition system ("Nova" feature).
//!
//! This example visualizes how historical events (Chronicle) evolve into legends (Oral Tradition)
//! as they are retold in taverns.
//!
//! Run with: `cargo run --features nova --example oral_tradition_demo`

#[cfg(feature = "nova")]
mod app {
    use bevy_ecs::prelude::*;
    use crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    };
    use ratatui::{prelude::*, widgets::*};
    use scale::layer1::chronicle::{Chronicle, EventImportance};
    use scale::layer1::needs::Needs;
    use scale::layer1::oral_tradition::{
        collect_chronicles_system, storytelling_system, OralTradition, StoryGenre,
    };
    use scale::layer1::social::Tavern;
    use scale::shared::log::MessageLog;
    use scale::shared::time::SimulationTime;
    use std::io;
    use std::time::Duration;

    pub struct App {
        world: World,
        state: TableState,
        should_quit: bool,
        last_tick: std::time::Instant,
        notification: Option<String>,
        notification_timer: Option<std::time::Instant>,
    }

    impl App {
        pub fn new() -> Self {
            let mut world = World::new();

            // Initialize Resources
            world.insert_resource(OralTradition::default());
            world.insert_resource(Chronicle::default());
            world.insert_resource(MessageLog::default());
            world.insert_resource(SimulationTime::default());

            // Spawn Tavern and Visitors (Pops) to simulate storytelling environment
            let mut tavern = Tavern::default();
            // Spawn 5 pops with needs
            for _ in 0..5 {
                let pop = world
                    .spawn(Needs {
                        leisure: 0.5,
                        rest: 0.5,
                        ..Default::default()
                    })
                    .id();
                tavern.visitors.push(pop);
            }
            world.spawn(tavern);

            // Seed initial history
            let mut chronicle = world.resource_mut::<Chronicle>();
            chronicle.add_event(
                0,
                "The colony was founded on a barren rock.".to_string(),
                EventImportance::Legendary,
            );
            chronicle.add_event(
                100,
                "First harvest was bountiful.".to_string(),
                EventImportance::Major,
            );

            let mut state = TableState::default();
            state.select(Some(0));

            Self {
                world,
                state,
                should_quit: false,
                last_tick: std::time::Instant::now(),
                notification: None,
                notification_timer: None,
            }
        }

        pub fn run(&mut self) -> io::Result<()> {
            // Setup terminal
            enable_raw_mode()?;
            let mut stdout = io::stdout();
            execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
            let backend = CrosstermBackend::new(stdout);
            let mut terminal = Terminal::new(backend)?;

            let res = self.run_app(&mut terminal);

            // Restore terminal
            disable_raw_mode()?;
            execute!(
                terminal.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            )?;
            terminal.show_cursor()?;

            res
        }

        fn run_app<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
            loop {
                terminal
                    .draw(|f| self.ui(f))
                    .map_err(|e| io::Error::other(e.to_string()))?;

                // Handle Input
                if event::poll(Duration::from_millis(50))? {
                    if let Event::Key(key) = event::read()? {
                        if key.kind == KeyEventKind::Press {
                            match key.code {
                                KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
                                KeyCode::Down => self.next(),
                                KeyCode::Up => self.previous(),
                                KeyCode::Char(' ') => self.tick_simulation(),
                                KeyCode::Char('a') => self.add_random_event(),
                                _ => {}
                            }
                        }
                    }
                }

                // Auto-tick simulation every 100ms
                if self.last_tick.elapsed() >= Duration::from_millis(100) {
                    self.tick_simulation();
                    self.last_tick = std::time::Instant::now();
                }

                // Clear notification after 2 seconds
                if let Some(timer) = self.notification_timer {
                    if timer.elapsed() >= Duration::from_secs(2) {
                        self.notification = None;
                        self.notification_timer = None;
                    }
                }

                if self.should_quit {
                    return Ok(());
                }
            }
        }

        fn tick_simulation(&mut self) {
            // Advance time
            self.world.resource_mut::<SimulationTime>().tick += 1;

            // Run Systems
            let mut schedule = Schedule::default();
            schedule.add_systems((collect_chronicles_system, storytelling_system));
            schedule.run(&mut self.world);
        }

        fn add_random_event(&mut self) {
            use rand::Rng;
            let tick = self.world.resource::<SimulationTime>().tick;
            let mut rng = rand::thread_rng();

            let events = [
                ("A meteor struck the warehouse!", EventImportance::Legendary),
                ("Famine struck the outer rim.", EventImportance::Major),
                ("A mysterious stranger arrived.", EventImportance::Standard),
                ("The reactor core stabilized.", EventImportance::Standard),
                ("A ghostly figure was seen.", EventImportance::Minor),
                (
                    "The ancient prophecy was fulfilled.",
                    EventImportance::Legendary,
                ),
            ];

            let (text, importance) = events[rng.gen_range(0..events.len())];
            self.world
                .resource_mut::<Chronicle>()
                .add_event(tick, text.to_string(), importance);

            self.notification = Some(format!("Event Added: {}", text));
            self.notification_timer = Some(std::time::Instant::now());
        }

        fn next(&mut self) {
            let count = self.world.resource::<OralTradition>().stories.len();
            if count == 0 {
                return;
            }
            let i = self
                .state
                .selected()
                .map_or(0, |i| if i >= count - 1 { 0 } else { i + 1 });
            self.state.select(Some(i));
        }

        fn previous(&mut self) {
            let count = self.world.resource::<OralTradition>().stories.len();
            if count == 0 {
                return;
            }
            let i = self
                .state
                .selected()
                .map_or(0, |i| if i == 0 { count - 1 } else { i - 1 });
            self.state.select(Some(i));
        }

        fn ui(&mut self, f: &mut Frame) {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Header
                    Constraint::Min(0),    // Content
                    Constraint::Length(3), // Footer
                ])
                .split(f.area());

            self.render_header(f, chunks[0]);

            // Main Content
            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(50), // Stories List
                    Constraint::Percentage(50), // Details
                ])
                .split(chunks[1]);

            self.render_table(f, main_chunks[0]);
            self.render_details(f, main_chunks[1]);
            self.render_footer(f, chunks[2]);
        }

        fn render_header(&self, f: &mut Frame, area: Rect) {
            let time = self.world.resource::<SimulationTime>();
            let header_text = format!(
                "Oral Tradition Explorer | Date: {} | Speed: Auto",
                time.tick
            );

            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(Span::styled(
                    " Mosaic UI Polish ",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ));

            let paragraph = Paragraph::new(header_text)
                .style(Style::default().fg(Color::White))
                .block(block)
                .alignment(Alignment::Center);

            f.render_widget(paragraph, area);

            // Notification Overlay (Right side of header)
            if let Some(msg) = &self.notification {
                let layout = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Min(0), Constraint::Length(60)])
                    .split(area);

                // Use the right side for notifications
                // We need to render it on top or just in the corner
                let notif_area = layout[1];
                // Manually adjust area to fit inside border?
                // Actually, just render a paragraph
                let notif = Paragraph::new(Span::styled(
                    format!("🔔 {}", msg),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ))
                .alignment(Alignment::Right)
                .block(Block::default().borders(Borders::NONE)); // No border to blend in

                // Render on top of the header content (technically z-index is order of rendering, so this is fine)
                f.render_widget(notif, notif_area);
            }
        }

        fn render_table(&mut self, f: &mut Frame, area: Rect) {
            let tradition = self.world.resource::<OralTradition>();

            let header_cells = ["Genre", "Historical Date", "Mutations", "Snippet"]
                .iter()
                .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));

            let header = Row::new(header_cells)
                .style(Style::default().bg(Color::DarkGray))
                .height(1)
                .bottom_margin(1);

            let rows = tradition.stories.iter().map(|story| {
                let genre_color = match story.genre {
                    StoryGenre::Heroic => Color::Yellow,
                    StoryGenre::Tragedy => Color::Red,
                    StoryGenre::Cautionary => Color::Magenta,
                    StoryGenre::Trivial => Color::Gray,
                };

                let snippet = if story.text.chars().count() > 30 {
                    let truncated: String = story.text.chars().take(27).collect();
                    format!("{}...", truncated)
                } else {
                    story.text.clone()
                };

                let cells = vec![
                    Cell::from(format!("{:?}", story.genre))
                        .style(Style::default().fg(genre_color)),
                    Cell::from(story.historical_date.to_string()),
                    Cell::from(story.mutations.to_string()),
                    Cell::from(snippet),
                ];

                Row::new(cells).height(1)
            });

            let table = Table::new(
                rows,
                [
                    Constraint::Length(12),
                    Constraint::Length(8),
                    Constraint::Length(10),
                    Constraint::Min(10),
                ],
            )
            .header(header)
            .block(Block::default().borders(Borders::ALL).title("Legends"))
            .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

            f.render_stateful_widget(table, area, &mut self.state);
        }

        fn render_details(&self, f: &mut Frame, area: Rect) {
            let block = Block::default().borders(Borders::ALL).title(Span::styled(
                " Legend Details ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ));

            let inner_area = block.inner(area);
            f.render_widget(block, area);

            let tradition = self.world.resource::<OralTradition>();

            if let Some(i) = self.state.selected() {
                if let Some(story) = tradition.stories.get(i) {
                    let effect_text = match story.genre {
                        StoryGenre::Heroic => "Boosts Leisure (Morale)",
                        StoryGenre::Tragedy => "Provides Catharsis (Small Leisure)",
                        StoryGenre::Cautionary => "Increases Wakefulness (Fear)",
                        StoryGenre::Trivial => "No significant effect",
                    };

                    let genre_color = match story.genre {
                        StoryGenre::Heroic => Color::Yellow,
                        StoryGenre::Tragedy => Color::Red,
                        StoryGenre::Cautionary => Color::Magenta,
                        StoryGenre::Trivial => Color::Gray,
                    };

                    // Split layout:
                    // 1. Metadata (Genre + Mutations Gauge)
                    // 2. Story Text
                    // 3. Effect
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Length(2), // Genre
                            Constraint::Length(2), // Mutations Gauge
                            Constraint::Length(1), // Spacer
                            Constraint::Min(4),    // Text
                            Constraint::Length(1), // Spacer
                            Constraint::Length(2), // Effect
                        ])
                        .split(inner_area);

                    // 1. Genre
                    let genre_line = Line::from(vec![
                        Span::styled("Genre: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::styled(
                            format!("{:?}", story.genre),
                            Style::default().fg(genre_color),
                        ),
                    ]);
                    f.render_widget(Paragraph::new(genre_line), chunks[0]);

                    // 2. Mutations Gauge
                    let mutation_pct = (story.mutations * 10).clamp(0, 100) as u16;
                    let mutation_color = if story.mutations > 5 {
                        Color::Red
                    } else if story.mutations > 2 {
                        Color::Yellow
                    } else {
                        Color::Green
                    };
                    let gauge = Gauge::default()
                        .block(Block::default())
                        .gauge_style(Style::default().fg(mutation_color))
                        .label(format!("Mutations: {}", story.mutations))
                        .percent(mutation_pct);
                    f.render_widget(gauge, chunks[1]);

                    // 3. Story Text
                    let text_block = Block::default().borders(Borders::TOP).title(Span::styled(
                        " Full Text ",
                        Style::default().fg(Color::DarkGray),
                    ));

                    // Parse the story text to highlight mutated parts
                    let mut spans = Vec::new();
                    let mutations = [
                        "Homeland",
                        "forged",
                        "returned to the void",
                        "The Cleansing Flame",
                        "The Breath of Giants",
                        "birthed from chaos",
                        " It is known.",
                        " So they say.",
                        " Or was it?",
                        " The spirits were watching.",
                        " And the colony survived.",
                        " Beware the void.",
                    ];

                    let mut current_text = story.text.clone();
                    while !current_text.is_empty() {
                        let mut first_match = None;
                        let mut first_idx = usize::MAX;

                        for m in &mutations {
                            if let Some(idx) = current_text.find(m) {
                                if idx < first_idx {
                                    first_idx = idx;
                                    first_match = Some(*m);
                                }
                            }
                        }

                        if let Some(m) = first_match {
                            if first_idx > 0 {
                                spans.push(Span::styled(
                                    current_text[..first_idx].to_string(),
                                    Style::default().fg(Color::Green),
                                ));
                            }
                            spans.push(Span::styled(
                                m.to_string(),
                                Style::default()
                                    .fg(Color::Magenta)
                                    .add_modifier(Modifier::BOLD),
                            ));
                            current_text = current_text[first_idx + m.len()..].to_string();
                        } else {
                            spans.push(Span::styled(
                                current_text,
                                Style::default().fg(Color::Green),
                            ));
                            break;
                        }
                    }

                    let p_text = Paragraph::new(Line::from(spans))
                        .block(text_block)
                        .wrap(Wrap { trim: true });
                    f.render_widget(p_text, chunks[3]);

                    // 4. Effect
                    let effect_line = Line::from(vec![
                        Span::styled(
                            "Effect: ",
                            Style::default().add_modifier(Modifier::UNDERLINED),
                        ),
                        Span::styled(effect_text, Style::default().fg(Color::Yellow)),
                    ]);
                    f.render_widget(Paragraph::new(effect_line), chunks[5]);
                } else {
                    f.render_widget(Paragraph::new("Select a legend..."), inner_area);
                }
            } else {
                f.render_widget(Paragraph::new("Select a legend..."), inner_area);
            }
        }

        fn render_footer(&self, f: &mut Frame, area: Rect) {
            let footer_text =
                "Controls: 'a' Add Event | 'q' Quit | ↑/↓ Select Story | Space: Force Tick";
            let p = Paragraph::new(footer_text)
                .style(Style::default().fg(Color::Gray))
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(p, area);
        }
    }
}

#[cfg(not(feature = "nova"))]
fn main() {
    println!("This example requires the 'nova' feature.");
    println!("Run with: cargo run --features nova --example oral_tradition_demo");
}

#[cfg(feature = "nova")]
fn main() -> anyhow::Result<()> {
    let mut app = app::App::new();
    app.run().map_err(|e| anyhow::anyhow!(e))
}
