//! Construction
//!
//! Handles the construction of buildings and great works.

pub mod great_works;
pub use great_works::*;

use crate::layer1::items::ItemType;
use bevy_ecs::prelude::*;

#[derive(Clone, Debug)]
pub struct ConstructionCost {
    pub item_type: ItemType,
    pub amount: u32,
}

#[derive(Component, Debug, Clone)]
pub struct ConstructionProgress {
    pub total_work_required: f32,
    pub current_work: f32,
}
pub mod material_provenance;
