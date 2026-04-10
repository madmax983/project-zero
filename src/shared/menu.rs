use crate::setup::StartScenarioId;
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
    use crate::layer1::{Chronicle, ColonyResources, Pop};
    use crate::platform::input::{GameKeyCode, GameKeyEvent};
    use crate::setup::{
        setup_world_with_config, start_scenario_definition, ActiveStartScenario, SetupConfig,
        StartScenarioId,
    };
    use crate::shared::input::{route_input, InputContext, InputContextStack};
    use crate::shared::state::GameState;

    fn key_event(code: GameKeyCode) -> GameKeyEvent {
        GameKeyEvent::new(code)
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

    #[test]
    fn test_menu_input_start_game() {
        let mut world = World::new();
        world.init_resource::<bevy::time::Time>();
        world.init_resource::<bevy::ecs::event::Events<crate::layer2::events::LaunchEvent>>();
        world
            .init_resource::<bevy::ecs::event::Events<crate::layer2::events::ShipDestroyedEvent>>();
        world.init_resource::<bevy::ecs::event::Events<crate::layer2::events::DetectionEvent>>();
        world.init_resource::<crate::shared::time::SimulationTime>();
        world.init_resource::<crate::layer1::chronicle::Chronicle>();
        world.init_resource::<bevy::ecs::event::Events<crate::layer2::skyhooks::LaunchIntent>>();
        world.init_resource::<bevy::ecs::event::Events<crate::layer2::cartographers_curse::SellTelemetryEvent>>();
        world.init_resource::<crate::layer1::stress::TraumaTracker>();
        world
            .init_resource::<bevy::ecs::event::Events<crate::layer1::chronicle::AddChronicleEvent>>(
            );
        world.init_resource::<bevy::ecs::event::Events<crate::layer1::economy::foreclosure::BuybackEvent>>();
        world.init_resource::<bevy::ecs::event::Events<crate::layer1::economy::foreclosure::ForecloseEvent>>();
        world.insert_resource(crate::layer1::economy::resources::ColonyResources::default());
        world.insert_resource(GameState::MainMenu);
        let active = start_scenario_definition(StartScenarioId::Classic);
        world.insert_resource(ActiveStartScenario {
            id: active.id,
            name: active.name,
            difficulty: active.difficulty,
        });

        let mut stack = InputContextStack::default();
        stack.push(InputContext::MainMenu);
        world.insert_resource(stack);

        world.insert_resource(MenuState {
            selected_index: 0,
            selected_scenario: StartScenarioId::SocialDrama,
            ..Default::default()
        }); // "Start Game" selected

        route_input(&mut world, key_event(GameKeyCode::Enter));

        // Should transition to Running
        assert_eq!(*world.resource::<GameState>(), GameState::Running);
        assert_eq!(
            world.resource::<ActiveStartScenario>().id,
            StartScenarioId::SocialDrama
        );
        // Should switch input context to Normal
        assert_eq!(
            world.resource::<InputContextStack>().current(),
            InputContext::Normal
        );
    }

    #[test]
    fn test_menu_input_quit() {
        let mut world = World::new();
        world.insert_resource(GameState::MainMenu);
        let active = start_scenario_definition(StartScenarioId::Classic);
        world.insert_resource(ActiveStartScenario {
            id: active.id,
            name: active.name,
            difficulty: active.difficulty,
        });
        world.insert_resource(InputContextStack::default());
        world
            .resource_mut::<InputContextStack>()
            .push(InputContext::MainMenu);

        world.insert_resource(MenuState {
            selected_index: 1,
            ..Default::default()
        }); // "Quit" selected

        route_input(&mut world, key_event(GameKeyCode::Enter));

        // Should transition to Quitting
        assert_eq!(*world.resource::<GameState>(), GameState::Quitting);
    }

    #[test]
    fn test_menu_start_game_applies_ground_survival_state() {
        let mut world = setup_world_with_config(SetupConfig {
            headless: true,
            ..Default::default()
        });
        *world.resource_mut::<GameState>() = GameState::MainMenu;

        let mut stack = InputContextStack::default();
        stack.push(InputContext::MainMenu);
        world.insert_resource(stack);

        world.insert_resource(MenuState {
            selected_index: 0,
            selected_scenario: StartScenarioId::GroundSurvival,
            ..Default::default()
        });

        route_input(&mut world, key_event(GameKeyCode::Enter));

        assert_eq!(*world.resource::<GameState>(), GameState::Running);
        assert_eq!(
            world.query::<&Pop>().iter(&world).count(),
            4,
            "Ground Survival should reduce the opening pop count on the menu path"
        );
        assert!(
            world.resource::<ColonyResources>().food < 10.0,
            "Ground Survival should reduce food on the menu path"
        );
        assert!(
            world
                .resource::<Chronicle>()
                .events
                .iter()
                .any(|event| event.text.contains("hard landing")),
            "Ground Survival intro text should be added on the menu path"
        );
    }

    #[test]
    fn test_menu_start_game_applies_social_drama_state() {
        let mut world = setup_world_with_config(SetupConfig {
            headless: true,
            ..Default::default()
        });
        *world.resource_mut::<GameState>() = GameState::MainMenu;

        let mut stack = InputContextStack::default();
        stack.push(InputContext::MainMenu);
        world.insert_resource(stack);

        world.insert_resource(MenuState {
            selected_index: 0,
            selected_scenario: StartScenarioId::SocialDrama,
            ..Default::default()
        });

        route_input(&mut world, key_event(GameKeyCode::Enter));

        let immigrant_count = world
            .query::<&crate::layer1::social::old_guard::Generation>()
            .iter(&world)
            .filter(|generation| {
                **generation == crate::layer1::social::old_guard::Generation::Immigrant
            })
            .count();
        assert_eq!(immigrant_count, 4);
        assert!(
            world
                .resource::<Chronicle>()
                .events
                .iter()
                .any(|event| event.text.contains("powder keg")),
            "Social Drama intro text should be added on the menu path"
        );
    }

    #[test]
    fn test_menu_start_game_applies_layer2_ready_state() {
        let mut world = setup_world_with_config(SetupConfig {
            headless: true,
            ..Default::default()
        });
        *world.resource_mut::<GameState>() = GameState::MainMenu;

        let mut stack = InputContextStack::default();
        stack.push(InputContext::MainMenu);
        world.insert_resource(stack);

        world.insert_resource(MenuState {
            selected_index: 0,
            selected_scenario: StartScenarioId::Layer2Ready,
            ..Default::default()
        });

        route_input(&mut world, key_event(GameKeyCode::Enter));

        assert_eq!(
            *world.resource::<crate::layer2::visibility::SystemVisibility>(),
            crate::layer2::visibility::SystemVisibility::Full,
            "Layer 2 Ready should unlock system visibility on the menu path"
        );
        assert!(
            world
                .resource::<Chronicle>()
                .events
                .iter()
                .any(|event| event.text.contains("orbital charter")),
            "Layer 2 Ready intro text should be added on the menu path"
        );
    }
}
