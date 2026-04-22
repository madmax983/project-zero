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
                let text = if log.messages.is_empty() {
                    "(No messages)".to_string()
                } else {
                    // ⚡ Bolt Optimization:
                    // Pre-allocate a single String with a reasonable capacity.
                    // This avoids `log.messages.len()` intermediate String allocations from `format!`,
                    // as well as the intermediate `Vec` allocation from `.collect::<Vec<_>>()`.
                    // Each message is roughly 50 bytes.
                    use std::fmt::Write;
                    let mut buf = String::with_capacity(log.messages.len() * 50);
                    for (i, msg) in log.messages.iter().enumerate() {
                        if i > 0 {
                            buf.push('\n');
                        }
                        let prefix = match msg.color {
                            ratatui::style::Color::Red | ratatui::style::Color::LightRed => "[ERR]",
                            ratatui::style::Color::Yellow | ratatui::style::Color::LightYellow => {
                                "[WRN]"
                            }
                            ratatui::style::Color::Green | ratatui::style::Color::LightGreen => {
                                "[OK ]"
                            }
                            ratatui::style::Color::Cyan | ratatui::style::Color::LightCyan => {
                                "[INF]"
                            }
                            _ => "[LOG]",
                        };
                        let _ = write!(buf, "{} {}", prefix, msg.text);
                    }
                    buf
                };

                let paragraph = Paragraph::new(text).block(
                    Block::default()
                        .title(" Message Log ")
                        .borders(Borders::ALL),
                );

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
