//! Platform-agnostic input types.
//!
//! These types decouple the game's input handling from any specific backend
//! (crossterm for native, ratzilla for WASM). Each platform adapter implements
//! `From` conversions to translate platform-specific events into these types.

/// Platform-agnostic key codes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GameKeyCode {
    /// A character key.
    Char(char),
    /// Enter / Return.
    Enter,
    /// Escape.
    Esc,
    /// Tab.
    Tab,
    /// Arrow up.
    Up,
    /// Arrow down.
    Down,
    /// Arrow left.
    Left,
    /// Arrow right.
    Right,
    /// Backspace.
    Backspace,
    /// Delete.
    Delete,
    /// Shift+Tab.
    BackTab,
}

/// Platform-agnostic key modifiers.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct GameKeyModifiers {
    /// Control key.
    pub ctrl: bool,
    /// Alt/Option key.
    pub alt: bool,
    /// Shift key.
    pub shift: bool,
}

impl GameKeyModifiers {
    /// Create modifiers from individual flags.
    #[must_use]
    pub const fn new(ctrl: bool, alt: bool, shift: bool) -> Self {
        Self { ctrl, alt, shift }
    }
}

/// A platform-agnostic keyboard event.
///
/// Only represents key-press events; repeat/release are filtered at the
/// platform translation boundary.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct GameKeyEvent {
    /// The key that was pressed.
    pub code: GameKeyCode,
    /// Modifier state active during the key event.
    pub modifiers: GameKeyModifiers,
}

impl GameKeyEvent {
    /// Create a new key event.
    #[must_use]
    pub const fn new(code: GameKeyCode) -> Self {
        Self {
            code,
            modifiers: GameKeyModifiers::new(false, false, false),
        }
    }

    /// Attach explicit modifier state.
    #[must_use]
    pub const fn with_modifiers(mut self, modifiers: GameKeyModifiers) -> Self {
        self.modifiers = modifiers;
        self
    }

    /// Mark the event as control-modified.
    #[must_use]
    pub const fn with_ctrl(mut self) -> Self {
        self.modifiers.ctrl = true;
        self
    }

    /// Mark the event as alt-modified.
    #[must_use]
    pub const fn with_alt(mut self) -> Self {
        self.modifiers.alt = true;
        self
    }

    /// Mark the event as shift-modified.
    #[must_use]
    pub const fn with_shift(mut self) -> Self {
        self.modifiers.shift = true;
        self
    }
}

/// A platform-agnostic mouse click event.
///
/// Only represents left-button click-down events, which is all the game uses.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct GameMouseEvent {
    /// Screen column (x coordinate).
    pub x: u16,
    /// Screen row (y coordinate).
    pub y: u16,
}

impl GameMouseEvent {
    /// Create a new mouse event.
    #[must_use]
    pub const fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::shell::ShellConfig;

    #[test]
    fn test_game_key_event_creation() {
        let event = GameKeyEvent::new(GameKeyCode::Char('q'));
        assert_eq!(event.code, GameKeyCode::Char('q'));
        assert_eq!(event.modifiers, GameKeyModifiers::default());
    }

    #[test]
    fn test_game_key_event_with_ctrl() {
        let event = GameKeyEvent::new(GameKeyCode::Char('k')).with_ctrl();
        assert!(event.modifiers.ctrl);
        assert!(!event.modifiers.alt);
        assert!(!event.modifiers.shift);
    }

    #[test]
    fn test_shell_config_round_trips_default_workspace() {
        let cfg = ShellConfig::default();
        let json = serde_json::to_string(&cfg).unwrap();
        let decoded: ShellConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.startup_workspace, "Colony Ops");
    }

    #[test]
    fn test_game_key_code_equality() {
        assert_eq!(GameKeyCode::Char('a'), GameKeyCode::Char('a'));
        assert_ne!(GameKeyCode::Char('a'), GameKeyCode::Char('b'));
        assert_ne!(GameKeyCode::Enter, GameKeyCode::Esc);
    }

    #[test]
    fn test_game_key_event_is_copy() {
        let event = GameKeyEvent::new(GameKeyCode::Enter);
        let copy = event;
        assert_eq!(event, copy);
    }

    #[test]
    fn test_game_mouse_event_creation() {
        let event = GameMouseEvent::new(10, 20);
        assert_eq!(event.x, 10);
        assert_eq!(event.y, 20);
    }

    #[test]
    fn test_game_mouse_event_is_copy() {
        let event = GameMouseEvent::new(5, 5);
        let copy = event;
        assert_eq!(event, copy);
    }

    #[test]
    fn test_all_key_code_variants() {
        // Ensure all variants are constructible and distinct
        let codes = [
            GameKeyCode::Char('x'),
            GameKeyCode::Enter,
            GameKeyCode::Esc,
            GameKeyCode::Tab,
            GameKeyCode::Up,
            GameKeyCode::Down,
            GameKeyCode::Left,
            GameKeyCode::Right,
            GameKeyCode::Backspace,
            GameKeyCode::Delete,
            GameKeyCode::BackTab,
        ];
        // Each variant should be distinct from the others
        for (i, a) in codes.iter().enumerate() {
            for (j, b) in codes.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b, "Variants at {i} and {j} should differ");
                }
            }
        }
    }
}
