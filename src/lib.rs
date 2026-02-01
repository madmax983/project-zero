use bevy::prelude::*;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Loading,
    Playing,
    Paused,
}

pub mod layer1;
pub mod layer2;
pub mod layer3;
pub mod ui;
