//! Crafting system for the colony.
//!
//! This module handles the creation of items from resources. It allows entities to
//! trigger crafting actions and handles the generation of primary items along with
//! associated [`byproducts`].
//!
//! # Crafting Cycle
//! 1. An entity triggers a [`CraftEvent`].
//! 2. The event is processed by systems checking available resources.
//! 3. Work progress increments over time in a `CraftingBuilding`.
//! 4. Once complete, primary outputs and byproducts are added to an `Inventory`.

use bevy_ecs::prelude::*;

/// Event triggered when an entity begins crafting an item.
///
/// Listen to this event to initialize crafting sequences.
///
/// ## Examples
/// ```
/// use bevy_ecs::prelude::*;
/// use scale::layer1::crafting::CraftEvent;
///
/// let mut world = World::new();
/// let crafter = world.spawn_empty().id();
///
/// let event = CraftEvent {
///     crafter,
///     item_type: "Steel Axe".to_string(),
/// };
/// ```
#[derive(Event, Debug)]
pub struct CraftEvent {
    /// The entity performing the crafting.
    pub crafter: Entity,
    /// The string identifier of the item being crafted.
    pub item_type: String,
}

/// The outcome quality of a crafted item.
///
/// Quality can affect the value, durability, or utility of the output.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Quality {
    /// Substandard quality, often caused by low skill or poor materials.
    Poor,
    /// Standard acceptable quality.
    Normal,
    /// Exceptional quality, granting bonuses.
    Masterpiece,
}

pub mod byproducts;
