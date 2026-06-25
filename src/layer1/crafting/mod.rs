use bevy_ecs::prelude::*;

#[derive(Event, Debug)]
pub struct CraftEvent {
    pub crafter: Entity,
    pub item_type: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Quality {
    Poor,
    Normal,
    Masterpiece,
}
pub mod byproducts;
