use super::{render_with_frame, SharedWorld};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::{buffer::Buffer, layout::Rect};
use ratatui_hypertile_extras::HypertilePlugin;

use crate::shared::log::MessageLog;

pub struct LogPlugin {
    world: SharedWorld,
}

impl LogPlugin {
    pub const fn new(world: SharedWorld) -> Self {
        Self { world }
    }
}

impl HypertilePlugin for LogPlugin {
    fn render(&self, area: Rect, buf: &mut Buffer, _is_focused: bool) {
        let world = self.world.borrow();
        render_with_frame(area, buf, |frame| {
            if let Some(log) = world.get_resource::<MessageLog>() {
                let paragraph = if log.messages.is_empty() {
                    Paragraph::new("(No messages)").block(
                        Block::default()
                            .title(" Message Log ")
                            .borders(Borders::ALL),
                    )
                } else {
                    use ratatui::style::{Modifier, Style};
                    use ratatui::text::{Line, Span};

                    let lines: Vec<Line> = log
                        .messages
                        .iter()
                        .map(|msg| {
                            let (prefix, prefix_style) = match msg.color {
                                ratatui::style::Color::Red | ratatui::style::Color::LightRed => (
                                    "[ERR] ",
                                    Style::default().fg(msg.color).add_modifier(Modifier::BOLD),
                                ),
                                ratatui::style::Color::Yellow
                                | ratatui::style::Color::LightYellow => (
                                    "[WRN] ",
                                    Style::default().fg(msg.color).add_modifier(Modifier::BOLD),
                                ),
                                ratatui::style::Color::Green
                                | ratatui::style::Color::LightGreen => (
                                    "[OK ] ",
                                    Style::default().fg(msg.color).add_modifier(Modifier::BOLD),
                                ),
                                ratatui::style::Color::Cyan | ratatui::style::Color::LightCyan => (
                                    "[INF] ",
                                    Style::default().fg(msg.color).add_modifier(Modifier::BOLD),
                                ),
                                _ => ("[LOG] ", Style::default().fg(msg.color)),
                            };

                            Line::from(vec![
                                Span::styled(prefix, prefix_style),
                                Span::styled(msg.text.clone(), Style::default().fg(msg.color)),
                            ])
                        })
                        .collect();

                    Paragraph::new(lines).block(
                        Block::default()
                            .title(" Message Log ")
                            .borders(Borders::ALL),
                    )
                };

                frame.render_widget(paragraph, frame.area());
            } else {
                let paragraph = Paragraph::new("MessageLog not initialized.").block(
                    Block::default()
                        .title(" Message Log ")
                        .borders(Borders::ALL),
                );
                frame.render_widget(paragraph, frame.area());
            }
        });
    }
}
