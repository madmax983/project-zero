use bevy_ecs::prelude::*;

#[derive(Component, Debug, Clone, PartialEq, Eq, Default)]
pub enum CulturalTag {
    #[default]
    Default,
    Fringe,
}
