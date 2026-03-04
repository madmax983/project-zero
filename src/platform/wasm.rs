//! WASM platform adapter (ratzilla → `GameKeyEvent` / `GameMouseEvent`).

#[cfg(target_arch = "wasm32")]
use ratzilla::event::{KeyCode, KeyEvent, MouseEvent, MouseEventKind};

#[cfg(not(target_arch = "wasm32"))]
struct DummyEvent;
#[cfg(not(target_arch = "wasm32"))]
struct KeyEvent;
#[cfg(not(target_arch = "wasm32"))]
struct MouseEvent;

use super::input::{GameKeyCode, GameKeyEvent, GameMouseEvent};

/// Convert a ratzilla `KeyEvent` to a platform-agnostic `GameKeyEvent`.
///
/// Returns `Err(())` for keys we don't handle.
#[cfg(target_arch = "wasm32")]
impl TryFrom<KeyEvent> for GameKeyEvent {
    type Error = ();

    fn try_from(key: KeyEvent) -> Result<Self, Self::Error> {
        let code = match key.code {
            KeyCode::Char(c) => GameKeyCode::Char(c),
            KeyCode::Enter => GameKeyCode::Enter,
            KeyCode::Esc => GameKeyCode::Esc,
            KeyCode::Tab => GameKeyCode::Tab,
            // KeyCode::BackTab => GameKeyCode::BackTab, // Not supported in ratzilla 0.3
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

#[cfg(not(target_arch = "wasm32"))]
impl TryFrom<KeyEvent> for GameKeyEvent {
    type Error = ();

    fn try_from(_key: KeyEvent) -> Result<Self, Self::Error> {
        Err(())
    }
}

/// Convert a ratzilla `MouseEvent` to a platform-agnostic `GameMouseEvent`.
///
/// Only `Pressed` events are translated. Ratzilla uses `u32` for coordinates;
/// we saturate-cast to `u16` (terminal grids don't exceed 65535 columns).
#[cfg(target_arch = "wasm32")]
impl TryFrom<MouseEvent> for GameMouseEvent {
    type Error = ();

    #[allow(clippy::cast_possible_truncation)]
    fn try_from(mouse: MouseEvent) -> Result<Self, Self::Error> {
        if mouse.event == MouseEventKind::Pressed {
            Ok(Self {
                x: mouse.x.min(u32::from(u16::MAX)) as u16,
                y: mouse.y.min(u32::from(u16::MAX)) as u16,
            })
        } else {
            Err(())
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl TryFrom<MouseEvent> for GameMouseEvent {
    type Error = ();

    fn try_from(_mouse: MouseEvent) -> Result<Self, Self::Error> {
        Err(())
    }
}
