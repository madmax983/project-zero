use bevy_ecs::prelude::*;

/// Resources for the Main Menu.
#[derive(Resource)]
pub struct MenuState {
    /// The index of the currently selected option.
    pub selected_index: usize,
    /// The list of menu options.
    pub options: Vec<String>,
}

impl Default for MenuState {
    fn default() -> Self {
        Self {
            selected_index: 0,
            options: vec!["Start Game".to_string(), "Quit".to_string()],
        }
    }
}

impl MenuState {
    /// Select the next option.
    #[allow(clippy::missing_const_for_fn)]
    pub fn next(&mut self) {
        if self.selected_index < self.options.len() - 1 {
            self.selected_index += 1;
        }
    }

    /// Select the previous option.
    #[allow(clippy::missing_const_for_fn)]
    pub fn prev(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::state::GameState;
    use crate::shared::input::{InputContext, InputContextStack, InputRouter};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key_event(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    #[test]
    fn test_gamestate_main_menu_variant() {
        let state = GameState::MainMenu;
        assert_ne!(state, GameState::Running);
        assert_ne!(state, GameState::Paused);
    }

    #[test]
    fn test_input_context_main_menu_variant() {
        let ctx = InputContext::MainMenu;
        assert_ne!(ctx, InputContext::Normal);
    }

    #[test]
    fn test_menu_state_resource() {
        let menu = MenuState::default();
        // Should default to first option (Start Game)
        assert_eq!(menu.selected_index, 0);
        // Should have options
        assert!(menu.options.len() >= 2);
        assert_eq!(menu.options[0], "Start Game");
        assert_eq!(menu.options[1], "Quit");
    }

    #[test]
    fn test_menu_navigation_down() {
        let mut menu = MenuState::default();
        // default is 0

        menu.next();
        assert_eq!(menu.selected_index, 1);

        // Should wrap or clamp (design decision: clamp)
        menu.next();
        assert_eq!(menu.selected_index, 1); // Assuming 2 options
    }

    #[test]
    fn test_menu_navigation_up() {
        let mut menu = MenuState {
            selected_index: 1,
            ..Default::default()
        };

        menu.prev();
        assert_eq!(menu.selected_index, 0);

        menu.prev();
        assert_eq!(menu.selected_index, 0); // Clamp
    }

    #[test]
    fn test_menu_input_start_game() {
        let mut world = World::new();
        world.insert_resource(GameState::MainMenu);

        let mut stack = InputContextStack::default();
        stack.push(InputContext::MainMenu);
        world.insert_resource(stack);

        world.insert_resource(MenuState { selected_index: 0, ..Default::default() }); // "Start Game" selected

        let mut router = InputRouter::new();
        router.route(&mut world, key_event(KeyCode::Enter));

        // Should transition to Running
        assert_eq!(*world.resource::<GameState>(), GameState::Running);
        // Should switch input context to Normal
        assert_eq!(world.resource::<InputContextStack>().current(), InputContext::Normal);
    }

    #[test]
    fn test_menu_input_quit() {
        let mut world = World::new();
        world.insert_resource(GameState::MainMenu);
        world.insert_resource(InputContextStack::default());
        world.resource_mut::<InputContextStack>().push(InputContext::MainMenu);

        world.insert_resource(MenuState { selected_index: 1, ..Default::default() }); // "Quit" selected

        let mut router = InputRouter::new();
        router.route(&mut world, key_event(KeyCode::Enter));

        // Should transition to Quitting
        assert_eq!(*world.resource::<GameState>(), GameState::Quitting);
    }
}
