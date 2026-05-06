use bevy_ecs::prelude::*;

/// Defines the current view mode of the game.
///
/// The game loop switches between these modes to determine which systems to run
/// and which UI to render.
#[derive(Resource, Default, Debug, PartialEq, Eq, Clone, Copy)]
pub enum ViewMode {
    /// The Colony View (Layer 1).
    /// Focuses on tile-based management of the colony surface.
    #[default]
    Colony,
    /// The System View (Layer 2).
    /// Focuses on orbital mechanics, fleet movement, and system resources.
    System,
}
