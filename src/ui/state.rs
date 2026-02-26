use bevy_ecs::prelude::*;

/// Global UI state resource.
///
/// Controls the visibility of major UI elements.
#[derive(Resource, Default, Debug, Clone)]
pub struct UiState {
    /// If true, suppresses the global UI (Status Bar, Info Panel, Chronicle, Tech Tree).
    /// Used for "Cinematic" or "Possession" modes.
    pub suppress_global_ui: bool,
}
