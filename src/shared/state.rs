//! High-level game state management.
//!
//! This module defines the macro states of the game engine (e.g., whether
//! we are in a menu, actively running the simulation, or paused). Systems
//! use this to determine if they should execute on a given frame.

use bevy_ecs::prelude::Resource;

/// Represents the high-level state of the game loop.
///
/// This resource dictates which systems run during the Bevy `Update` schedule.
///
/// # Examples
///
/// ```
/// use scale::shared::state::GameState;
///
/// let mut state = GameState::default();
/// assert_eq!(state, GameState::MainMenu);
///
/// state = GameState::Running;
/// assert_eq!(state, GameState::Running);
/// ```
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
