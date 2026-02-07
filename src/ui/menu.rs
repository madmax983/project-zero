use crate::shared::menu::MenuState;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

/// Renders the main menu screen.
///
/// Displays the game title and a selectable list of options (Start Game, Quit).
pub fn render_main_menu(frame: &mut Frame, area: Rect, state: &MenuState) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Length(10), // Menu items
            Constraint::Min(10),
        ])
        .split(area);

    // Title
    let title = Paragraph::new("SCALE")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, layout[0]);

    // Options
    let constraints: Vec<Constraint> = state
        .options
        .iter()
        .map(|_| Constraint::Length(1))
        .collect();
    let menu_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(layout[1]);

    for (i, option) in state.options.iter().enumerate() {
        let style = if i == state.selected_index {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let text = if i == state.selected_index {
            format!("> {option}")
        } else {
            format!("  {option}")
        };

        let p = Paragraph::new(text)
            .style(style)
            .alignment(Alignment::Center);
        frame.render_widget(p, menu_layout[i]);
    }
}
