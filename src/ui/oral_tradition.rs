#[cfg(feature = "nova")]
use bevy_ecs::prelude::*;
#[cfg(feature = "nova")]
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, Cell, Clear, Row, Table},
};

#[cfg(feature = "nova")]
use crate::layer1::oral_tradition::{OralTradition, StoryGenre};

#[cfg(feature = "nova")]
pub fn render_oral_tradition(frame: &mut Frame, area: Rect, world: &World) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let tradition = world.get_resource::<OralTradition>();

    let title = if let Some(trad) = tradition {
        format!(" Oral Tradition ({} Stories) ", trad.stories.len())
    } else {
        " Oral Tradition ".to_string()
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().bg(Color::Black));

    frame.render_widget(Clear, area);

    // Table Header
    let header = Row::new(vec!["Genre", "Date", "Mutations", "Story Text"])
        .style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Cyan),
        )
        .bottom_margin(1);

    let rows: Vec<Row> = if let Some(trad) = tradition {
        trad.stories
            .iter()
            .rev()
            .map(|story| {
                let (genre_color, genre_text) = match story.genre {
                    StoryGenre::Heroic => (Color::Yellow, "🌟 Heroic"),
                    StoryGenre::Tragedy => (Color::Red, "🎭 Tragedy"),
                    StoryGenre::Cautionary => (Color::Magenta, "⚠️ Cautionary"),
                    StoryGenre::Trivial => (Color::DarkGray, "📝 Trivial"),
                };

                Row::new(vec![
                    Cell::from(genre_text).style(Style::default().fg(genre_color)),
                    Cell::from(story.historical_date.to_string())
                        .style(Style::default().fg(Color::Cyan)),
                    Cell::from(story.mutations.to_string()).style(Style::default().fg(Color::Cyan)),
                    Cell::from(story.text.clone()).style(Style::default().fg(Color::White)),
                ])
            })
            .collect()
    } else {
        vec![]
    };

    // Column Widths
    let widths = [
        Constraint::Length(14), // Genre
        Constraint::Length(8),  // Date
        Constraint::Length(11), // Mutations
        Constraint::Min(20),    // Story Text
    ];

    let table = Table::new(rows, widths).header(header).block(block);

    frame.render_widget(table, area);
}
