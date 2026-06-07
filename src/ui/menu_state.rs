use crate::shared::scenario::StartScenarioId;
use bevy_ecs::prelude::*;

/// Resources for the Main Menu.
#[derive(Resource)]
pub struct MenuState {
    /// The index of the currently selected option.
    pub selected_index: usize,
    /// The list of menu options.
    pub options: Vec<String>,
    /// The currently selected built-in start scenario.
    pub selected_scenario: StartScenarioId,
}

impl Default for MenuState {
    fn default() -> Self {
        Self {
            selected_index: 0,
            options: vec!["Start Game".to_string(), "Quit".to_string()],
            selected_scenario: StartScenarioId::Classic,
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

    /// Select the next start scenario.
    pub fn next_scenario(&mut self) {
        let scenarios = StartScenarioId::all();
        let current = scenarios
            .iter()
            .position(|id| *id == self.selected_scenario)
            .unwrap_or(0);
        if current + 1 < scenarios.len() {
            self.selected_scenario = scenarios[current + 1];
        }
    }

    /// Select the previous start scenario.
    pub fn prev_scenario(&mut self) {
        let scenarios = StartScenarioId::all();
        let current = scenarios
            .iter()
            .position(|id| *id == self.selected_scenario)
            .unwrap_or(0);
        if current > 0 {
            self.selected_scenario = scenarios[current - 1];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::scenario::StartScenarioId;

    #[test]
    fn test_menu_state_resource() {
        let menu = MenuState::default();
        // Should default to first option (Start Game)
        assert_eq!(menu.selected_index, 0);
        assert_eq!(menu.selected_scenario, StartScenarioId::Classic);
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
    fn test_menu_scenario_cycles_forward_and_back() {
        let mut menu = MenuState::default();

        menu.next_scenario();
        assert_eq!(menu.selected_scenario, StartScenarioId::GroundSurvival);

        menu.prev_scenario();
        assert_eq!(menu.selected_scenario, StartScenarioId::Classic);
    }
}
