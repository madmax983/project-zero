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

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::style::Color;
    use ratatui_hypertile_extras::HypertilePlugin;
    use std::cell::RefCell;
    use std::rc::Rc;

    fn setup_world() -> Rc<RefCell<World>> {
        let mut world = World::new();
        world.insert_resource(MessageLog::new(10));
        Rc::new(RefCell::new(world))
    }

    fn buffer_text(buffer: &Buffer) -> String {
        buffer
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>()
    }

    #[test]
    fn test_log_plugin_empty() {
        let world = setup_world();
        let plugin = LogPlugin::new(Rc::clone(&world));
        let area = Rect::new(0, 0, 40, 10);
        let mut buffer = Buffer::empty(area);

        plugin.render(area, &mut buffer, false);

        let text = buffer_text(&buffer);
        assert!(text.contains("Message Log"));
        assert!(text.contains("(No messages)"));
    }

    #[test]
    fn test_log_plugin_with_messages() {
        let world = setup_world();
        {
            let mut w = world.borrow_mut();
            let mut log = w.resource_mut::<MessageLog>();
            log.add_colored("Error Message", Color::Red);
            log.add_colored("Warning Message", Color::Yellow);
            log.add_colored("Info Message", Color::Cyan);
            log.add_colored("Success Message", Color::Green);
            log.add_colored("Normal Message", Color::White);
        }

        let plugin = LogPlugin::new(Rc::clone(&world));
        let area = Rect::new(0, 0, 40, 10);
        let mut buffer = Buffer::empty(area);

        plugin.render(area, &mut buffer, false);

        let text = buffer_text(&buffer);
        assert!(text.contains("Message Log"));
        assert!(text.contains("[ERR] Error Message"));
        assert!(text.contains("[WRN] Warning Message"));
        assert!(text.contains("[INF] Info Message"));
        assert!(text.contains("[OK ] Success Message"));
        assert!(text.contains("[LOG] Normal Message"));
    }

    #[test]
    fn test_log_plugin_uninitialized() {
        let world = Rc::new(RefCell::new(World::new()));
        let plugin = LogPlugin::new(Rc::clone(&world));
        let area = Rect::new(0, 0, 40, 10);
        let mut buffer = Buffer::empty(area);

        plugin.render(area, &mut buffer, false);

        let text = buffer_text(&buffer);
        assert!(text.contains("Message Log"));
        assert!(text.contains("MessageLog not initialized."));
    }
}
