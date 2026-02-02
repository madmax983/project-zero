use bevy_ecs::prelude::*;
use ratatui::style::Color;
use std::collections::VecDeque;

/// A single message in the log.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Message {
    /// The text content of the message.
    pub text: String,
    /// The color of the message.
    pub color: Color,
}

/// Resource to store the game message log.
#[derive(Resource)]
pub struct MessageLog {
    /// The buffer of messages.
    pub messages: VecDeque<Message>,
    /// Maximum number of messages to keep.
    pub max_size: usize,
}

impl Default for MessageLog {
    fn default() -> Self {
        Self {
            messages: VecDeque::new(),
            max_size: 50,
        }
    }
}

impl MessageLog {
    /// Creates a new message log with a specific size limit.
    #[must_use]
    pub const fn new(max_size: usize) -> Self {
        Self {
            messages: VecDeque::new(),
            max_size,
        }
    }

    /// Adds a message to the log with default white color.
    pub fn add(&mut self, text: impl Into<String>) {
        self.add_colored(text, Color::White);
    }

    /// Adds a colored message to the log.
    pub fn add_colored(&mut self, text: impl Into<String>, color: Color) {
        let message = Message {
            text: text.into(),
            color,
        };

        self.messages.push_back(message);
        if self.messages.len() > self.max_size {
            self.messages.pop_front();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_log_default() {
        let log = MessageLog::default();
        assert!(log.messages.is_empty());
        assert_eq!(log.max_size, 50);
    }

    #[test]
    fn test_add_message() {
        let mut log = MessageLog::new(5);
        log.add("Hello");
        assert_eq!(log.messages.len(), 1);
        assert_eq!(log.messages[0].text, "Hello");
        assert_eq!(log.messages[0].color, Color::White);
    }

    #[test]
    fn test_add_colored_message() {
        let mut log = MessageLog::new(5);
        log.add_colored("Warning", Color::Red);
        assert_eq!(log.messages[0].text, "Warning");
        assert_eq!(log.messages[0].color, Color::Red);
    }

    #[test]
    fn test_log_overflow() {
        let mut log = MessageLog::new(2);
        log.add("1");
        log.add("2");
        log.add("3");

        assert_eq!(log.messages.len(), 2);
        assert_eq!(log.messages[0].text, "2");
        assert_eq!(log.messages[1].text, "3");
    }
}
