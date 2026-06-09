//! Demo of the Base Narrative Generator.
//!
//! This example demonstrates the `NarrativeGenerator` system (available by default),
//! which generates procedural text from templates (mad-libs style).
//!
//! # Distinction from "Nova" Feature
//!
//! This is **NOT** the "Oral Tradition" system (part of the `nova` feature).
//!
//! - **Narrative Generator (This Demo):** Generates static text strings from templates.
//!   Used for descriptions, flavor text, and history generation.
//!   Available in `scale::shared::narrative`.
//!
//! - **Oral Tradition (Nova Feature):** Simulates living legends that spread and evolve
//!   in taverns based on simulation events. Requires `cargo run --features nova`.
//!   See `examples/oral_tradition_demo.rs`.

use comfy_table::{presets::UTF8_FULL, Cell, Color as TableColor, Table};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};
use scale::prelude::*;
use std::io;
use std::time::Duration;

struct App {
    generator: NarrativeGenerator,
    template_ids: Vec<String>,
    state: ListState,
    generated_segments: Option<Vec<NarrativeSegment>>,
    error_message: Option<String>,
    context: NarrativeContext,
    should_quit: bool,
}

impl App {
    fn new() -> anyhow::Result<Self> {
        let mut generator = NarrativeGenerator::default();
        // Load lore from files
        println!("Loading lore...");
        if let Err(e) = generator.load_from_files("./lore") {
            eprintln!("Failed to load from ./lore: {}. Using embedded.", e);
            generator = NarrativeGenerator::from_embedded();
        }

        let template_ids: Vec<String> = generator.get_template_ids().into_iter().cloned().collect();

        let mut context = NarrativeContext::default();
        context.insert("CIV_NAME", "Terran Dominion");
        context.insert("ORIGIN_STAR", "Sol");
        context.insert("YEAR", "2150");
        context.insert("LEADER_NAME", "Valerian");

        // Common variables required by various other templates in the demo
        context.insert("NAME", "John Doe");
        context.insert("LOCK_STATUS", "locked");
        context.insert("COLONY", "Alpha Site");
        context.insert("SHIP_NAME", "The Wanderer");
        context.insert("RESOURCE", "Water");

        let mut state = ListState::default();
        if !template_ids.is_empty() {
            state.select(Some(0));
        }

        Ok(Self {
            generator,
            template_ids,
            state,
            generated_segments: None,
            error_message: None,
            context,
            should_quit: false,
        })
    }

    fn on_tick(&mut self) {}

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.template_ids.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.template_ids.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn generate(&mut self) {
        if let Some(i) = self.state.selected() {
            if let Some(id) = self.template_ids.get(i) {
                match self.generator.generate_structured(id, &self.context) {
                    Ok(segments) => {
                        self.generated_segments = Some(segments);
                        self.error_message = None;
                    }
                    Err(e) => {
                        self.generated_segments = None;
                        self.error_message = Some(format!("Error: {e}"));
                    }
                }
            }
        }
    }
}

fn main() -> anyhow::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let app_result = App::new();

    if let Ok(mut app) = app_result {
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
            let error_msg = format!("✗ {}", err);
            let mut table = Table::new();
            table
                .load_preset(UTF8_FULL)
                .apply_modifier(comfy_table::modifiers::UTF8_ROUND_CORNERS)
                .set_content_arrangement(comfy_table::ContentArrangement::Dynamic);
            table.add_row(vec![Cell::new(&error_msg).fg(TableColor::White)]);
            println!("{table}");
        }
    } else {
        // Restore terminal if app creation failed
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        if let Some(err) = app_result.err() {
            let error_msg = format!("✗ Failed to initialize app: {}", err);
            let mut table = Table::new();
            table
                .load_preset(UTF8_FULL)
                .apply_modifier(comfy_table::modifiers::UTF8_ROUND_CORNERS)
                .set_content_arrangement(comfy_table::ContentArrangement::Dynamic);
            table.add_row(vec![Cell::new(&error_msg).fg(TableColor::White)]);
            eprintln!("{table}");
        }
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    loop {
        terminal
            .draw(|f| ui(f, app))
            .map_err(|e| io::Error::other(e.to_string()))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            app.should_quit = true;
                        }
                        KeyCode::Down => app.next(),
                        KeyCode::Up => app.previous(),
                        KeyCode::Enter => app.generate(),
                        _ => {}
                    }
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
        app.on_tick();
    }
}

