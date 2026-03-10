use bevy_ecs::prelude::*;
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, List, ListItem},
};

use crate::layer1::notifications::{NotificationQueue, NotificationSeverity};
use crate::shared::time::SimulationTime;

/// Renders the notifications overlay in the top-right corner of the given area.
///
/// Shows up to 5 most recent notifications, colored by severity.
/// If there are no active notifications, nothing is rendered.
pub fn render_notifications(frame: &mut Frame, area: Rect, world: &World) {
    let Some(queue) = world.get_resource::<NotificationQueue>() else {
        return;
    };

    if queue.active.is_empty() {
        return;
    }

    let width = area.width.min(35);
    let display_count = queue.active.len().min(5);
    #[allow(clippy::cast_possible_truncation)] // display_count is capped at 5
    let height = (display_count as u16 + 2).min(8);

    // Position overlay at top-right of area
    let overlay = Rect {
        x: area.x + area.width.saturating_sub(width),
        y: area.y,
        width,
        height,
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" Notifications ");
    let inner = block.inner(overlay);
    frame.render_widget(block, overlay);

    let max_text_width = inner.width as usize;

    let current_tick = world.get_resource::<SimulationTime>().map_or(0, |t| t.tick);

    // Take the last `display_count` notifications, show newest first
    let items: Vec<ListItem> = queue
        .active
        .iter()
        .rev()
        .take(display_count)
        .map(|n| {
            let age = current_tick.saturating_sub(n.created_at);
            let time_left = n
                .expires_at
                .unwrap_or(u64::MAX)
                .saturating_sub(current_tick);

            // Fade Out Logic (Ludwig: "Ease-Out")
            // If expiring in < 20 ticks (2s), dim the text.
            let color = if time_left < 20 {
                Color::DarkGray
            } else {
                match n.severity {
                    NotificationSeverity::Info => Color::Cyan,
                    NotificationSeverity::Success => Color::Green,
                    NotificationSeverity::Warning => Color::Yellow,
                    NotificationSeverity::Error => Color::Red,
                }
            };

            let display_text = if n.text.len() > max_text_width && max_text_width > 1 {
                format!(
                    "{}\u{2026}",
                    &n.text.chars().take(max_text_width - 1).collect::<String>()
                )
            } else {
                n.text.clone()
            };

            // Slide In Logic (Ludwig: "Juice")
            // Changed from linear to an "Ease-Out" curve for smoother motion.
            // UI elements should "slide" in and settle naturally, not teleport rigidly.
            let padding = if age < 10 {
                // t goes from 0.0 to 1.0 over 10 ticks
                let t = age as f32 / 10.0;
                // Ease-Out Cubic: f(t) = 1 - (1 - t)^3
                let ease_out = 1.0 - (1.0 - t).powi(3);
                // Start with 15 chars of padding, ease down to 0
                #[allow(clippy::cast_possible_truncation)]
                let p = (15.0 * (1.0 - ease_out)).round() as usize;
                p
            } else {
                0
            };

            let padded_text = format!("{:width$}{}", "", display_text, width = padding);

            ListItem::new(Line::styled(padded_text, Style::default().fg(color)))
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, inner);
}
