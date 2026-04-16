use super::{render_with_frame, SharedWorld};
use ratatui::{buffer::Buffer, layout::Rect};
use ratatui_hypertile_extras::HypertilePlugin;
use ratatui::widgets::{Block, Borders, Paragraph};

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
                let text = if log.messages.is_empty() {
                    "(No messages)".to_string()
                } else {
                    log.messages.iter()
                        .map(|msg| format!("{} {}", match msg.color {
                            ratatui::style::Color::Red | ratatui::style::Color::LightRed => "[ERR]",
                            ratatui::style::Color::Yellow | ratatui::style::Color::LightYellow => "[WRN]",
                            ratatui::style::Color::Green | ratatui::style::Color::LightGreen => "[OK ]",
                            ratatui::style::Color::Cyan | ratatui::style::Color::LightCyan => "[INF]",
                            _ => "[LOG]",
                        }, msg.text))
                        .collect::<Vec<_>>()
                        .join("\n")
                };

                let paragraph = Paragraph::new(text)
                    .block(Block::default().title(" Message Log ").borders(Borders::ALL));

                frame.render_widget(paragraph, frame.area());
            } else {
                let paragraph = Paragraph::new("MessageLog not initialized.")
                    .block(Block::default().title(" Message Log ").borders(Borders::ALL));
                frame.render_widget(paragraph, frame.area());
            }
        });
    }
}
