use bevy_ecs::prelude::*;

/// Global UI state resource.
///
/// Controls coarse UI suppression outside the shell-managed pane model.
#[derive(Resource, Default, Debug, Clone)]
pub struct UiState {
    /// If true, suppresses the shell and renders only the fullscreen map/notifications path.
    /// Used for "Cinematic" or "Possession" modes.
    pub suppress_global_ui: bool,
}
