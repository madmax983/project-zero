use bevy_ecs::prelude::Resource;

/// Represents the high-level state of the game loop.
#[derive(Resource, Default, PartialEq, Eq, Clone, Copy, Debug)]
pub enum GameState {
    /// The game is in the main menu.
    #[default]
    MainMenu,
    /// The simulation is running normally.
    Running,
    /// The simulation is paused, but input is still handled.
    Paused,
    /// The game is in the process of shutting down.
    Quitting,
}

use bevy_ecs::event::Event;

/// Event emitted when the game ends.
#[derive(Event, Debug, Clone, PartialEq, Eq)]
pub enum GameOverEvent {
    /// The colony was fully assimilated by Symbiont Spores.
    SymbiontAssimilation,
}
