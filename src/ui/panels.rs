use bevy_ecs::prelude::*;
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, List, ListItem},
};

use crate::shared::log::MessageLog;
use crate::ui::inspector::render_inspector;

/// Renders the right-hand information panel.
///
/// This panel is split into:
/// 1. **Inspector**: Context-sensitive details about the selected tile/entity.
/// 2. **Message Log**: Recent game events (bottom).
pub fn render_info_panel(frame: &mut Frame, area: Rect, world: &World) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),    // Stats
            Constraint::Length(10), // Log
        ])
        .split(area);

    let stats_area = chunks[0];
    let log_area = chunks[1];

    // Stats Panel
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" Info ");
    let inner = block.inner(stats_area);
    frame.render_widget(block, stats_area);

    // Delegate internal rendering to inspector
    render_inspector(frame, inner, world);

    // Message Log Panel
    let log_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" Log ");
    let log_inner = log_block.inner(log_area);
    frame.render_widget(log_block, log_area);

    if let Some(log) = world.get_resource::<MessageLog>() {
        let height = log_inner.height as usize;
        let start = log.messages.len().saturating_sub(height);
        let max_w = log_inner.width as usize;
        let items: Vec<ListItem> = log
            .messages
            .iter()
            .skip(start)
            .map(|m| {
                let display_text = if m.text.len() > max_w && max_w > 1 {
                    let idx = m
                        .text
                        .char_indices()
                        .nth(max_w - 1)
                        .map(|(i, _)| i)
                        .unwrap_or(m.text.len());
                    format!("{}\u{2026}", &m.text[..idx])
                } else {
                    m.text.clone()
                };
                ListItem::new(Line::styled(display_text, Style::default().fg(m.color)))
            })
            .collect();

        let list = List::new(items);
        frame.render_widget(list, log_inner);
    }
}