#[allow(clippy::too_many_lines)]
fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Footer
        ])
        .split(f.area());

    // Header
    let title = Paragraph::new("Mosaic 🎨: Narrative Generator Demo")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Footer
    let footer_text = "Use ↑/↓ to select template | ENTER to Generate | 'q' to Quit";
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[2]);

    // Main Content (Split Left/Right)
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30), // List
            Constraint::Percentage(70), // Details
        ])
        .split(chunks[1]);

    // Left: Template List
    let items: Vec<ListItem> = app
        .template_ids
        .iter()
        .map(|id| {
            ListItem::new(Line::from(vec![Span::raw(id)])).style(Style::default().fg(Color::Gray))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(Span::styled(
                    " Templates ",
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                )),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    f.render_stateful_widget(list, main_chunks[0], &mut app.state);

    // Right: Details & Output
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40), // Pattern Preview
            Constraint::Percentage(60), // Generated Output
        ])
        .split(main_chunks[1]);

    // Pattern Preview
    let selected_id = app.state.selected().and_then(|i| app.template_ids.get(i));
    let pattern_items = if let Some(id) = selected_id {
        if let Some(tmpl) = app.generator.get_template(id) {
            tmpl.patterns
                .iter()
                .map(|p| ListItem::new(Line::from(format!("• {}", p))))
                .collect::<Vec<ListItem>>()
        } else {
            vec![ListItem::new(Line::from("Template not found."))]
        }
    } else {
        vec![ListItem::new(Line::from("No template selected."))]
    };

    let pattern_list = List::new(pattern_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(Span::styled(
                    " Patterns Preview ",
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                )),
        )
        .style(Style::default().fg(Color::Gray));

    f.render_widget(pattern_list, right_chunks[0]);

    // Generated Output
    if let Some(err) = &app.error_message {
        let (err_type, action) = if err.contains("Missing required context") {
            ("Missing Context", "Check context.insert() logic.")
        } else if err.contains("has no options defined") {
            (
                "Empty Fragment",
                "Add options to the fragment in lore files.",
            )
        } else if err.contains("No lore files found") {
            ("Files Missing", "Check the directory for TEMPLATES.md.")
        } else if err.contains("Template not found") {
            (
                "Missing Template",
                "Verify template ID exists in TEMPLATES.md.",
            )
        } else {
            ("Unknown Error", "Review the error message.")
        };

        let rows = vec![
            ratatui::widgets::Row::new(vec![
                ratatui::widgets::Cell::from(Span::styled(
                    "Type",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )),
                ratatui::widgets::Cell::from(Span::styled(
                    err_type,
                    Style::default().fg(Color::Red),
                )),
            ]),
            ratatui::widgets::Row::new(vec![
                ratatui::widgets::Cell::from(Span::styled(
                    "Message",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )),
                ratatui::widgets::Cell::from(Span::styled(
                    err.clone(),
                    Style::default().fg(Color::White),
                )),
            ]),
            ratatui::widgets::Row::new(vec![
                ratatui::widgets::Cell::from(Span::styled(
                    "Action",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )),
                ratatui::widgets::Cell::from(Span::styled(
                    action,
                    Style::default().fg(Color::Cyan),
                )),
            ]),
        ];

        let table =
            ratatui::widgets::Table::new(rows, [Constraint::Length(10), Constraint::Min(40)])
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Thick)
                        .border_style(Style::default().fg(Color::Red))
                        .title(Span::styled(
                            " ✗ ERROR ",
                            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                        ))
                        .padding(Padding::uniform(1)),
                );
        f.render_widget(table, right_chunks[1]);
    } else if let Some(segments) = &app.generated_segments {
        let spans: Vec<Span> = segments
            .iter()
            .map(|seg| match seg {
                NarrativeSegment::Text(t) => Span::styled(t, Style::default().fg(Color::White)),
                NarrativeSegment::Slot { value, .. } => Span::styled(
                    value.clone(),
                    Style::default()
                        .fg(Color::Yellow)
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD)
                        .add_modifier(Modifier::ITALIC),
                ),
                NarrativeSegment::MissingContext(e) => Span::styled(
                    format!(" [MISSING CONTEXT: {}] ", e),
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::Red)
                        .add_modifier(Modifier::BOLD),
                ),
                NarrativeSegment::MissingFragmentOptions(e) => Span::styled(
                    format!(" [MISSING FRAGMENT OPTIONS: {}] ", e),
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::Red)
                        .add_modifier(Modifier::BOLD),
                ),
            })
            .collect();

        let line = Line::from(spans);
        f.render_widget(
            Paragraph::new(line)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(Color::Green))
                        .title(Span::styled(
                            " ✨ Generated Story ",
                            Style::default()
                                .fg(Color::Green)
                                .add_modifier(Modifier::BOLD),
                        ))
                        .padding(Padding::uniform(2)),
                )
                .wrap(Wrap { trim: true }),
            right_chunks[1],
        );
    } else {
        f.render_widget(
            Paragraph::new(
                "Press [ENTER] to generate a new story...
Use [UP] and [DOWN] to select a different template.",
            )
            .style(
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            )
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::DarkGray))
                    .padding(Padding::uniform(2)),
            ),
            right_chunks[1],
        );
    }
}
