use bevy_ecs::prelude::*;
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, Clear, Row, Table},
};

use crate::layer1::{format_event_prefix, Chronicle, EventImportance};

/// Renders the chronicle pane into the provided area.
pub fn render_chronicle(frame: &mut Frame, area: Rect, world: &World) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let chronicle = world.resource::<Chronicle>();

    let block = Block::default()
        .title(" Chronicle ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().bg(Color::Black));

    frame.render_widget(Clear, area);

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

    frame.render_widget(table, area);
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
