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
        state: ListState,
        should_quit: bool,
        last_tick: std::time::Instant,
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

            let mut state = ListState::default();
            state.select(Some(0));

            Self {
                world,
                state,
                should_quit: false,
                last_tick: std::time::Instant::now(),
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
                (
                    "A mysterious stranger arrived.",
                    EventImportance::Standard,
                ),
                (
                    "The reactor core stabilized.",
                    EventImportance::Standard,
                ),
                ("A ghostly figure was seen.", EventImportance::Minor),
                ("The ancient prophecy was fulfilled.", EventImportance::Legendary),
            ];

            let (text, importance) = events[rng.gen_range(0..events.len())];
            self.world
                .resource_mut::<Chronicle>()
                .add_event(tick, text.to_string(), importance);
        }

        fn next(&mut self) {
            let count = self.world.resource::<OralTradition>().stories.len();
            if count == 0 {
                return;
            }
            let i = self.state.selected().map_or(0, |i| {
                if i >= count - 1 {
                    0
                } else {
                    i + 1
                }
            });
            self.state.select(Some(i));
        }

        fn previous(&mut self) {
            let count = self.world.resource::<OralTradition>().stories.len();
            if count == 0 {
                return;
            }
            let i = self.state.selected().map_or(0, |i| {
                if i == 0 {
                    count - 1
                } else {
                    i - 1
                }
            });
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

            // Header
            let time = self.world.resource::<SimulationTime>();
            let header = Paragraph::new(format!(
                "Oral Tradition Explorer | Tick: {} | Speed: Auto",
                time.tick
            ))
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .block(Block::default().borders(Borders::ALL));
            f.render_widget(header, chunks[0]);

            // Footer
            let footer = Paragraph::new(
                "Controls: 'a' Add Event | 'q' Quit | ↑/↓ Select Story | Space: Force Tick",
            )
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::ALL));
            f.render_widget(footer, chunks[2]);

            // Main Content
            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(50), // Stories List
                    Constraint::Percentage(50), // Details
                ])
                .split(chunks[1]);

            // Left: Stories List
            let tradition = self.world.resource::<OralTradition>();
            let items: Vec<ListItem> = tradition
                .stories
                .iter()
                .map(|story| {
                    let style = match story.genre {
                        StoryGenre::Heroic => Style::default().fg(Color::Yellow),
                        StoryGenre::Tragedy => Style::default().fg(Color::Red),
                        StoryGenre::Cautionary => Style::default().fg(Color::Magenta),
                        StoryGenre::Trivial => Style::default().fg(Color::Gray),
                    };
                    ListItem::new(Line::from(vec![
                        Span::styled(format!("[{:?}] ", story.genre), style),
                        Span::raw(&story.text),
                    ]))
                })
                .collect();

            let list = List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Active Legends (Oral Tradition)"),
                )
                .highlight_style(
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                );
            f.render_stateful_widget(list, main_chunks[0], &mut self.state);

            // Right: Details
            let details_block = Block::default().borders(Borders::ALL).title("Legend Details");

            if let Some(i) = self.state.selected() {
                if let Some(story) = tradition.stories.get(i) {
                    let details_text = vec![
                        Line::from(vec![
                            Span::styled("Genre: ", Style::default().fg(Color::Cyan)),
                            Span::raw(format!("{:?}", story.genre)),
                        ]),
                        Line::from(vec![
                            Span::styled("Mutations: ", Style::default().fg(Color::Cyan)),
                            Span::raw(format!("{}", story.mutations)),
                        ]),
                        Line::from(vec![
                            Span::styled("Origin Tick: ", Style::default().fg(Color::Cyan)),
                            Span::raw(format!("{}", story.origin_tick)),
                        ]),
                        Line::from(""),
                        Line::from(Span::styled(
                            "Current Text:",
                            Style::default().add_modifier(Modifier::UNDERLINED),
                        )),
                        Line::from(Span::styled(
                            &story.text,
                            Style::default().fg(Color::Green),
                        )),
                        Line::from(""),
                        Line::from(Span::styled(
                            "Effect:",
                            Style::default().add_modifier(Modifier::UNDERLINED),
                        )),
                        Line::from(match story.genre {
                            StoryGenre::Heroic => "Boosts Leisure (Morale)",
                            StoryGenre::Tragedy => "Provides Catharsis (Small Leisure)",
                            StoryGenre::Cautionary => "Increases Wakefulness (Fear)",
                            StoryGenre::Trivial => "No significant effect",
                        }),
                    ];

                    let p = Paragraph::new(details_text).block(details_block).wrap(Wrap { trim: true });
                    f.render_widget(p, main_chunks[1]);
                } else {
                     f.render_widget(Paragraph::new("Select a story...").block(details_block), main_chunks[1]);
                }
            } else {
                f.render_widget(Paragraph::new("Select a story...").block(details_block), main_chunks[1]);
            }
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
