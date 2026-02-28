use bevy_ecs::prelude::*;
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, Clear, Row, Table},
};

use crate::layer1::{format_event_prefix, Chronicle, ChronicleUiState, EventImportance};

/// Renders the chronicle overlay if active.
///
/// This displays a scrolling history of colony events (births, deaths, construction, etc.).
/// It is toggled by the [`ChronicleUiState`] resource.
pub fn render_chronicle(frame: &mut Frame, area: Rect, world: &World) {
    let ui_state = world.resource::<ChronicleUiState>();
    if !ui_state.is_open {
        return;
    }

    let chronicle = world.resource::<Chronicle>();

    let block = Block::default()
        .title(" Chronicle (Press L/H to close) ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().bg(Color::Black));

    let popup_area = centered_rect(60, 60, area);
    frame.render_widget(Clear, popup_area); // Clear background

    // Table Header
    let header = Row::new(vec!["Time", "Imp", "Event"])
        .style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Cyan),
        )
        .bottom_margin(1);

    // Table Rows
    let rows: Vec<Row> = chronicle
        .events
        .iter()
        .rev() // Newest first
        .map(|evt| {
            let color = get_importance_color(evt.importance);
            let prefix = format_event_prefix(evt.importance);

            Row::new(vec![
                if evt.year == 0 {
                    format!("Pre [{}]", evt.tick)
                } else {
                    format!("Y{} [{}]", evt.year, evt.tick)
                },
                prefix.to_string(),
                evt.text.clone(),
            ])
            .style(Style::default().fg(color))
        })
        .collect();

    // Column Widths
    let widths = [
        Constraint::Length(12), // Time
        Constraint::Length(4),  // Type
        Constraint::Min(20),    // Event
    ];

    let table = Table::new(rows, widths).header(header).block(block);

    frame.render_widget(table, popup_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Returns the display color for an event based on its importance.
///
/// # Examples
///
/// ```
/// use scale::ui::chronicle::get_importance_color;
/// use scale::layer1::EventImportance;
/// use ratatui::style::Color;
///
/// assert_eq!(get_importance_color(EventImportance::Legendary), Color::Yellow);
/// assert_eq!(get_importance_color(EventImportance::Minor), Color::DarkGray);
/// ```
#[must_use]
pub const fn get_importance_color(importance: EventImportance) -> Color {
    match importance {
        EventImportance::Legendary => Color::Yellow,
        EventImportance::Major => Color::Magenta,
        EventImportance::Standard => Color::White,
        EventImportance::Minor => Color::DarkGray,
    }
}
