use bevy::prelude::*;

#[derive(Component)]
pub struct PlanetNode;

#[derive(Component)]
pub struct PlanetTexture {
    pub id: String,
    pub scars: Vec<String>,
}
