//! Native platform adapter (crossterm → `GameKeyEvent` / `GameMouseEvent`).

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, MouseEvent, MouseEventKind};

use super::input::{GameKeyCode, GameKeyEvent, GameMouseEvent};

/// Convert a crossterm `KeyEvent` to a platform-agnostic `GameKeyEvent`.
///
/// Only `KeyEventKind::Press` events are translated. On Windows, crossterm
/// also sends `Release` and `Repeat` events which would double-fire inputs.
impl TryFrom<KeyEvent> for GameKeyEvent {
    type Error = ();

    fn try_from(key: KeyEvent) -> Result<Self, Self::Error> {
        if key.kind != KeyEventKind::Press {
            return Err(());
        }
        let code = match key.code {
            KeyCode::Char(c) => GameKeyCode::Char(c),
            KeyCode::Enter => GameKeyCode::Enter,
            KeyCode::Esc => GameKeyCode::Esc,
            KeyCode::Tab => GameKeyCode::Tab,
            KeyCode::Up => GameKeyCode::Up,
            KeyCode::Down => GameKeyCode::Down,
            KeyCode::Left => GameKeyCode::Left,
            KeyCode::Right => GameKeyCode::Right,
            KeyCode::Backspace => GameKeyCode::Backspace,
            KeyCode::Delete => GameKeyCode::Delete,
            _ => return Err(()),
        };
        Ok(Self { code })
    }
}

/// Convert a crossterm `MouseEvent` to a platform-agnostic `GameMouseEvent`.
///
/// Only left-button mouse-down events are translated; all others return `None`.
impl TryFrom<MouseEvent> for GameMouseEvent {
    type Error = ();

    fn try_from(mouse: MouseEvent) -> Result<Self, Self::Error> {
        if let MouseEventKind::Down(_button) = mouse.kind {
            Ok(Self {
                x: mouse.column,
                y: mouse.row,
            })
        } else {
            Err(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEventKind, KeyModifiers, MouseButton};

    fn crossterm_key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    #[test]
    fn test_translate_char_key() {
        let key = crossterm_key(KeyCode::Char('q'));
        let game_key = GameKeyEvent::try_from(key).unwrap();
        assert_eq!(game_key.code, GameKeyCode::Char('q'));
    }

    #[test]
    fn test_translate_enter() {
        let key = crossterm_key(KeyCode::Enter);
        let game_key = GameKeyEvent::try_from(key).unwrap();
        assert_eq!(game_key.code, GameKeyCode::Enter);
    }

    #[test]
    fn test_translate_esc() {
        let key = crossterm_key(KeyCode::Esc);
        let game_key = GameKeyEvent::try_from(key).unwrap();
        assert_eq!(game_key.code, GameKeyCode::Esc);
    }

    #[test]
    fn test_translate_arrows() {
        assert_eq!(
            GameKeyEvent::try_from(crossterm_key(KeyCode::Up))
                .unwrap()
                .code,
            GameKeyCode::Up
        );
        assert_eq!(
            GameKeyEvent::try_from(crossterm_key(KeyCode::Down))
                .unwrap()
                .code,
            GameKeyCode::Down
        );
        assert_eq!(
            GameKeyEvent::try_from(crossterm_key(KeyCode::Left))
                .unwrap()
                .code,
            GameKeyCode::Left
        );
        assert_eq!(
            GameKeyEvent::try_from(crossterm_key(KeyCode::Right))
                .unwrap()
                .code,
            GameKeyCode::Right
        );
    }

    #[test]
    fn test_translate_tab() {
        let game_key = GameKeyEvent::try_from(crossterm_key(KeyCode::Tab)).unwrap();
        assert_eq!(game_key.code, GameKeyCode::Tab);
    }

    #[test]
    fn test_translate_backspace_delete() {
        let bs = GameKeyEvent::try_from(crossterm_key(KeyCode::Backspace)).unwrap();
        assert_eq!(bs.code, GameKeyCode::Backspace);

        let del = GameKeyEvent::try_from(crossterm_key(KeyCode::Delete)).unwrap();
        assert_eq!(del.code, GameKeyCode::Delete);
    }

    #[test]
    fn test_unhandled_key_returns_none() {
        let result = GameKeyEvent::try_from(crossterm_key(KeyCode::F(1)));
        assert!(result.is_err());
    }

    #[test]
    fn test_key_release_ignored() {
        let key = KeyEvent::new_with_kind(
            KeyCode::Char(' '),
            KeyModifiers::empty(),
            KeyEventKind::Release,
        );
        assert!(GameKeyEvent::try_from(key).is_err());
    }

    #[test]
    fn test_key_repeat_ignored() {
        let key = KeyEvent::new_with_kind(
            KeyCode::Char(' '),
            KeyModifiers::empty(),
            KeyEventKind::Repeat,
        );
        assert!(GameKeyEvent::try_from(key).is_err());
    }

    #[test]
    fn test_translate_mouse_down() {
        let mouse = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 10,
            row: 20,
            modifiers: KeyModifiers::empty(),
        };
        let game_mouse = GameMouseEvent::try_from(mouse).unwrap();
        assert_eq!(game_mouse.x, 10);
        assert_eq!(game_mouse.y, 20);
    }

    #[test]
    fn test_translate_mouse_move_ignored() {
        let mouse = MouseEvent {
            kind: MouseEventKind::Moved,
            column: 10,
            row: 20,
            modifiers: KeyModifiers::empty(),
        };
        assert!(GameMouseEvent::try_from(mouse).is_err());
    }

    #[test]
    fn test_translate_mouse_up_ignored() {
        let mouse = MouseEvent {
            kind: MouseEventKind::Up(MouseButton::Left),
            column: 10,
            row: 20,
            modifiers: KeyModifiers::empty(),
        };
        assert!(GameMouseEvent::try_from(mouse).is_err());
    }
}
